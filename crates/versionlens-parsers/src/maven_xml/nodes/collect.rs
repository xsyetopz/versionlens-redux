mod event;
mod state;
mod text;

use quick_xml::Reader;

use crate::positions::to_usize;

use super::XmlNode;
use event::xml_event_finished;
use state::XmlCollector;

pub(in crate::maven_xml) fn collect_nodes(text: &str) -> Option<Vec<XmlNode>> {
    let mut reader = Reader::from_str(text);
    let mut collector = XmlCollector::default();

    loop {
        let start = to_usize(reader.buffer_position());
        let event = reader.read_event().ok()?;
        let end = to_usize(reader.buffer_position());
        if xml_event_finished(event, &mut collector, start, end)? {
            break;
        }
    }

    collector.finish()
}
