//! Registry configuration each ecosystem's project files hold: npmrc chains, `NuGet`, Composer,
//! Python indexes, Cargo registries, and Maven mirrors, with their credentials.

use super::*;

#[test]
fn reads_npmrc_chain_with_scopes_auth_and_variables() {
    let root = project(
        "npm",
        &[
            (
                "app/package.json",
                r#"{ "dependencies": { "left-pad": "1.3.0", "@acme/ui": "2.0.0" } }"#,
            ),
            (
                "app/.npmrc",
                "registry=https://npm.example.com/repo/\n@acme:registry=https://acme.example/npm/\n//acme.example/npm/:_authToken=${ACME_TOKEN}\n",
            ),
            ("app/.env", "ACME_TOKEN=from-dotenv\n"),
            (
                "home/.npmrc",
                "; user\n//npm.example.com/:_auth=dTpw\n//acme.example/npm/:_authToken=user-token\n",
            ),
        ],
    );
    assert_eq!(
        targets(&root, "app/package.json", &env(&root, &[])),
        [
            (
                "left-pad".into(),
                vec![target(
                    "https://npm.example.com/repo/{enc}",
                    Some("Basic dTpw")
                )]
            ),
            (
                "@acme/ui".into(),
                vec![target(
                    "https://acme.example/npm/{enc}",
                    Some("Bearer from-dotenv")
                )]
            ),
        ]
    );
    // The environment beats `.env`, and `NPM_CONFIG_USERCONFIG` replaces `~/.npmrc`.
    std::fs::write(
        root.join("other.npmrc"),
        "//npm.example.com/repo/:username=u\n//npm.example.com/repo/:_password=cA==\n",
    )
    .unwrap();
    let other = root.join("other.npmrc");
    let vars = env(
        &root,
        &[
            ("ACME_TOKEN", "from-env"),
            ("NPM_CONFIG_USERCONFIG", other.to_str().unwrap()),
        ],
    );
    assert_eq!(
        targets(&root, "app/package.json", &vars),
        [
            (
                "left-pad".into(),
                vec![target(
                    "https://npm.example.com/repo/{enc}",
                    Some(&basic("u", "p"))
                )]
            ),
            (
                "@acme/ui".into(),
                vec![target(
                    "https://acme.example/npm/{enc}",
                    Some("Bearer from-env")
                )]
            ),
        ]
    );
}

