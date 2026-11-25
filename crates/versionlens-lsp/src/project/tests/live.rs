//! Live checks that each kind of project registry configuration sends a lookup to the registry it
//! names and that registry answers: public mirrors stand in for private registries, and a local
//! server that refuses requests without the configured token checks the credentials. They need
//! the network, so they are ignored by default; `just registries-live` runs them.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

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
                "[source.crates-io]\nreplace-with = \"rsproxy\"\n\n[source.rsproxy]\nregistry = \"sparse+https://rsproxy.cn/index/\"\n",
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

/// A registry on this machine that answers an npm packument only to `Bearer s3cret`, and 401 to
/// anything else; it serves `requests` requests and returns the `Authorization` header of each.
fn token_registry(requests: usize) -> (String, std::thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        let mut seen = Vec::new();
        for stream in listener.incoming().take(requests) {
            let mut stream = stream.unwrap();
            let mut auth = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 || line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("authorization:") {
                    auth = value.trim().to_owned();
                }
            }
            let (status, body) = if auth == "bearer s3cret" {
                (
                    "200 OK",
                    r#"{"versions":{"1.0.0":{},"1.1.0":{}},"dist-tags":{"latest":"1.1.0"}}"#,
                )
            } else {
                ("401 Unauthorized", "{}")
            };
            write!(
                stream,
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            seen.push(auth);
        }
        seen
    });
    (url, server)
}

#[test]
#[ignore = "needs the network"]
fn sends_the_configured_token_from_the_environment() {
    let (url, server) = token_registry(2);
    let host = url.trim_start_matches("http:");
    let root = project(
        "live-auth",
        &[
            (
                "with/package.json",
                r#"{ "dependencies": { "@acme/ui": "1.0.0" } }"#,
            ),
            (
                "with/.npmrc",
                &format!("@acme:registry={url}\n{host}:_authToken=${{ACME_TOKEN}}\n"),
            ),
            ("with/.env", "ACME_TOKEN=s3cret\n"),
            (
                "without/package.json",
                r#"{ "dependencies": { "@acme/ui": "1.0.0" } }"#,
            ),
            ("without/.npmrc", &format!("@acme:registry={url}\n")),
        ],
    );
    let with = looked_up(&root, "with/package.json", &env(&root, &[]));
    let without = looked_up(&root, "without/package.json", &env(&root, &[]));
    let seen = server.join().unwrap();
    eprintln!("config npmrc-auth with: {with:?}\nconfig npmrc-auth without: {without:?}");
    assert_eq!(seen, ["bearer s3cret", ""]);
    assert!(matches!(with[0].2, Ok(2)), "{with:?}");
    assert!(without[0].2.is_err(), "{without:?}");
}
