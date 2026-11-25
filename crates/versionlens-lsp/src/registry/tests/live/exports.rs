//! OSV's bulk exports, read whole, for the registries [`super::NO_KNOWN_ADVISORY`] names: each
//! export a registry asks must hold no entry for that registry's packages, so the samples' empty
//! answers stay a checked fact. OSV publishes one `all.zip` per ecosystem and the list of
//! ecosystems beside them: google.github.io/osv.dev/data/#data-dumps.
//!
//! An export is kept in the system's temporary folder and asked for again with its `ETag`, so a
//! rerun downloads only an export that changed. `unzip`, which macOS and the Linux CI images ship,
//! streams the archive, since the crate has no zip reader.

use std::collections::BTreeSet;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;

use super::block_on;

const BUCKET: &str = "https://osv-vulnerabilities.storage.googleapis.com";
const AGENT: &str = "versionlens live check";

/// Ecosystem names OSV would use for the registries that ask GIT: none may appear in its list.
const ABSENT: &[&str] = &[
    "D",
    "Dub",
    "Chef",
    "Elm",
    "Haxe",
    "Haxelib",
    "MoonBit",
    "Mooncakes",
    "PureScript",
    "V",
    "VPM",
    "Lean",
    "Reservoir",
    "Typst",
    "Ansible",
    "Ansible Galaxy",
    "Galaxy",
    "Forgejo",
    "Gitea",
];

/// The `ecosystem` and `name` of every package an entry affects, and the repository of every
/// GIT range, lowercased.
#[derive(Default)]
struct Entry {
    id: String,
    packages: Vec<(String, String)>,
    repos: Vec<String>,
}

fn client() -> reqwest::Client {
    // Run alone, the test builds the first client, so it installs the provider `Http::new` would.
    drop(rustls::crypto::ring::default_provider().install_default());
    reqwest::Client::builder()
        .user_agent(AGENT)
        .build()
        .unwrap()
}

fn text(url: &str) -> String {
    block_on(async {
        let response = client().get(url).send().await.unwrap();
        assert!(
            response.status().is_success(),
            "{url}: HTTP {}",
            response.status()
        );
        response.text().await.unwrap()
    })
}

/// The path of `{ecosystem}/all.zip`, downloaded unless the copy kept from an earlier run still
/// matches the export's `ETag`.
fn export(ecosystem: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("versionlens-osv-exports");
    std::fs::create_dir_all(&dir).unwrap();
    let (zip, tag) = (
        dir.join(format!("{ecosystem}.zip")),
        dir.join(format!("{ecosystem}.etag")),
    );
    let url = format!("{BUCKET}/{ecosystem}/all.zip");
    block_on(async {
        let mut request = client().get(&url);
        if zip.exists()
            && let Ok(etag) = std::fs::read_to_string(&tag)
        {
            request = request.header("if-none-match", etag);
        }
        let mut response = request.send().await.unwrap();
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            return;
        }
        assert!(
            response.status().is_success(),
            "{url}: HTTP {}",
            response.status()
        );
        let etag = response
            .headers()
            .get("etag")
            .map(|v| v.to_str().unwrap().to_owned());
        let partial = dir.join(format!("{ecosystem}.part"));
        let mut file = std::fs::File::create(&partial).unwrap();
        while let Some(chunk) = response.chunk().await.unwrap() {
            file.write_all(&chunk).unwrap();
        }
        std::fs::rename(&partial, &zip).unwrap();
        std::fs::write(&tag, etag.unwrap_or_default()).unwrap();
    });
    zip
}

