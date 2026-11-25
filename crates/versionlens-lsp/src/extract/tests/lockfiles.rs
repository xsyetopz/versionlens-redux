//! Lockfiles, read as the version each package resolved to.

use super::{deps, pairs};

#[test]
fn reads_package_lock_versions() {
    let text = r#"{
  "name": "app",
  "lockfileVersion": 3,
  "packages": {
    "": { "name": "app", "version": "1.0.0", "dependencies": { "lodash": "^4.17.0" } },
    "node_modules/@scope/pkg": { "version": "2.0.0", "resolved": "https://registry.npmjs.org/x" },
    "node_modules/a/node_modules/lodash": { "version": "3.10.1" },
    "node_modules/local": { "resolved": "packages/local", "link": true },
    "packages/local": { "name": "local", "version": "0.1.0" }
  },
  "dependencies": {
    "lodash": {
      "version": "4.17.21"
    }
  }
}"#;
    assert_eq!(
        deps("package-lock.json", text),
        pairs(&[
            ("@scope/pkg", "2.0.0"),
            ("lodash", "3.10.1"),
            ("lodash", "4.17.21")
        ])
    );
}

#[test]
fn reads_yarn_lock_versions() {
    let classic = "# yarn lockfile v1\n\n\"@babel/code-frame@7.0.0-beta.40\", \"@babel/code-frame@^7.0.0\":\n  version \"7.0.0\"\n  resolved \"https://registry.yarnpkg.com/x\"\n\nlodash@^4.17.0:\n  version \"4.17.21\"\n\nlocal@file:../local:\n  version \"0.1.0\"\n";
    assert_eq!(
        deps("yarn.lock", classic),
        pairs(&[("@babel/code-frame", "7.0.0"), ("lodash", "4.17.21")])
    );
    let berry = "__metadata:\n  version: 8\n\n\"lodash@npm:^4.17.0\":\n  version: 4.17.21\n  resolution: \"lodash@npm:4.17.21\"\n\n\"app@workspace:.\":\n  version: 0.0.0-use.local\n\n\"alias@npm:other@^1.0.0\":\n  version: 1.2.0\n";
    assert_eq!(deps("yarn.lock", berry), pairs(&[("lodash", "4.17.21")]));
}

#[test]
fn reads_pnpm_lock_importers() {
    let text = "lockfileVersion: '9.0'\n\nimporters:\n\n  .:\n    devDependencies:\n      '@types/node':\n        specifier: ^20.0.0\n        version: 20.11.5\n      vite:\n        specifier: workspace:*\n        version: link:packages/vite\n      react-dom:\n        specifier: ^18.2.0\n        version: 18.2.0(react@18.2.0)\n\npackages:\n\n  '@types/node@20.11.5':\n    resolution: {integrity: sha512-x}\n";
    assert_eq!(
        deps("pnpm-lock.yaml", text),
        pairs(&[("@types/node", "20.11.5"), ("react-dom", "18.2.0")])
    );
}

#[test]
fn reads_bun_lock_packages() {
    let text = "{\n  \"lockfileVersion\": 1,\n  \"workspaces\": {\n    \"\": { \"name\": \"app\", \"dependencies\": { \"lodash\": \"^4.17.0\" } },\n  },\n  \"packages\": {\n    \"@types/node\": [\"@types/node@20.11.5\", \"\", {}, \"sha512-x\"],\n    \"a/lodash\": [\"lodash@3.10.1\", \"\", {}, \"sha512-y\"],\n    \"app\": [\"app@workspace:.\"],\n  }\n}\n";
    assert_eq!(
        deps("bun.lock", text),
        pairs(&[("@types/node", "20.11.5"), ("lodash", "3.10.1")])
    );
}

#[test]
fn reads_composer_lock_packages() {
    let text = r#"{"packages": [{"name": "monolog/monolog", "version": "3.5.0"}], "packages-dev": [{"name": "phpunit/phpunit", "version": "10.5.9"}]}"#;
    assert_eq!(
        deps("composer.lock", text),
        pairs(&[("monolog/monolog", "3.5.0"), ("phpunit/phpunit", "10.5.9")])
    );
}

#[test]
fn reads_cargo_lock_registry_crates() {
    let text = "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\ndependencies = [\n \"serde\",\n]\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.200\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"x\"\n\n[[package]]\nname = \"forked\"\nversion = \"0.2.0\"\nsource = \"git+https://github.com/o/forked#abc\"\n\n[[package]]\nname = \"sparse\"\nversion = \"0.3.0\"\nsource = \"sparse+https://index.crates.io/\"\n";
    assert_eq!(
        deps("Cargo.lock", text),
        pairs(&[("serde", "1.0.200"), ("sparse", "0.3.0")])
    );
}

#[test]
fn reads_python_lockfiles() {
    let poetry = "[[package]]\nname = \"anyio\"\nversion = \"4.4.0\"\ndescription = \"x\"\noptional = false\n\n[[package]]\nname = \"certifi\"\nversion = \"2024.2.2\"\n\n[metadata]\nlock-version = \"2.0\"\n";
    assert_eq!(
        deps("poetry.lock", poetry),
        pairs(&[("anyio", "4.4.0"), ("certifi", "2024.2.2")])
    );
    let uv = "version = 1\n\n[[package]]\nname = \"anyio\"\nversion = \"4.4.0\"\nsource = { registry = \"https://pypi.org/simple\" }\n\n[[package]]\nname = \"app\"\nversion = \"0.1.0\"\nsource = { editable = \".\" }\n";
    assert_eq!(deps("uv.lock", uv), pairs(&[("anyio", "4.4.0")]));
}

#[test]
fn reads_gemfile_lock_specs() {
    let text = "GEM\n  remote: https://rubygems.org/\n  specs:\n    actionpack (7.1.3)\n      rack (>= 2.2.4)\n    nokogiri (1.16.2-x86_64-linux)\n      racc (~> 1.4)\n\nPLATFORMS\n  x86_64-linux\n\nDEPENDENCIES\n  rails (~> 7.1)\n\nBUNDLED WITH\n   2.5.3\n";
    assert_eq!(
        deps("Gemfile.lock", text),
        pairs(&[("actionpack", "7.1.3"), ("nokogiri", "1.16.2")])
    );
}
