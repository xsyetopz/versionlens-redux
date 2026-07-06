use std::iter::successors;
type RegistryUrlConfigs = Vec<RegistryUrlConfig>;

fn go_no_proxy_patterns(env: &[(String, String)]) -> Vec<String> {
    env_config_value(env, "GONOPROXY")
        .or_else(|| env_config_value(env, "GOPRIVATE"))
        .map(parse_go_glob_patterns)
        .unwrap_or_default()
}

fn parse_go_glob_patterns(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|value| value.trim())
        .filter(|pattern| !pattern.is_empty() && *pattern != "none")
        .map(|value| value.to_owned())
        .collect()
}

fn go_module_matches_patterns(module: &str, patterns: &[String]) -> bool {
    go_module_prefixes(module).any(|prefix| {
        patterns
            .iter()
            .any(|pattern| go_path_pattern_matches_bytes(pattern.as_bytes(), prefix.as_bytes()))
    })
}

fn go_module_prefixes(module: &str) -> impl Iterator<Item = &str> {
    successors(Some(module), |prefix| {
        prefix.rsplit_once('/').map(|(parent, _)| parent)
    })
}

fn go_path_pattern_matches_bytes(pattern: &[u8], value: &[u8]) -> bool {
    match pattern.split_first() {
        None => value.is_empty(),
        Some((&b'*', rest)) => {
            go_path_pattern_matches_bytes(rest, value)
                || value.split_first().is_some_and(|(&byte, tail)| {
                    byte != b'/' && go_path_pattern_matches_bytes(pattern, tail)
                })
        }
        Some((&b'?', rest)) => value
            .split_first()
            .is_some_and(|(&byte, tail)| byte != b'/' && go_path_pattern_matches_bytes(rest, tail)),
        Some((&b'[', _)) => value.split_first().is_some_and(|(&byte, tail)| {
            byte != b'/'
                && go_path_character_class_matches(pattern, byte)
                    .is_some_and(|rest| go_path_pattern_matches_bytes(rest, tail))
        }),
        Some((&expected, rest)) => value.split_first().is_some_and(|(&byte, tail)| {
            byte == expected && go_path_pattern_matches_bytes(rest, tail)
        }),
    }
}

fn go_path_character_class_matches(pattern: &[u8], value: u8) -> Option<&[u8]> {
    let mut index = 1;
    let negated = matches!(pattern.get(index), Some(b'^'));
    if negated {
        index += 1;
    }

    let mut matched = false;
    let mut has_term = false;
    while let Some(&term) = pattern.get(index) {
        if term == b']' {
            has_term = true;
            break;
        }

        if matches!(pattern.get(index + 1), Some(b'-'))
            && let Some(&range_end) = pattern.get(index + 2)
            && range_end != b']'
        {
            if term <= value && value <= range_end {
                matched = true;
            }
            index += 3;
        } else {
            if term == value {
                matched = true;
            }
            index += 1;
        }
    }

    if !has_term || index == usize::from(negated) + 1 {
        return None;
    }

    (matched != negated).then_some(&pattern[index + 1..])
}

fn go_proxy_disables_default_registry(env: &[(String, String)]) -> bool {
    env_config_value(env, "GOPROXY").is_some_and(|value| {
        !value
            .split([',', '|'])
            .map(|value| value.trim())
            .take_while(|entry| *entry != "off")
            .any(|entry| !entry.is_empty() && entry != "direct")
    })
}

fn python_registry_url_configs(input: &DocumentInput) -> RegistryUrlConfigs {
    let mut urls = parse_python_registry_urls(&input.text);
    urls.extend(parse_pipfile_source_urls(&input.text));
    urls.extend(
        dot_file_texts(input, &["pip.conf"])
            .iter()
            .flat_map(|text| parse_pip_conf_registry_urls(text)),
    );
    urls.extend(
        dot_file_texts(input, &["uv.toml"])
            .iter()
            .flat_map(|text| parse_uv_registry_urls(text)),
    );
    urls.extend(parse_pip_env_registry_urls(&env_entries(input)));

    let mut configs = vec![];
    for url in urls {
        if configs.iter().any(|config: &RegistryUrlConfig| {
            config.url == url && config.ecosystem == Python
        }) {
            continue;
        }
        configs.push(RegistryUrlConfig {
            ecosystem: Python,
            url,
        });
    }
    configs
}

fn npm_registry_timeout_ms(timeout_ms: Option<u64>) -> u64 {
    match timeout_ms {
        Some(0) => 30_000,
        Some(timeout_ms) => timeout_ms,
        None => 300_000,
    }
}

fn parse_mix_hex_api_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let start = line.find("api_url:")? + "api_url:".len();
            let value = line.get(start..)?.trim_start();
            let value = value.strip_prefix('"')?;
            let end = value.find('"')?;
            let url = value.get(..end)?.trim();
            (!url.is_empty()).then(|| url.to_owned())
        })
        .collect()
}

fn parse_rebar_packages_cdn_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let start = line.find("rebar_packages_cdn")? + "rebar_packages_cdn".len();
            let value = line.get(start..)?.trim_start();
            let value = value.strip_prefix(',')?.trim_start();
            let value = value.strip_prefix('"')?;
            let end = value.find('"')?;
            let url = value.get(..end)?.trim();
            (!url.is_empty()).then(|| url.to_owned())
        })
        .collect()
}