#[test]
fn npm_auth_needs_whole_path_segment_and_default_registry_needs_no_target() {
    let root = project(
        "npm-host",
        &[
            ("package.json", r#"{ "dependencies": { "a": "1.0.0" } }"#),
            ("home/.npmrc", "//registry.npmjs.org.evil/:_authToken=x\n"),
        ],
    );
    assert_eq!(
        targets(&root, "package.json", &env(&root, &[])),
        [("a".into(), vec![])]
    );
    let vars = env(&root, &[("npm_config_registry", "https://mirror.example/")]);
    assert_eq!(
        targets(&root, "package.json", &vars),
        [(
            "a".into(),
            vec![target("https://mirror.example/{enc}", None)]
        )]
    );
}

#[test]
fn reads_nuget_config_files_nearest_first_with_clear_credentials_and_mapping() {
    let root = project(
        "nuget",
        &[
            (
                "nuget.config",
                r#"<configuration><packageSources><add key="nuget.org" value="https://api.nuget.org/v3/index.json" /></packageSources></configuration>"#,
            ),
            (
                "src/app/nuget.config",
                r#"<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <packageSources>
    <clear />
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
    <add key="Contoso Feed" value="https://contoso.example/v3/index.json" />
    <add key="Off" value="https://off.example/v3/index.json" />
  </packageSources>
  <disabledPackageSources><add key="Off" value="true" /></disabledPackageSources>
  <packageSourceCredentials>
    <Contoso_x0020_Feed>
      <add key="Username" value="u" />
      <add key="ClearTextPassword" value="%FEED_PASSWORD%" />
    </Contoso_x0020_Feed>
  </packageSourceCredentials>
  <packageSourceMapping>
    <packageSource key="nuget.org"><package pattern="*" /></packageSource>
    <packageSource key="Contoso Feed"><package pattern="Contoso.*" /></packageSource>
  </packageSourceMapping>
</configuration>"#,
            ),
            (
                "src/app/app.csproj",
                r#"<Project><ItemGroup><PackageReference Include="Contoso.Core" Version="1.0.0" /><PackageReference Include="Newtonsoft.Json" Version="13.0.1" /></ItemGroup></Project>"#,
            ),
        ],
    );
    let found = targets(
        &root,
        "src/app/app.csproj",
        &env(&root, &[("FEED_PASSWORD", "p")]),
    );
    let contoso = Target {
        url: "https://contoso.example/v3/index.json".into(),
        auth: Some(basic("u", "p")),
        index: true,
    };
    assert_eq!(
        found,
        [
            ("Contoso.Core".into(), vec![(contoso.url, contoso.auth)]),
            (
                "Newtonsoft.Json".into(),
                vec![target(
                    "https://api.nuget.org/v3-flatcontainer/{lower}/index.json",
                    None
                )]
            ),
        ]
    );
}

#[test]
fn asks_dotnet_for_sources_when_no_file_names_one() {
    let root = project(
        "nuget-dotnet",
        &[(
            "a.csproj",
            r#"<Project><ItemGroup><PackageReference Include="X" Version="1.0.0" /></ItemGroup></Project>"#,
        )],
    );
    let mut env = env(&root, &[]);
    env.run = |_, program, args| {
        (program == "dotnet" && args.starts_with(&["nuget", "list", "source"])).then(|| {
            "E https://feed.example/v3/index.json\nD https://off.example/v3/index.json\n".to_owned()
        })
    };
    assert_eq!(
        targets(&root, "a.csproj", &env),
        [(
            "X".into(),
            vec![target("https://feed.example/v3/index.json", None)]
        )]
    );
}

#[test]
fn takes_setting_sources_before_dotnet_when_no_file_names_one() {
    // Arrange: upstream 1.3.0's `versionlens.dotnet.nuget.sources` and a dotnet that lists another.
    let root = project(
        "nuget-setting",
        &[(
            "a.csproj",
            r#"<Project><ItemGroup><PackageReference Include="X" Version="1.0.0" /></ItemGroup></Project>"#,
        )],
    );
    let mut env = env(&root, &[]);
    env.run = |_, _, _| Some("E https://listed.example/v3/index.json\n".to_owned());
    env.nuget = vec!["https://setting.example/v3/index.json".to_owned()];

    // Act
    let resolved = targets(&root, "a.csproj", &env);

    // Assert
    assert_eq!(
        resolved,
        [(
            "X".into(),
            vec![target("https://setting.example/v3/index.json", None)]
        )]
    );
}

#[test]
fn reads_composer_repositories_and_auth_json() {
    let root = project(
        "composer",
        &[
            (
                "composer.json",
                r#"{
  "repositories": [
    { "type": "composer", "url": "https://repo.example/", "only": ["acme/*"] },
    { "type": "vcs", "url": "https://github.com/x/y" },
    { "packagist.org": false }
  ],
  "require": { "acme/lib": "^1.0", "monolog/monolog": "^3.0" }
}"#,
            ),
            (
                "auth.json",
                r#"{ "http-basic": { "repo.example": { "username": "u", "password": "p" } } }"#,
            ),
        ],
    );
    assert_eq!(
        targets(&root, "composer.json", &env(&root, &[])),
        [
            (
                "acme/lib".into(),
                vec![target(
                    "https://repo.example/p2/{name}.json",
                    Some(&basic("u", "p"))
                )]
            ),
            ("monolog/monolog".into(), vec![]),
        ]
    );
}

