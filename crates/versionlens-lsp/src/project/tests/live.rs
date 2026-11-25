//! Live checks that each kind of project registry configuration sends a lookup to the registry it
//! names and that registry answers: public mirrors stand in for private registries, and a local
//! server that refuses requests without the configured token checks the credentials. They need
//! the network, so they are ignored by default; `just registries-live` runs them.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use super::*;
use crate::config::Settings;
use crate::http::Http;

/// The versions each dependency of the manifest at `path` gets through its configured targets,
/// with the first target's URL, or the error the lookup ended with.
fn looked_up(root: &Path, path: &str, env: &Env) -> Vec<(String, String, Result<usize, String>)> {
    let path = root.join(path);
    let text = std::fs::read_to_string(&path).unwrap();
    let file = file_type_of(path.to_str().unwrap(), &text).unwrap();
    let mut deps = extract(file, &text);
    resolve(&path, &text, &mut deps, env);
    // The checks ask whether each route answers, not how fast, so the client waits 60s.
    let http = Http::new(Settings {
        timeout: std::time::Duration::from_secs(60),
        ..Settings::default()
    })
    .unwrap();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    deps.into_iter()
        .filter(|d| d.registry != crate::extract::PROJECT)
        .map(|d| {
            let registry = crate::registry::get(d.registry).unwrap();
            let url = d
                .targets
                .first()
                .map_or_else(String::new, |t| t.url.clone());
            let found = runtime.block_on(http.versions(registry, &d.name, None, &[], &d.targets));
            (d.name, url, found.map(|v| v.list.len()))
        })
        .collect()
}

/// Asserts each dependency went to a configured target and got versions, and prints what it got.
fn assert_answered(kind: &str, found: &[(String, String, Result<usize, String>)]) {
    for (name, url, result) in found {
        eprintln!("config {kind} {name} via {url}: {result:?}");
    }
    let failed: Vec<_> = found
        .iter()
        .filter(|(_, url, result)| url.is_empty() || !matches!(result, Ok(n) if *n > 0))
        .collect();
    assert!(failed.is_empty(), "{kind}: {failed:#?}");
}

#[test]
#[ignore = "needs the network"]
fn each_configured_registry_answers() {
    let root = project(
        "live",
        &[
            (
                "npm/package.json",
                r#"{ "dependencies": { "lodash": "4.17.21" } }"#,
            ),
            ("npm/.npmrc", "registry=https://registry.npmmirror.com/\n"),
            (
                "nuget/nuget.config",
                r#"<configuration><packageSources><clear /><add key="dotnet-public" value="https://pkgs.dev.azure.com/dnceng/public/_packaging/dotnet-public/nuget/v3/index.json" /></packageSources></configuration>"#,
            ),
            (
                "nuget/app.csproj",
                r#"<Project><ItemGroup><PackageReference Include="Newtonsoft.Json" Version="13.0.1" /></ItemGroup></Project>"#,
            ),
            (
                "composer/composer.json",
                r#"{ "repositories": [{ "type": "composer", "url": "https://mirrors.aliyun.com/composer/" }, { "packagist.org": false }], "require": { "monolog/monolog": "^3.0" } }"#,
            ),
            (
                "uv/pyproject.toml",
                "[project]\nname = \"app\"\ndependencies = [\"torch>=2.0\"]\n\n[[tool.uv.index]]\nname = \"pytorch\"\nurl = \"https://download.pytorch.org/whl/cpu\"\nexplicit = true\n\n[tool.uv.sources]\ntorch = { index = \"pytorch\" }\n",
            ),
            (
                "pip/requirements.txt",
                "--index-url https://download.pytorch.org/whl/cpu\ntorch==2.4.0\n",
            ),
            (
                "pipenv/Pipfile",
                "[[source]]\nname = \"pytorch\"\nurl = \"https://download.pytorch.org/whl/cpu\"\n\n[packages]\ntorch = { version = \"==2.4.0\", index = \"pytorch\" }\n",
            ),
            ("conf/requirements.txt", "torch==2.4.0\n"),
            (
                "home/.config/pip/pip.conf",
                "[global]\nindex-url = https://download.pytorch.org/whl/cpu\n",
            ),
            (
                "cargo/.cargo/config.toml",
                "[source.crates-io]\nreplace-with = \"tuna\"\n\n[source.tuna]\nregistry = \"sparse+https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/\"\n",
            ),
            ("cargo/Cargo.toml", "[dependencies]\nserde = \"1\"\n"),
            (
                "maven/pom.xml",
                "<project><dependencies><dependency><groupId>org.slf4j</groupId><artifactId>slf4j-api</artifactId><version>2.0.9</version></dependency></dependencies></project>",
            ),
        ],
    );
    let none = env(&root, &[]);
    assert_answered("npmrc", &looked_up(&root, "npm/package.json", &none));
    assert_answered("nuget.config", &looked_up(&root, "nuget/app.csproj", &none));
    assert_answered(
        "composer",
        &looked_up(&root, "composer/composer.json", &none),
    );
    assert_answered("uv", &looked_up(&root, "uv/pyproject.toml", &none));
    assert_answered(
        "requirements",
        &looked_up(&root, "pip/requirements.txt", &none),
    );
    assert_answered("pipfile", &looked_up(&root, "pipenv/Pipfile", &none));
    assert_answered(
        "pip.conf",
        &looked_up(&root, "conf/requirements.txt", &none),
    );
    assert_answered("cargo", &looked_up(&root, "cargo/Cargo.toml", &none));
    // The mirror of Central lives in the user's settings.xml, which only this root's home holds.
    std::fs::create_dir_all(root.join("home/.m2")).unwrap();
    std::fs::write(
        root.join("home/.m2/settings.xml"),
        "<settings><mirrors><mirror><id>gcs</id><mirrorOf>central</mirrorOf><url>https://maven-central.storage-download.googleapis.com/maven2/</url></mirror></mirrors></settings>",
    )
    .unwrap();
    assert_answered("settings.xml", &looked_up(&root, "maven/pom.xml", &none));
}

