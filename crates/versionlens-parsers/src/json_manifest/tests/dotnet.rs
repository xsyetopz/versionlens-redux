use super::{DocumentInput, Ecosystem, parse_document};
use crate::document::test_support::extract_range;

#[test]
fn parses_dotnet_project_json_dependencies() {
    let text = r#"{
  "dependencies": {
    "Newtonsoft.Json": "13.0.1",
    "NUnit": {
      "version": "4.3.2"
    }
  },
  "frameworks": {
    "net472": {
      "dependencies": {
        "System.Text.Json": "8.0.5"
      }
    }
  },
  "runtimes": {
    "win": {
      "dependencies": {
        "runtime.win.System.IO": "4.3.0"
      }
    }
  }
}"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Dotnet);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "Newtonsoft.Json");
    assert_eq!(dependencies[0].requirement, "13.0.1");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "13.0.1"
    );
    assert_eq!(dependencies[1].name, "NUnit");
    assert_eq!(dependencies[1].requirement, "4.3.2");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "4.3.2"
    );
    assert_eq!(dependencies[2].group, "frameworks.net472.dependencies");
    assert_eq!(dependencies[2].name, "System.Text.Json");
    assert_eq!(dependencies[2].requirement, "8.0.5");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "8.0.5"
    );
    assert_eq!(dependencies[3].group, "runtimes.win.dependencies");
    assert_eq!(dependencies[3].name, "runtime.win.System.IO");
    assert_eq!(dependencies[3].requirement, "4.3.0");
}
