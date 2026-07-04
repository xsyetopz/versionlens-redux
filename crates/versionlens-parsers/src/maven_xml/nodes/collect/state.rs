use quick_xml::events::BytesStart;
use std::ops::Range as ByteRange;

use super::super::XmlNode;

#[derive(Default)]
pub(super) struct XmlCollector {
    stack: Vec<OpenNode>,
    nodes: Vec<XmlNode>,
}

impl XmlCollector {
    pub(super) fn open_node(&mut self, event: &BytesStart<'_>, start: usize) -> Option<()> {
        let name = event_name(event)?;
        let path = child_path(&self.stack, &name);
        self.stack.push(OpenNode {
            name,
            path,
            open_start: start,
            text: String::new(),
            text_range: None,
        });
        Some(())
    }

    pub(super) fn empty_node(
        &mut self,
        event: &BytesStart<'_>,
        open_start: usize,
        close_end: usize,
    ) -> Option<()> {
        let name = event_name(event)?;
        let path = child_path(&self.stack, &name);
        self.nodes.push(XmlNode {
            name,
            path,
            open_start,
            close_end,
            text: String::new(),
            text_range: None,
        });
        Some(())
    }

    pub(super) fn append_text(&mut self, value: &str, start: usize, end: usize) {
        let Some(open) = self.stack.last_mut() else {
            return;
        };
        open.text.push_str(value);
        open.text_range = Some(match open.text_range.take() {
            Some(range) => range.start..end,
            None => start..end,
        });
    }

    pub(super) fn close_node(&mut self, end_name: &[u8], close_end: usize) -> Option<()> {
        let open = self.stack.pop()?;
        if end_name != open.name.as_bytes() {
            return None;
        }
        self.nodes.push(XmlNode {
            name: open.name,
            path: open.path,
            open_start: open.open_start,
            close_end,
            text: open.text.trim().to_owned(),
            text_range: open.text_range,
        });
        Some(())
    }

    pub(super) fn finish(self) -> Option<Vec<XmlNode>> {
        self.stack.is_empty().then_some(self.nodes)
    }
}

struct OpenNode {
    name: String,
    path: String,
    open_start: usize,
    text: String,
    text_range: Option<ByteRange<usize>>,
}

fn child_path(stack: &[OpenNode], name: &str) -> String {
    let mut parts = stack
        .iter()
        .map(|node| node.name.as_str())
        .collect::<Vec<_>>();
    parts.push(name);
    parts.join(".")
}

fn event_name(event: &BytesStart<'_>) -> Option<String> {
    std::str::from_utf8(event.name().as_ref())
        .ok()
        .map(ToOwned::to_owned)
}