/// A registry's answers: the path suffix each body answers, with `{url}` naming the server.
type Routes = Vec<(&'static str, String)>;

/// Each request a registry saw: its path and its `Authorization` header.
type Seen = Arc<Mutex<Vec<(String, String)>>>;

/// A registry on this machine that answers only requests carrying `auth`, with the body of the
/// first route whose suffix ends the path, and 401 otherwise, as a private registry does. It runs
/// until the test ends.
fn token_registry(auth: &'static str, routes: Routes) -> (String, Seen) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&seen);
    let routes: Vec<_> = routes
        .into_iter()
        .map(|(suffix, body)| (suffix, body.replace("{url}", &url)))
        .collect();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            reader.read_line(&mut request).unwrap();
            let path = request.split(' ').nth(1).unwrap_or_default().to_owned();
            let mut sent = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                if let Some((name, value)) = line.split_once(':')
                    && name.eq_ignore_ascii_case("authorization")
                {
                    value.trim().clone_into(&mut sent);
                }
            }
            let route = routes.iter().find(|(suffix, _)| path.ends_with(suffix));
            let (status, body) = match route {
                Some((_, body)) if sent == auth => ("200 OK", body.as_str()),
                Some(_) => ("401 Unauthorized", ""),
                None => ("404 Not Found", ""),
            };
            write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            log.lock().unwrap().push((path, sent));
        }
    });
    (url, seen)
}

#[test]
#[ignore = "needs the network"]
fn sends_the_npmrc_token_from_the_environment() {
    let packument = r#"{"versions":{"1.0.0":{},"1.1.0":{}},"dist-tags":{"latest":"1.1.0"}}"#;
    let (url, seen) = token_registry("Bearer s3cret", vec![("/@acme%2fui", packument.into())]);
    let host = url.trim_start_matches("http:");
    let root = project(
        "live-npm-auth",
        &[
            (
                "with/package.json",
                r#"{ "dependencies": { "@acme/ui": "1.0.0" } }"#,
            ),
            (
                "with/.npmrc",
                &format!("@acme:registry={url}/\n{host}/:_authToken=${{ACME_TOKEN}}\n"),
            ),
            ("with/.env", "ACME_TOKEN=s3cret\n"),
            (
                "without/package.json",
                r#"{ "dependencies": { "@acme/ui": "1.0.0" } }"#,
            ),
            ("without/.npmrc", &format!("@acme:registry={url}/\n")),
        ],
    );
    let with = looked_up(&root, "with/package.json", &env(&root, &[]));
    let without = looked_up(&root, "without/package.json", &env(&root, &[]));
    eprintln!("config npmrc-auth with: {with:?}\nconfig npmrc-auth without: {without:?}");
    assert_eq!(
        seen.lock()
            .unwrap()
            .iter()
            .map(|(_, a)| a.as_str())
            .collect::<Vec<_>>(),
        ["Bearer s3cret", ""]
    );
    assert!(matches!(with[0].2, Ok(2)), "{with:?}");
    assert!(without[0].2.is_err(), "{without:?}");
}

