use std::io;
use std::thread;

use anyhow::{Context, Result};
use lsp_server::{Connection, Message};

pub(super) fn run(dispatch: impl FnOnce(&Connection) -> Result<()>) -> Result<()> {
    let (incoming, receiver) = crossbeam_channel::bounded(64);
    let (sender, outgoing) = crossbeam_channel::bounded::<Message>(64);
    let reader = thread::spawn(move || -> io::Result<()> {
        let stdin = io::stdin();
        let mut input = stdin.lock();
        while let Some(message) = Message::read(&mut input)? {
            let exiting = matches!(&message, Message::Notification(notification) if notification.method == "exit");
            if incoming.send(message).is_err() || exiting {
                break;
            }
        }
        Ok(())
    });
    let writer = thread::spawn(move || -> io::Result<()> {
        let stdout = io::stdout();
        let mut output = stdout.lock();
        for message in outgoing {
            message.write(&mut output)?;
        }
        Ok(())
    });
    let connection = Connection { sender, receiver };
    let result = dispatch(&connection);
    drop(connection);
    writer
        .join()
        .map_err(|_| anyhow::anyhow!("LSP writer panicked"))?
        .context("failed to write LSP output")?;
    // A failed dispatcher must not wait for a client holding stdin open.
    if reader.is_finished() {
        reader
            .join()
            .map_err(|_| anyhow::anyhow!("LSP reader panicked"))?
            .context("failed to read LSP input")?;
    }
    result
}
