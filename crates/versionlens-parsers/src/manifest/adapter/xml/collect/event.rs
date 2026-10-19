use quick_xml::events::Event;
use quick_xml::events::Event::{
    Empty as XmlEventEmpty, End as XmlEventEnd, Eof as XmlEventEof, Start as XmlEventStart,
    Text as XmlEventText,
};

use super::{state::XmlCollector, text::collect_xml_text};

type XmlEvent<'a> = Event<'a>;

pub(super) fn xml_event_finished(
    event: XmlEvent<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<bool> {
    if matches!(event, XmlEventEof) {
        return Some(true);
    }

    collect_xml_event(event, collector, start, end)?;
    Some(false)
}

fn collect_xml_event(
    event: XmlEvent<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<()> {
    if collect_xml_element_event(&event, collector, start, end)? {
        return Some(());
    }

    if let XmlEventText(event) = event {
        collect_xml_text(event, collector, start, end);
    }

    Some(())
}

fn collect_xml_element_event(
    event: &Event<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<bool> {
    if let XmlEventStart(event) = event {
        collector.open_node(event, start);
        return Some(true);
    }
    if let XmlEventEmpty(event) = event {
        collector.empty_node(event, start, end);
        return Some(true);
    }
    if let XmlEventEnd(event) = event {
        collector.close_node(event.name().as_ref(), end)?;
        return Some(true);
    }

    Some(false)
}