/// GitHub Packages gets the token `.npmrc` reads from `${GITHUB_TOKEN}`, as its docs configure it:
/// docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-npm-registry
/// Here that is the token `gh auth token` prints. Asked for `@github/catalyst`, the registry answers
/// 401 to a request without one, 403 "does not match expected scopes" to a token without
/// `read:packages` (curl, 2026-11-25), and the packument to a token with it, so a 403 or versions
/// in place of the 401 shows the token arrived.
#[test]
#[ignore = "needs the network and a signed-in gh"]
fn sends_the_npmrc_token_to_github_packages() {
    let out = std::process::Command::new("gh")
        .args(["auth", "token", "--hostname", "github.com"])
        .output()
        .unwrap();
    let token = String::from_utf8(out.stdout).unwrap().trim().to_owned();
    assert!(!token.is_empty(), "gh is not signed in");
    let manifest = r#"{ "dependencies": { "@github/catalyst": "1.0.0" } }"#;
    let root = project(
        "live-github-packages",
        &[
            ("with/package.json", manifest),
            (
                "with/.npmrc",
                "@github:registry=https://npm.pkg.github.com\n//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}\n",
            ),
            ("without/package.json", manifest),
            (
                "without/.npmrc",
                "@github:registry=https://npm.pkg.github.com\n",
            ),
        ],
    );
    let with = looked_up(
        &root,
        "with/package.json",
        &env(&root, &[("GITHUB_TOKEN", &token)]),
    );
    let without = looked_up(&root, "without/package.json", &env(&root, &[]));
    let shown = |found: &[(String, String, Result<usize, String>)]| {
        found
            .iter()
            .map(|(n, u, r)| {
                format!(
                    "{n} via {u}: {:?}",
                    r.as_ref().map_err(|e| e.replace(&token, "***"))
                )
            })
            .collect::<Vec<_>>()
    };
    eprintln!(
        "config github-packages with: {:?}\nconfig github-packages without: {:?}",
        shown(&with),
        shown(&without)
    );
    assert!(
        with[0].1.starts_with("https://npm.pkg.github.com"),
        "{:?}",
        shown(&with)
    );
    assert!(
        matches!(&without[0].2, Err(e) if e.contains("401")),
        "{:?}",
        shown(&without)
    );
    assert!(
        matches!(&with[0].2, Ok(n) if *n > 0)
            || matches!(&with[0].2, Err(e) if e.contains("HTTP 403")),
        "{:?}",
        shown(&with)
    );
}