/// Every entry of an ecosystem's export.
fn entries(ecosystem: &str) -> Vec<Entry> {
    let zip = export(ecosystem);
    let mut unzip = Command::new("unzip")
        .arg("-p")
        .arg(&zip)
        .stdout(Stdio::piped())
        .spawn()
        .expect("unzip runs");
    let out = BufReader::new(unzip.stdout.take().unwrap());
    let lower = |v: &Value| v.as_str().unwrap_or_default().to_ascii_lowercase();
    let read: Vec<Entry> = serde_json::Deserializer::from_reader(out)
        .into_iter::<Value>()
        .map(|doc| {
            let doc = doc.unwrap();
            let affected = doc["affected"].as_array().cloned().unwrap_or_default();
            Entry {
                id: lower(&doc["id"]),
                packages: affected
                    .iter()
                    .map(|a| {
                        (
                            lower(&a["package"]["ecosystem"]),
                            lower(&a["package"]["name"]),
                        )
                    })
                    .collect(),
                repos: affected
                    .iter()
                    .flat_map(|a| a["ranges"].as_array().cloned().unwrap_or_default())
                    .map(|r| lower(&r["repo"]))
                    .filter(|r| !r.is_empty())
                    .collect(),
            }
        })
        .collect();
    assert!(unzip.wait().unwrap().success(), "unzip {}", zip.display());
    assert!(!read.is_empty(), "{ecosystem}/all.zip holds no entries");
    eprintln!("export {ecosystem}: {} entries", read.len());
    read
}

/// The ids of entries with a package or repository that `hit` accepts.
fn hits(
    entries: &[Entry],
    package: impl Fn(&str) -> bool,
    repo: impl Fn(&str) -> bool,
) -> Vec<String> {
    entries
        .iter()
        .filter(|e| e.packages.iter().any(|(_, n)| package(n)) || e.repos.iter().any(|r| repo(r)))
        .map(|e| e.id.clone())
        .collect()
}

/// The repository URL a GitHub `owner/repo` names in OSV's GIT ranges, lowercased.
fn github(path: &str) -> String {
    let path = path.trim_end_matches('/').trim_end_matches(".git");
    format!("https://github.com/{}", path.to_ascii_lowercase())
}

/// The Julia standard libraries outside the `_jll` binaries: a folder, or a `.version` file for
/// one kept in its own repository, under JuliaLang/julia's `stdlib`.
fn julia_stdlibs() -> BTreeSet<String> {
    let listing = text("https://api.github.com/repos/JuliaLang/julia/contents/stdlib?ref=master");
    let listing: Value = serde_json::from_str(&listing).unwrap();
    listing
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|e| {
            let name = e["name"].as_str()?;
            let name = match e["type"].as_str()? {
                "dir" => name,
                _ => name.strip_suffix(".version")?,
            };
            (!name.ends_with("_jll")).then(|| name.to_ascii_lowercase())
        })
        .collect()
}

/// The repositories of every Elm package (package.elm-lang.org/search.json), every package in
/// the PureScript package set (`packages.json` of purescript/package-sets), and every module on
/// mooncakes.io (its modules API).
fn listed_repos() -> [(&'static str, BTreeSet<String>); 3] {
    let elm: Value =
        serde_json::from_str(&text("https://package.elm-lang.org/search.json")).unwrap();
    let elm = elm
        .as_array()
        .unwrap()
        .iter()
        .map(|p| github(p["name"].as_str().unwrap()));
    let sets =
        text("https://raw.githubusercontent.com/purescript/package-sets/master/packages.json");
    let sets: Value = serde_json::from_str(&sets).unwrap();
    let purescript = sets
        .as_object()
        .unwrap()
        .values()
        .filter_map(|p| p["repo"].as_str());
    let modules: Value =
        serde_json::from_str(&text("https://mooncakes.io/api/v0/modules")).unwrap();
    let moonbit = modules
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|m| m["repository"].as_str());
    let url = |r: &str| {
        r.trim_end_matches('/')
            .trim_end_matches(".git")
            .to_ascii_lowercase()
    };
    [
        ("elm", elm.collect()),
        ("purescript", purescript.map(url).collect()),
        ("mooncakes", moonbit.map(url).collect()),
    ]
}

