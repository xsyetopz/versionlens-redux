use quick_xml::events::Event;

use super::{state::XmlCollector, text::collect_xml_text};

pub(super) fn xml_event_finished(
    event: Event<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<bool> {
    if matches!(event, Event::Eof) {
        return Some(true);
    }

    collect_xml_event(event, collector, start, end)?;
    Some(false)
}

fn collect_xml_event(
    event: Event<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<()> {
    if collect_xml_element_event(&event, collector, start, end)? {
        return Some(());
    }

    if let Event::Text(event) = event {
        return collect_xml_text(event, collector, start, end);
    }

    Some(())
}

fn collect_xml_element_event(
    event: &Event<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) -> Option<bool> {
    if let Event::Start(event) = event {
        collector.open_node(event, start)?;
        return Some(true);
    }
    if let Event::Empty(event) = event {
        collector.empty_node(event, start, end)?;
        return Some(true);
    }
    if let Event::End(event) = event {
        collector.close_node(event.name().as_ref(), end)?;
        return Some(true);
    }

    Some(false)
}