fn hex_registry_url_configs(input: &DocumentInput, kind: ManifestKind) -> Vec<String> {
    if let Some(url) = env_config_value(&env_entries(input), "HEX_API_URL")
        .map(|value| value.trim())
        .filter(|url| !url.is_empty())
    {
        return vec![url.to_owned()];
    }

    if kind == RebarConfig {
        if let Some(url) = env_config_value(&env_entries(input), "HEX_CDN")
            .map(|value| value.trim())
            .filter(|url| !url.is_empty())
        {
            return vec![url.to_owned()];
        }

        let urls = parse_rebar_packages_cdn_urls(&input.text);
        if !urls.is_empty() {
            return urls;
        }
    }

    parse_mix_hex_api_urls(&input.text)
}

fn maven_auth_entries(input: &DocumentInput) -> Vec<MavenAuthEntry> {
    dot_file_texts(input, &["settings.xml"])
        .iter()
        .flat_map(|text| parse_maven_settings_auth_entries(text))
        .collect()
}

fn maven_uses_mirror(input: &DocumentInput) -> bool {
    dot_file_texts(input, &["settings.xml"])
        .iter()
        .any(|text| !parse_maven_settings_mirror_urls(text).is_empty())
}

fn parse_maven_registry_urls(input: &DocumentInput, kind: ManifestKind) -> Vec<String> {
    let settings_texts = dot_file_texts(input, &["settings.xml"]);
    let mirrors = settings_texts
        .iter()
        .flat_map(|text| parse_maven_settings_mirrors(text))
        .collect::<Vec<_>>();
    let mirror_urls = all_repository_mirror_urls(&mirrors);
    if !mirror_urls.is_empty() {
        return mirror_urls;
    }

    let document_repositories = match kind {
        GradleBuild
        | GradleSettings
        | GradleVersionCatalogToml => parse_gradle_registry_repositories(input, kind),
        SbtBuild => parse_sbt_maven_repositories(&input.text),
        ClojureDepsEdn => parse_clojure_maven_repositories(&input.text),
        LeiningenProjectClj => parse_leiningen_maven_repositories(&input.text),
        _ => parse_maven_pom_repositories(&input.text),
    };
    let repositories = document_repositories.into_iter().chain(
        settings_texts
            .iter()
            .flat_map(|text| parse_maven_settings_repositories(text)),
    );

    mirrored_maven_repository_urls(repositories, &mirrors)
}

fn parse_gradle_registry_repositories(
    input: &DocumentInput,
    kind: ManifestKind,
) -> Vec<MavenNamedRepository> {
    if kind == GradleVersionCatalogToml {
        return dot_file_texts(input, &["settings.gradle", "settings.gradle.kts"])
            .iter()
            .flat_map(|text| parse_gradle_dependency_maven_repositories(text))
            .collect();
    }

    let repositories = if kind == GradleBuild {
        parse_gradle_dependency_maven_repositories(&input.text)
    } else {
        parse_gradle_maven_repositories(&input.text)
    };
    if kind == GradleBuild {
        let settings_texts = dot_file_texts(input, &["settings.gradle", "settings.gradle.kts"]);
        let settings_repositories = settings_texts
            .iter()
            .flat_map(|text| parse_gradle_dependency_maven_repositories(text))
            .collect::<Vec<_>>();
        if repositories.is_empty() || gradle_settings_repositories_override_project(&settings_texts)
        {
            return settings_repositories;
        }
    }
    repositories
}

fn parse_gradle_plugin_registry_urls(
    input: &DocumentInput,
    kind: ManifestKind,
) -> RegistryUrlConfigs {
    if !matches!(
        kind,
        GradleBuild
            | GradleSettings
            | GradleVersionCatalogToml
    ) {
        return vec![];
    }

    let mut repositories = parse_gradle_plugin_maven_repositories(&input.text);
    if matches!(
        kind,
        GradleBuild | GradleVersionCatalogToml
    ) {
        repositories.extend(
            dot_file_texts(input, &["settings.gradle", "settings.gradle.kts"])
                .iter()
                .flat_map(|text| parse_gradle_plugin_maven_repositories(text)),
        );
    }

    repositories
        .into_iter()
        .map(|repository| RegistryUrlConfig {
            ecosystem: Maven,
            url: repository.url,
        })
        .collect()
}

fn gradle_settings_repositories_override_project(settings_texts: &[String]) -> bool {
    settings_texts.iter().any(|text| {
        text.contains("RepositoriesMode.PREFER_SETTINGS")
            || text.contains("RepositoriesMode.FAIL_ON_PROJECT_REPOS")
    })
}

fn is_gradle_plugin_marker(dependency: &Dependency) -> bool {
    dependency.group == "plugins" && dependency.name.ends_with(".gradle.plugin")
}

fn all_repository_mirror_urls(mirrors: &[MavenMirror]) -> Vec<String> {
    mirrors
        .iter()
        .filter(|mirror| mirror.mirror_of == "*")
        .map(|mirror| mirror.url.as_str().to_owned())
        .collect()
}

fn mirrored_maven_repository_urls(
    repositories: impl Iterator<Item = MavenNamedRepository>,
    mirrors: &[MavenMirror],
) -> Vec<String> {
    repositories
        .map(|repository| mirrored_maven_repository_url(repository, mirrors))
        .collect()
}

fn mirrored_maven_repository_url(
    repository: MavenNamedRepository,
    mirrors: &[MavenMirror],
) -> String {
    mirrors
        .iter()
        .find(|mirror| mirror.mirror_of == repository.id)
        .map_or(repository.url, |mirror| mirror.url.as_str().to_owned())
}
