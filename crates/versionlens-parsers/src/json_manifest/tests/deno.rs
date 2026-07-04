use super::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;

#[test]
fn parses_deno_json_imports() {
    let text = r#"{
	    "imports": {
	    "@std/assert": "jsr:@std/assert@^1.0.0",
	    "luca": "jsr:@luca/cases@1.0.0",
	    "chalk": "npm:chalk@5.3.0",
	    "emptyJsr": "jsr:@std/assert@",
	    "emptyNpm": "npm:chalk@",
	    "url": "https://deno.land/std/mod.ts"
	  },
	  "scopes": {
	    "https://deno.land/x/app/": {
	      "@scope/pkg": "jsr:@scope/pkg@0.2.0"
	    },
	    "https://deno.land/x/other/": {
	      "chalk": "npm:chalk@5.4.0"
	    }
	  }
	}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "jsr:@std/assert@^1.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert@^1.0.0"
    );
    assert_eq!(dependencies[1].name, "luca");
    assert_eq!(dependencies[1].requirement, "jsr:@luca/cases@1.0.0");
    assert_eq!(dependencies[1].hosted_name.as_deref(), Some("@luca/cases"));
    assert_eq!(dependencies[2].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[2].name, "chalk");
    assert_eq!(dependencies[2].requirement, "npm:chalk@5.3.0");
    assert_eq!(dependencies[2].hosted_name.as_deref(), Some("chalk"));
    assert_eq!(dependencies[3].name, "emptyJsr");
    assert_eq!(dependencies[3].requirement, "jsr:@std/assert@");
    assert_eq!(dependencies[4].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[4].name, "emptyNpm");
    assert_eq!(dependencies[4].requirement, "npm:chalk@");
    assert_eq!(dependencies[5].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[5].name, "url");
    assert_eq!(dependencies[5].requirement, "https://deno.land/std/mod.ts");
}

#[test]
fn deno_imports_preserve_import_specifiers_like_upstream_npm_parser() {
    let text = r#"{
  "imports": {
    "@std/assert": "jsr:@std/assert@^1.0.0",
    "chalk": "npm:chalk@5.3.0",
    "url": "https://deno.land/std/mod.ts"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "jsr:@std/assert@^1.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert@^1.0.0"
    );
    assert_eq!(dependencies[1].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[1].name, "chalk");
    assert_eq!(dependencies[1].requirement, "npm:chalk@5.3.0");
    assert_eq!(dependencies[2].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[2].name, "url");
    assert_eq!(dependencies[2].requirement, "https://deno.land/std/mod.ts");
}

#[test]
fn parses_configured_deno_scopes() {
    let text = r#"{
	    "imports": {
	    "@std/assert": "jsr:@std/assert@^1.0.0"
	  },
	  "scopes": {
	    "https://deno.land/x/app/": {
	      "@scope/pkg": "jsr:@scope/pkg@0.2.0"
	    },
	    "https://deno.land/x/other/": {
	      "chalk": "npm:chalk@5.4.0"
	    }
	  }
	}"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["imports", "scopes"],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "jsr:@std/assert@^1.0.0");
    assert_eq!(dependencies[1].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[1].group, "scopes.https://deno.land/x/app/");
    assert_eq!(dependencies[1].name, "@scope/pkg");
    assert_eq!(dependencies[1].requirement, "jsr:@scope/pkg@0.2.0");
    assert_eq!(dependencies[2].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[2].group, "scopes.https://deno.land/x/other/");
    assert_eq!(dependencies[2].name, "chalk");
    assert_eq!(dependencies[2].requirement, "npm:chalk@5.4.0");
}

#[test]
fn parses_smoke_deno_smoke_shapes() {
    let text = r#"{
  "imports": {
    "@std/assert": "jsr:@std/assert@1.0.19",
    "luca": "jsr:@luca/cases@1.0.0",
    "cowsay": "npm:cowsay@1.6.0",
    "cases": "https://deno.land/x/case/mod.ts"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/deno.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "jsr:@std/assert@1.0.19");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert@1.0.19"
    );
    assert_eq!(dependencies[1].name, "luca");
    assert_eq!(dependencies[1].requirement, "jsr:@luca/cases@1.0.0");
    assert_eq!(dependencies[2].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[2].name, "cowsay");
    assert_eq!(dependencies[2].requirement, "npm:cowsay@1.6.0");
    assert_eq!(dependencies[3].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[3].name, "cases");
    assert_eq!(
        dependencies[3].requirement,
        "https://deno.land/x/case/mod.ts"
    );
}

#[test]
fn parses_unversioned_deno_imports_as_full_import_specifiers() {
    let text = r#"{
  "imports": {
    "@std/assert": "jsr:@std/assert",
    "chalk": "npm:chalk"
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Deno);
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "jsr:@std/assert");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert"
    );
    assert_eq!(dependencies[1].ecosystem, Ecosystem::Npm);
    assert_eq!(dependencies[1].name, "chalk");
    assert_eq!(dependencies[1].requirement, "npm:chalk");
}
