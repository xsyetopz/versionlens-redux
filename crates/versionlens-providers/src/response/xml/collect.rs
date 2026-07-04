use quick_xml::Reader;
use quick_xml::events::Event;

mod event;

use event::{ElementTextEvent, element_text_event};

pub(super) fn collect_element_texts<F>(
    body: &str,
    element_name: &[u8],
    mut map_text: F,
) -> Option<Vec<String>>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut reader = Reader::from_str(body);
    let mut collector = ElementTextCollector::new(element_name, &mut map_text);

    loop {
        let event = reader.read_event().ok()?;
        if collector.event_finished(event)? {
            break;
        }
    }

    Some(collector.versions)
}

struct ElementTextCollector<'a, F>
where
    F: FnMut(&str) -> Option<String>,
{
    element_name: &'a [u8],
    in_element: bool,
    versions: Vec<String>,
    map_text: &'a mut F,
}

impl<'a, F> ElementTextCollector<'a, F>
where
    F: FnMut(&str) -> Option<String>,
{
    fn new(element_name: &'a [u8], map_text: &'a mut F) -> Self {
        Self {
            element_name,
            in_element: false,
            versions: Vec::new(),
            map_text,
        }
    }

    fn event_finished(&mut self, event: Event<'_>) -> Option<bool> {
        match element_text_event(event, self.in_element, self.element_name)? {
            ElementTextEvent::Start => self.in_element = true,
            ElementTextEvent::Text(text) => self.collect_text(&text),
            ElementTextEvent::End => self.in_element = false,
            ElementTextEvent::Finished => return Some(true),
            ElementTextEvent::Ignored => {}
        }

        Some(false)
    }

    fn collect_text(&mut self, text: &str) {
        if let Some(version) = (self.map_text)(text) {
            self.versions.push(version);
        }
    }
}
