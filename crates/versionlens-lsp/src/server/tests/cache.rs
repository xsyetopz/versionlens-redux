use std::os::unix::fs::PermissionsExt;

use super::*;

#[test]
fn an_unwritable_cache_directory_falls_back_to_memory_with_one_warning() -> Result<()> {
    let parent = versionlens_test_support::temporary_directory("versionlens-lsp-read-only-cache")?;
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o555))?;
    let directory = parent.join("cache");
    let (server, client) = Connection::memory();
    let server_thread = thread::spawn(move || run_connection(&server, Some(Ok(directory))));

    let result = (|| {
        initialize_workspace(&client, json!({"capabilities": {}}))?;
        send_request(&client, 2, "shutdown", Value::Null)?;
        let mut warnings = Vec::new();
        loop {
            match client.receiver.recv_timeout(Duration::from_secs(10))? {
                Message::Notification(notification)
                    if notification.method == "window/logMessage"
                        && notification.params["type"] == 2 =>
                {
                    warnings.push(notification.params["message"].clone());
                }
                Message::Response(response) if response.id == 2.into() => break,
                _ => {}
            }
        }
        send_notification(&client, "exit", Value::Null)?;
        server_thread
            .join()
            .map_err(|_| anyhow::anyhow!("server panicked"))??;
        anyhow::ensure!(warnings.len() == 1, "warnings: {warnings:?}");
        let warning = warnings[0].as_str().unwrap_or_default();
        anyhow::ensure!(
            warning.contains("persistent cache is disabled"),
            "warning: {warning}"
        );
        Ok(())
    })();

    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755))?;
    std::fs::remove_dir_all(&parent)?;
    result
}