#[test]
#[ignore = "needs the network"]
fn advisory_free_registries_hold_no_osv_entries() {
    // Arrange
    let listed = text(&format!("{BUCKET}/ecosystems.txt"));
    let ecosystems: BTreeSet<&str> = listed.lines().map(str::trim).collect();
    let stdlibs = julia_stdlibs();
    let lists = listed_repos();
    let (julia, go, crates, git) = (
        entries("Julia"),
        entries("Go"),
        entries("crates.io"),
        entries("GIT"),
    );
    let none = |_: &str| false;
    let plugin = |n: &str| n.contains("/packer-plugin-");
    let samples = [
        ("dub", "vibe-d/vibe.d"),
        ("chef", "sous-chefs/nginx"),
        ("haxelib", "haxefoundation/hxcpp"),
        ("vpm", "vlang/vsl"),
        ("reservoir", "leanprover-community/mathlib4"),
        ("galaxy-role", "geerlingguy/ansible-role-java"),
    ];
    let hosts = [
        "codeberg.org/",
        "code.forgejo.org/",
        "data.forgejo.org/",
        "gitea.com/",
    ];

    // Act
    let mut found: Vec<(&str, Vec<String>)> = vec![
        ("julia-stdlib", hits(&julia, |n| stdlibs.contains(n), none)),
        ("packer", hits(&go, plugin, none)),
        ("packer", hits(&git, none, plugin)),
        (
            "typst-compiler",
            hits(&crates, |n| n.starts_with("typst"), none),
        ),
        ("typst-compiler", hits(&git, none, |r| r.contains("typst"))),
        (
            "galaxy-role",
            hits(&git, none, |r| r.contains("ansible-role")),
        ),
        (
            "forgejo",
            hits(&git, none, |r| {
                let r = r
                    .trim_start_matches("https://")
                    .trim_start_matches("http://");
                hosts.iter().any(|h| r.starts_with(h))
            }),
        ),
    ];
    for (id, repos) in &lists {
        found.push((id, hits(&git, none, |r| repos.contains(r))));
    }
    found.push(("purescript", hits(&git, none, |r| r.contains("purescript"))));
    found.push(("mooncakes", hits(&git, none, |r| r.contains("moonbit"))));
    for (id, repo) in samples {
        let repo = github(repo);
        found.push((id, hits(&git, none, |r| r == repo)));
    }

    // Assert
    // The matchers find what the exports do hold: the sample advisories of other registries.
    let controls = [
        ("Julia HTTP", hits(&julia, |n| n == "http", none).len()),
        ("crates.io time", hits(&crates, |n| n == "time", none).len()),
        (
            "Go golang.org/x/text",
            hits(&go, |n| n == "golang.org/x/text", none).len(),
        ),
        (
            "GIT curl",
            hits(&git, none, |r| r == "https://github.com/curl/curl").len(),
        ),
    ];
    assert!(controls.iter().all(|(_, n)| *n > 0), "{controls:?}");
    let named: Vec<&&str> = ABSENT.iter().filter(|e| ecosystems.contains(**e)).collect();
    assert!(named.is_empty(), "OSV now exports {named:?}");
    for asked in ["GIT", "Go", "Julia", "crates.io"] {
        assert!(ecosystems.contains(asked), "OSV no longer exports {asked}");
    }
    assert!(
        stdlibs.contains("dates") && stdlibs.contains("statistics"),
        "{stdlibs:?}"
    );
    for (id, repos) in &lists {
        assert!(repos.len() > 100, "{id} lists {} repositories", repos.len());
    }
    let held: Vec<String> = found
        .iter()
        .filter(|(_, ids)| !ids.is_empty())
        .map(|(id, ids)| format!("{id}: {ids:?}"))
        .collect();
    assert!(held.is_empty(), "exports hold entries for {held:?}");
    for (id, _) in &found {
        assert!(super::NO_KNOWN_ADVISORY.contains(id), "{id}");
    }
}