#[test]
fn keys_composer_auth_by_host_and_port() {
    // Composer 2.11.2 sent auth.json's `127.0.0.1:PORT` credentials to a repository on that port,
    // and none for a bare `127.0.0.1` key.
    let repo = r#"{ "repositories": [{ "type": "composer", "url": "https://repo.example:8443/" }], "require": { "acme/lib": "^1.0" } }"#;
    let read = |key: &str| {
        let auth =
            format!(r#"{{ "http-basic": {{ "{key}": {{ "username": "u", "password": "p" }} }} }}"#);
        let root = project(
            "composer-port",
            &[("composer.json", repo), ("auth.json", &auth)],
        );
        targets(&root, "composer.json", &env(&root, &[]))[0].1[0]
            .1
            .clone()
    };
    assert_eq!(read("repo.example:8443"), Some(basic("u", "p")));
    assert_eq!(read("repo.example"), None);
}

#[test]
fn routes_python_dependencies_to_their_named_or_default_index() {
    let root = project(
        "python",
        &[
            (
                "pyproject.toml",
                r#"[project]
name = "app"
dependencies = ["httpx>=0.27", "torch>=2.0"]

[[tool.uv.index]]
name = "pytorch"
url = "https://download.pytorch.org/whl/cpu"
explicit = true

[tool.uv.sources]
torch = { index = "pytorch" }

[[tool.poetry.source]]
name = "private"
url = "https://pypi.example/simple/"
priority = "explicit"

[tool.poetry.dependencies]
internal = { version = "^1.0", source = "private" }
"#,
            ),
            (
                "reqs/requirements.txt",
                "--index-url https://u:p@mirror.example/simple\nflask==3.0.0\n",
            ),
            (
                "home/.config/pip/pip.conf",
                "[global]\nindex-url = https://conf.example/simple\n",
            ),
            ("other/requirements.txt", "flask==3.0.0\n"),
        ],
    );
    let vars = env(
        &root,
        &[
            ("POETRY_HTTP_BASIC_PRIVATE_USERNAME", "pu"),
            ("POETRY_HTTP_BASIC_PRIVATE_PASSWORD", "pp"),
        ],
    );
    assert_eq!(
        targets(&root, "pyproject.toml", &vars),
        [
            (
                "httpx".into(),
                vec![target("https://conf.example/simple/{name}/", None)]
            ),
            (
                "torch".into(),
                vec![target("https://download.pytorch.org/whl/cpu/{name}/", None)]
            ),
            (
                "internal".into(),
                vec![target(
                    "https://pypi.example/simple/{name}/",
                    Some(&basic("pu", "pp"))
                )]
            ),
        ]
    );
    assert_eq!(
        targets(&root, "reqs/requirements.txt", &vars),
        [(
            "flask".into(),
            vec![target(
                "https://mirror.example/simple/{name}/",
                Some(&basic("u", "p"))
            )]
        )]
    );
    let index = env(&root, &[("PIP_INDEX_URL", "https://pypi.org/simple")]);
    assert_eq!(
        targets(&root, "other/requirements.txt", &index),
        [("flask".into(), vec![])]
    );
}

#[test]
fn reads_pipfile_sources() {
    let root = project(
        "pipfile",
        &[(
            "Pipfile",
            "[[source]]\nname = \"pypi\"\nurl = \"https://pypi.org/simple\"\n\n[[source]]\nname = \"home\"\nurl = \"https://home.example/simple\"\n\n[packages]\nrequests = \"==2.31.0\"\nmine = { version = \"==1.0\", index = \"home\" }\n",
        )],
    );
    assert_eq!(
        targets(&root, "Pipfile", &env(&root, &[])),
        [
            ("requests".into(), vec![]),
            (
                "mine".into(),
                vec![target("https://home.example/simple/{name}/", None)]
            ),
        ]
    );
}

#[test]
fn reads_cargo_registries_replacement_and_tokens() {
    let root = project(
        "cargo",
        &[
            (
                "ws/.cargo/config.toml",
                "[registries.my]\nindex = \"sparse+https://cargo.example/index/\"\n\n[source.crates-io]\nreplace-with = \"mirror\"\n\n[source.mirror]\nregistry = \"sparse+https://mirror.example/\"\n",
            ),
            (
                "ws/app/Cargo.toml",
                "[dependencies]\nserde = \"1\"\nmine = { version = \"1\", registry = \"my\" }\n",
            ),
            (
                "home/.cargo/credentials.toml",
                "[registries.my]\ntoken = \"from-file\"\n",
            ),
            (
                "home/.cargo/config.toml",
                "[registry]\nglobal-credential-providers = [\"cargo:token\"]\n",
            ),
        ],
    );
    let found = |vars: &[(&str, &str)]| targets(&root, "ws/app/Cargo.toml", &env(&root, vars));
    assert_eq!(
        found(&[]),
        [
            (
                "serde".into(),
                vec![target("https://mirror.example/{cargo}", None)]
            ),
            (
                "mine".into(),
                vec![target(
                    "https://cargo.example/index/{cargo}",
                    Some("from-file")
                )]
            ),
        ]
    );
    assert_eq!(
        found(&[("CARGO_REGISTRIES_MY_TOKEN", "from-env")])[1],
        (
            "mine".into(),
            vec![target(
                "https://cargo.example/index/{cargo}",
                Some("from-env")
            )]
        )
    );
}

/// Cargo 1.100.0 sends a registry's token only when `cargo:token` is its credential provider: a
/// `registries.<name>.credential-provider`, else one of `registry.global-credential-providers`.
/// Without one it stops with "authenticated registries require a credential-provider to be
/// configured" and sends neither the `credentials.toml` token nor `CARGO_REGISTRIES_<NAME>_TOKEN`.
#[test]
fn sends_cargo_token_only_through_cargo_token_provider() {
    let files = |name: &str, config: &str| {
        let config =
            format!("[registries.my]\nindex = \"sparse+https://cargo.example/index/\"\n{config}");
        let root = project(
            &format!("cargo-provider-{name}"),
            &[
                ("app/.cargo/config.toml", config.as_str()),
                (
                    "app/Cargo.toml",
                    "[dependencies]\nmine = { version = \"1\", registry = \"my\" }\n",
                ),
                (
                    "home/.cargo/credentials.toml",
                    "[registries.my]\ntoken = \"from-file\"\n",
                ),
            ],
        );
        move |vars: &[(&str, &str)]| {
            let found = targets(&root, "app/Cargo.toml", &env(&root, vars));
            found[0].1[0].1.clone()
        }
    };
    let none = files("none", "");
    assert_eq!(none(&[]), None);
    assert_eq!(none(&[("CARGO_REGISTRIES_MY_TOKEN", "from-env")]), None);
    assert_eq!(
        none(&[("CARGO_REGISTRY_GLOBAL_CREDENTIAL_PROVIDERS", "cargo:token")]),
        Some("from-file".into())
    );
    assert_eq!(
        none(&[("CARGO_REGISTRIES_MY_CREDENTIAL_PROVIDER", "cargo:token")]),
        Some("from-file".into())
    );
    let own = files("own", "credential-provider = \"cargo:token\"\n");
    assert_eq!(
        own(&[("CARGO_REGISTRIES_MY_TOKEN", "from-env")]),
        Some("from-env".into())
    );
    // Providers later in the global list win, and a registry's own provider replaces the list.
    let global = files(
        "global",
        "\n[registry]\nglobal-credential-providers = [\"cargo:libsecret\", \"cargo:token\"]\n",
    );
    assert_eq!(global(&[]), Some("from-file".into()));
    let keychain = files(
        "keychain",
        "credential-provider = \"cargo:macos-keychain\"\n\n[registry]\nglobal-credential-providers = [\"cargo:token\"]\n",
    );
    assert_eq!(keychain(&[]), None);
    let alias = files(
        "alias",
        "credential-provider = \"plain\"\n\n[credential-alias]\nplain = \"cargo:token\"\n",
    );
    assert_eq!(alias(&[]), Some("from-file".into()));
}

#[test]
fn reads_maven_mirror_of_central_and_its_server() {
    let root = project(
        "maven",
        &[
            (
                "home/.m2/settings.xml",
                r"<settings>
  <servers>
    <server><id>corp</id><username>u</username><password>${env.MVN_PASSWORD}</password></server>
  </servers>
  <mirrors>
    <mirror><id>other</id><mirrorOf>snapshots</mirrorOf><url>https://other.example/</url></mirror>
    <mirror><id>corp</id><mirrorOf>external:*,!snapshots</mirrorOf><url>https://maven.example/repo/</url></mirror>
  </mirrors>
</settings>",
            ),
            (
                "pom.xml",
                "<project><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>1.0</version></dependency></dependencies></project>",
            ),
        ],
    );
    assert_eq!(
        targets(&root, "pom.xml", &env(&root, &[("MVN_PASSWORD", "p")])),
        [(
            "g:a".into(),
            vec![target(
                "https://maven.example/repo/{path}/maven-metadata.xml",
                Some(&basic("u", "p"))
            )]
        )]
    );
}

#[test]
fn substitutes_known_variables_and_keeps_unknown_ones() {
    let env = Env {
        vars: HashMap::from([("A".into(), "1".into())]),
        home: None,
        run: |_, _, _| None,
        nuget: Vec::new(),
    };
    assert_eq!(substitute("x${A}y${B}z", "${", "}", &env), "x1y${B}z");
    assert_eq!(substitute("%A%/%B%", "%", "%", &env), "1/%B%");
    assert_eq!(unbase64("cGFzcw==").as_deref(), Some("pass"));
    assert_eq!(base64(b"pass"), "cGFzcw==");
}
