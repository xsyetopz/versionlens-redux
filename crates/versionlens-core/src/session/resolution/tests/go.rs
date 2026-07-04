use super::{DocumentInput, standard_session};
use crate::registry::RegistryContext;

#[test]
fn go_mod_uses_workspace_go_proxy_urls() {
    let root = std::env::temp_dir().join(format!("versionlens-goproxy-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".env"),
        "GOPROXY=https://proxy.example.test/,direct|https://fallback.example.test|off\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: "module example.test/app\n\nrequire Go.uber.org/Zap v1.27.0\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://proxy.example.test/go.uber.org/zap/@v/list",
            "https://fallback.example.test/go.uber.org/zap/@v/list",
        ]
    );

    std::fs::remove_dir_all(root).unwrap();
}