/// The real `dotnet` CLI reports a source from a `NuGet.Config` this reader does not look for,
/// here the one under the `dotnet-home/` the CLI is given as `HOME`, and the lookup reaches it.
#[test]
#[ignore = "needs the dotnet CLI"]
fn reaches_the_feed_dotnet_lists() {
    let routes = vec![
        (
            "/v3/index.json",
            r#"{"resources":[{"@id":"{url}/flat/","@type":"PackageBaseAddress/3.0.0"}]}"#.into(),
        ),
        (
            "/flat/contoso.core/index.json",
            r#"{"versions":["1.0.0","1.1.0"]}"#.into(),
        ),
    ];
    let (url, seen) = token_registry("", routes);
    let config = format!(
        r#"<configuration><packageSources><clear /><add key="corp" value="{url}/v3/index.json" /></packageSources></configuration>"#
    );
    let root = project(
        "live-dotnet-list",
        &[
            ("dotnet-home/.nuget/NuGet/NuGet.Config", &config),
            (
                "app.csproj",
                r#"<Project><ItemGroup><PackageReference Include="Contoso.Core" Version="1.0.0" /></ItemGroup></Project>"#,
            ),
        ],
    );
    let mut env = env(&root, &[]);
    env.run = |dir, program, args| {
        let home = dir.join("dotnet-home");
        let out = std::process::Command::new(program)
            .args(args)
            .current_dir(dir)
            .env("HOME", &home)
            .env("DOTNET_CLI_HOME", &home)
            .env("DOTNET_CLI_TELEMETRY_OPTOUT", "1")
            .env("DOTNET_NOLOGO", "1")
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let found = looked_up(&root, "app.csproj", &env);
    let sent: Vec<_> = seen.lock().unwrap().clone();
    eprintln!("config dotnet-list {found:?} sent {sent:?}");
    assert!(
        matches!(found.as_slice(), [(_, target, Ok(2))] if target.starts_with(&url)),
        "{found:?} {sent:?}"
    );
}

/// What Composer 2.11.2, pip 26.2.2, `NuGet` 7.9.0 (`dotnet restore`), Maven 3.10.1, and Cargo
/// 1.100.0 sent to a capturing server for these same configurations: `Basic dTpw` (`u:p`) from
/// each password, and Cargo's token bare. Each reader's lookup must send the same and succeed.
#[test]
#[ignore = "needs the network"]
fn sends_the_header_each_package_manager_sends() {
    let basic = "Basic dTpw";
    let cases: Vec<(&str, &str, &'static str, Routes)> = vec![
        (
            "composer",
            "composer.json",
            basic,
            vec![("/p2/acme/lib.json", r#"{"packages":{"acme/lib":[{"version":"1.2.0"}]}}"#.into())],
        ),
        ("pip", "requirements.txt", basic, vec![("/simple/flask/", r#"{"versions":["3.0.0"]}"#.into())]),
        (
            "nuget",
            "app.csproj",
            basic,
            // NuGet asks the service index where package versions live, then asks there.
            vec![
                (
                    "/v3/index.json",
                    r#"{"resources":[{"@id":"{url}/flat/","@type":"PackageBaseAddress/3.0.0"}]}"#.into(),
                ),
                ("/flat/contoso.core/index.json", r#"{"versions":["1.0.0"]}"#.into()),
            ],
        ),
        (
            "maven",
            "pom.xml",
            basic,
            vec![(
                "/g/a/maven-metadata.xml",
                "<metadata><versioning><versions><version>1.0</version></versions></versioning></metadata>".into(),
            )],
        ),
        (
            "cargo",
            "Cargo.toml",
            "from-file",
            vec![("/mi/ne/mine", r#"{"name":"mine","vers":"1.0.0","deps":[],"cksum":"0","features":{},"yanked":false}"#.into())],
        ),
    ];
    for (kind, manifest, auth, routes) in cases {
        let (url, seen) = token_registry(auth, routes);
        let host = url.trim_start_matches("http://").to_owned();
        let files = auth_files(kind, &url, &host);
        let files: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, t)| (p.as_str(), t.as_str()))
            .collect();
        let root = project(&format!("live-auth-{kind}"), &files);
        let found = looked_up(&root, manifest, &env(&root, &[("MVN_PASSWORD", "p")]));
        let sent: Vec<_> = seen.lock().unwrap().clone();
        eprintln!("config {kind}-auth {found:?} sent {sent:?}");
        assert!(
            matches!(found.as_slice(), [(_, _, Ok(n))] if *n > 0),
            "{kind}: {found:?} {sent:?}"
        );
        assert!(
            !sent.is_empty() && sent.iter().all(|(_, a)| a == auth),
            "{kind}: {sent:?}"
        );
    }
}

/// The project files that point `kind`'s reader at the registry at `url` with user `u` and
/// password `p`, or Cargo's token `from-file`.
fn auth_files(kind: &str, url: &str, host: &str) -> Vec<(String, String)> {
    match kind {
        "composer" => vec![
            (
                "composer.json".into(),
                format!(r#"{{ "repositories": [{{ "type": "composer", "url": "{url}/" }}, {{ "packagist.org": false }}], "require": {{ "acme/lib": "^1.0" }} }}"#),
            ),
            (
                "auth.json".into(),
                format!(r#"{{ "http-basic": {{ "{host}": {{ "username": "u", "password": "p" }} }} }}"#),
            ),
        ],
        "pip" => vec![(
            "requirements.txt".into(),
            format!("--index-url http://u:p@{host}/simple\nflask==3.0.0\n"),
        )],
        "nuget" => vec![
            (
                "nuget.config".into(),
                format!(r#"<configuration><packageSources><clear /><add key="corp" value="{url}/v3/index.json" /></packageSources><packageSourceCredentials><corp><add key="Username" value="u" /><add key="ClearTextPassword" value="p" /></corp></packageSourceCredentials></configuration>"#),
            ),
            (
                "app.csproj".into(),
                r#"<Project><ItemGroup><PackageReference Include="Contoso.Core" Version="1.0.0" /></ItemGroup></Project>"#.into(),
            ),
        ],
        "maven" => vec![
            (
                "home/.m2/settings.xml".into(),
                format!("<settings><servers><server><id>corp</id><username>u</username><password>${{env.MVN_PASSWORD}}</password></server></servers><mirrors><mirror><id>corp</id><mirrorOf>external:*,!snapshots</mirrorOf><url>{url}/maven/</url></mirror></mirrors></settings>"),
            ),
            (
                "pom.xml".into(),
                "<project><dependencies><dependency><groupId>g</groupId><artifactId>a</artifactId><version>1.0</version></dependency></dependencies></project>".into(),
            ),
        ],
        _ => vec![
            (
                ".cargo/config.toml".into(),
                format!("[registries.my]\nindex = \"sparse+{url}/index/\"\n"),
            ),
            (
                "home/.cargo/credentials.toml".into(),
                "[registries.my]\ntoken = \"from-file\"\n".into(),
            ),
            // Cargo sends the token only through this provider, as it did to the capturing server.
            (
                "home/.cargo/config.toml".into(),
                "[registry]\nglobal-credential-providers = [\"cargo:token\"]\n".into(),
            ),
            (
                "Cargo.toml".into(),
                "[dependencies]\nmine = { version = \"1\", registry = \"my\" }\n".into(),
            ),
        ],
    }
}
