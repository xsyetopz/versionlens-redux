use super::{windows_uri_path, workspace_file_uri, workspace_path};
use std::path::PathBuf;

#[test]
fn unicode_and_reserved_path_characters_round_trip() {
    assert_eq!(
        workspace_path("file:///tmp/%E2%9C%93%20project/a%23b%25.json"),
        Some(PathBuf::from("/tmp/✓ project/a#b%.json"))
    );
    assert_eq!(
        workspace_path("file://localhost/tmp/package.json"),
        Some(PathBuf::from("/tmp/package.json"))
    );
}

#[test]
fn invalid_file_uris_have_no_filesystem_path() {
    for uri in [
        "vscode-remote://ssh/work/package.json",
        "https://example.test/package.json",
        "untitled:package.json",
        "file:///tmp/%XY",
        "file:///tmp/%00",
        "file:///tmp/part#fragment",
    ] {
        assert_eq!(workspace_path(uri), None, "{uri}");
    }
}

#[test]
fn filesystem_paths_encode_as_document_uris() {
    let root = std::env::temp_dir();
    for name in [
        "package.json",
        "✓ project/a#b%?.json",
        "ci files/dépendances.yaml",
    ] {
        let path = root.join(name);
        let uri = workspace_file_uri(&path).expect("absolute UTF-8 path");
        assert_eq!(workspace_path(&uri), Some(path));
        assert!(!uri.contains([' ', '#', '?']));
    }
    assert_eq!(
        workspace_file_uri(std::path::Path::new("relative.json")),
        None
    );
}

#[test]
fn windows_canonical_paths_use_file_uri_drive_and_authority_syntax() {
    for (path, expected) in [
        (r"C:\workspace\package.json", "C:/workspace/package.json"),
        (
            r"\\?\C:\workspace\package.json",
            "C:/workspace/package.json",
        ),
        (r"\\server\share\package.json", "server/share/package.json"),
        (
            r"\\?\UNC\server\share\package.json",
            "server/share/package.json",
        ),
        (
            r"\\?\unc\server\share\package.json",
            "server/share/package.json",
        ),
    ] {
        assert_eq!(windows_uri_path(path).as_deref(), Some(expected));
    }
    for path in [
        r"\\.\COM1",
        r"\\?\GLOBALROOT\Device\HarddiskVolume1",
        r"\\?\relative",
    ] {
        assert_eq!(windows_uri_path(path), None);
    }
}

#[cfg(windows)]
#[test]
fn windows_canonical_file_paths_round_trip_through_editor_uris() {
    for (path, expected) in [
        (
            r"\\?\C:\workspace\package.json",
            "file:///C:/workspace/package.json",
        ),
        (
            r"\\?\UNC\server\share\package.json",
            "file://server/share/package.json",
        ),
    ] {
        let uri = workspace_file_uri(std::path::Path::new(path)).unwrap();
        assert_eq!(uri, expected);
        assert!(workspace_path(&uri).unwrap().is_absolute());
    }
}
