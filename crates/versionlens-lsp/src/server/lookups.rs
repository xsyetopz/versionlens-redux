//! Registry and OSV lookups: scheduling them on the runtime, folding their results into the cache,
//! and reporting their progress to the editor.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use lsp_types::MessageType;
use serde_json::json;

use super::{Entry, Event, Found, Key, Pending, Server, key};
use crate::extract::Dep;
use crate::git_ref::GIT_REF;
use crate::osv::{self, Query};
use crate::registry::{self, Registry};
use crate::render;
use crate::version::{Req, Version};

/// How long a failed lookup is remembered before the next edit retries it.
const RETRY: Duration = Duration::from_secs(60);

impl Server {
    /// Drops every cached and in-flight lookup, after which `schedule_all` sends them again
    /// through the current HTTP client; a reply to an old lookup is ignored when it arrives.
    pub(super) fn restart_lookups(&mut self) {
        self.cache.clear();
        self.inflight.clear();
        self.lookups += 1;
    }

    pub(super) fn schedule_all(&mut self) {
        let uris: Vec<String> = self.docs.keys().cloned().collect();
        for uri in uris {
            self.schedule(&uri);
        }
    }

    pub(super) fn schedule(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        let wanted: Vec<Key> = doc.deps.iter().map(key).collect();
        for key in wanted {
            let ttl = self.settings.cache_for(key.0);
            let fresh = self
                .cache
                .get(&key)
                .is_some_and(|e| e.at.elapsed() < if e.result.is_ok() { ttl } else { RETRY });
            let Some(registry) = registry::get(key.0) else {
                continue;
            };
            if fresh || !self.inflight.insert(key.clone()) {
                continue;
            }
            self.fetch(registry, key);
        }
        self.start_progress();
        self.check_vulns(uri);
    }

    fn fetch(&self, registry: &'static Registry, key: Key) {
        let (http, tx, generation) = (self.http.clone(), self.events.0.clone(), self.lookups);
        self.runtime.spawn(async move {
            let started = Instant::now();
            let result = http
                .versions(registry, &key.1, key.2.as_deref(), &key.3, &key.4)
                .await;
            drop(tx.send(Event::Fetched(key, result, started.elapsed(), generation)));
        });
    }

    pub(super) fn event(&mut self, event: Event) {
        match event {
            Event::Fetched(_, _, _, generation) if generation != self.lookups => {}
            Event::Fetched(key, result, took, _) => {
                self.inflight.remove(&key);
                let result = match result {
                    Ok(found) => {
                        self.log(
                            MessageType::LOG,
                            format!(
                                "{} {}: {} versions in {took:?}",
                                key.0,
                                key.1,
                                found.list.len()
                            ),
                        );
                        let parse = |list: &[String]| -> Vec<Version> {
                            let mut out: Vec<Version> =
                                list.iter().filter_map(|v| Version::parse(v)).collect();
                            out.sort();
                            out
                        };
                        let refs = if key.0 == GIT_REF {
                            found.list.clone()
                        } else {
                            Vec::new()
                        };
                        Ok(Arc::new(Found {
                            versions: parse(&found.list),
                            refs,
                            tagged: found.latest.as_deref().and_then(Version::parse),
                            yanked: parse(&found.yanked),
                            deprecated: parse(&found.deprecated),
                        }))
                    }
                    Err(error) => {
                        self.log(
                            MessageType::WARNING,
                            format!("{} {}: {error}", key.0, key.1),
                        );
                        Err(error)
                    }
                };
                self.cache.insert(
                    key,
                    Entry {
                        at: Instant::now(),
                        result,
                    },
                );
                let uris: Vec<String> = self.docs.keys().cloned().collect();
                for uri in uris {
                    self.check_vulns(&uri);
                }
            }
            Event::Installed(uri, result) => self.install_done(&uri, result),
            Event::Osv(queries, result) => {
                for query in &queries {
                    self.osv_inflight.remove(query);
                }
                match result {
                    Ok(ids) => self.vulns.extend(queries.into_iter().zip(ids)),
                    Err(error) => {
                        self.log(
                            MessageType::ERROR,
                            format!("vulnerability check failed: {error}"),
                        );
                        if !self.osv_warned {
                            self.osv_warned = true;
                            let text = format!(
                                "VersionLens could not check vulnerabilities: {error}. Details are in the VersionLens output."
                            );
                            self.notify(
                                "window/showMessage",
                                json!({ "type": MessageType::WARNING, "message": text }),
                            );
                        }
                    }
                }
            }
        }
        self.dirty.get_or_insert_with(Instant::now);
    }

    /// Asks OSV about a document once every lookup it needs has finished.
    fn check_vulns(&mut self, uri: &str) {
        let Some(doc) = self.docs.get(uri) else {
            return;
        };
        if !doc.open
            || !self.settings.vulnerabilities
            || doc.deps.iter().any(|d| self.inflight.contains(&key(d)))
        {
            return;
        }
        let queries: Vec<Query> = doc
            .deps
            .iter()
            .filter_map(|dep| self.query(dep))
            .filter(|q| !self.vulns.contains_key(q) && !self.osv_inflight.contains(q))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        if queries.is_empty() {
            return;
        }
        self.osv_inflight.extend(queries.iter().cloned());
        let (http, tx) = (self.http.clone(), self.events.0.clone());
        self.runtime.spawn(async move {
            let result = osv::query(&http, &queries).await;
            drop(tx.send(Event::Osv(queries, result)));
        });
    }

    pub(super) fn query(&self, dep: &Dep) -> Option<Query> {
        let registry = registry::route(registry::get(dep.registry)?, &dep.name);
        let found = self.cache.get(&key(dep))?.result.as_ref().ok()?;
        let req = Req::parse(&dep.req, dep.bare(registry))?;
        let version = render::current(&req, &found.versions)?;
        let name = registry::expand(registry.osv_name, &dep.name, dep.source.as_deref()).ok()?;
        Some(Query {
            ecosystem: registry.osv?,
            name: name.replace("{version}", &version.raw),
            version: version.raw,
            upstream: registry.upstream,
            package: dep.name.clone(),
        })
    }

    fn start_progress(&mut self) {
        if self.caps.progress && self.progress.is_none() && !self.inflight.is_empty() {
            let token = format!("versionlens/{}", self.next_id + 1);
            self.progress = Some((token.clone(), false, self.inflight.len()));
            self.request(
                "window/workDoneProgress/create",
                json!({ "token": token }),
                Pending::Progress(token),
            );
        } else if let Some((_, _, total)) = &mut self.progress {
            *total = (*total).max(self.inflight.len());
        }
    }

    pub(super) fn begin_progress(&mut self, token: &str) {
        if let Some((current, started, total)) = &mut self.progress
            && current == token
        {
            *started = true;
            let value = json!({ "kind": "begin", "title": "VersionLens", "message": format!("checking {total} packages"), "percentage": 0 });
            self.notify("$/progress", json!({ "token": token, "value": value }));
        }
    }

    pub(super) fn report_progress(&mut self) {
        let Some((token, started, total)) = self.progress.clone() else {
            return;
        };
        if self.inflight.is_empty() {
            self.progress = None;
            if started {
                self.notify(
                    "$/progress",
                    json!({ "token": token, "value": { "kind": "end", "message": "done" } }),
                );
            }
        } else if started {
            let done = total.saturating_sub(self.inflight.len());
            let value = json!({ "kind": "report", "message": format!("{done}/{total}"), "percentage": done * 100 / total.max(1) });
            self.notify("$/progress", json!({ "token": token, "value": value }));
        }
    }
}
