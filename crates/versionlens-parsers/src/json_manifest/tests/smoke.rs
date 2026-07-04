use super::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_smoke_pnpm_package_json_smoke_shapes() {
    let text = r#"{
  "packageManager": "pnpm@10.34.4",
  "dependencies": {
    "astro": "workspace:*",
    "something": "catalog:*",
    "overrides": "link:../overrides",
    "@types/react": "npm:types-react"
  },
  "pnpm": {
    "overrides": {
      "semver": "7.8.5",
      "axios@<1": "1.18.1",
      "somepackage": {
        "typescript": "6.0.3"
      }
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "packageManager");
    assert_eq!(dependencies[0].name, "pnpm");
    assert_eq!(dependencies[0].requirement, "10.34.4");
    assert_eq!(
        dependencies[1].requirement,
        "file:../overrides/package.json"
    );
    assert_eq!(dependencies[2].name, "types-react");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(dependencies[2].requirement_prefix, "npm:types-react@");
    assert_eq!(dependencies[3].group, "pnpm.overrides");
    assert_eq!(dependencies[4].name, "axios");
    assert_eq!(dependencies[5].name, "typescript");
}

#[test]
fn parses_smoke_npm_custom_file_smoke_shapes() {
    let text = r#"{
  "name": "smoke npm file registration",
  "dependencies": {},
  "devDependencies": {
    "typescript": "^6.0.3"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/web-module.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "devDependencies");
    assert_eq!(dependencies[0].name, "typescript");
    assert_eq!(dependencies[0].requirement, "^6.0.3");
}

#[test]
fn parses_smoke_npm_git_smoke_shapes() {
    let text = r#"{
  "name": "smoke-test",
  "title": "bun git smoke test",
  "dependencies": {},
  "devDependencies": {
    "gitpkgnotfound1": "git+https://git@github.com/testuser/test.git",
    "gitpkgnotfound2": "git+ssh://git@some.com/testuser/test.git"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[0].group, "devDependencies");
    assert_eq!(dependencies[0].name, "gitpkgnotfound1");
    assert_eq!(
        dependencies[0].requirement,
        "git+https://git@github.com/testuser/test.git"
    );
    assert_eq!(dependencies[1].name, "gitpkgnotfound2");
    assert_eq!(
        dependencies[1].requirement,
        "git+ssh://git@some.com/testuser/test.git"
    );
}

#[test]
fn parses_smoke_npm_faq_and_npmrc_smoke_shapes() {
    let faq = r#"{
  "devDependencies": {
    "projectz": "4.2.0",
    "@types/node": "26.0.1",
    "typescript": "^6.0.3"
  }
}"#;
    let faq_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: faq.to_owned(),
        workspace_root: None,
    });

    assert_eq!(faq_dependencies.len(), 3);
    assert_eq!(faq_dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(faq_dependencies[0].group, "devDependencies");
    assert_eq!(faq_dependencies[0].name, "projectz");
    assert_eq!(faq_dependencies[0].requirement, "4.2.0");
    assert_eq!(faq_dependencies[1].name, "@types/node");
    assert_eq!(faq_dependencies[1].requirement, "26.0.1");
    assert_eq!(faq_dependencies[2].name, "typescript");
    assert_eq!(faq_dependencies[2].requirement, "^6.0.3");

    let npmrc = r#"{
  "name": "smoke-test",
  "title": "smoke test",
  "dependencies": {},
  "devDependencies": {
    "@scope/some-package": "0.1"
  }
}"#;
    let npmrc_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: npmrc.to_owned(),
        workspace_root: None,
    });

    assert_eq!(npmrc_dependencies.len(), 1);
    assert_eq!(npmrc_dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(npmrc_dependencies[0].group, "devDependencies");
    assert_eq!(npmrc_dependencies[0].name, "@scope/some-package");
    assert_eq!(npmrc_dependencies[0].requirement, "0.1");
}

#[test]
fn parses_smoke_dub_smoke_shapes() {
    let text = r#"{
  "name": "sharex-lite",
  "description": "A minimal D application.",
  "dependencies": {
    "gtk-d:gtkd": "~>3.11.0",
    "imageformats": "~>7.0.2",
    "derelict-sdl2": "~>2.1.4",
    "luad": "~master",
    "painlessjson": {
      "version": "1.4.0"
    },
    "eventsystem": "~>2.0.0",
    "i18n": "~>0.1.0",
    "standardpaths": "~>0.8.3"
  },
  "subPackages": [
    "./modules/selector",
    {
      "standardpaths": "~>0.2.1"
    }
  ]
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dub);
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "~>3.11.0");
    assert_eq!(dependencies[4].name, "painlessjson");
    assert_eq!(dependencies[4].requirement, "1.4.0");
}

#[test]
fn parses_smoke_dub_selections_smoke_shapes() {
    let text = r#"{
  "fileVersion": 1,
  "versions": {
    "gtk-d:gtkd": "3.11.0",
    "imageformats": "7.0.2",
    "derelict-sdl2": "2.1.4"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.selections.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dub);
    assert_eq!(dependencies[0].group, "versions");
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "3.11.0");
    assert_eq!(dependencies[2].name, "derelict-sdl2");
    assert_eq!(dependencies[2].requirement, "2.1.4");
}
