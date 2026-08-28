use self::ElementTextEvent::{
    End as ElementTextEnd, Finished as ElementTextFinished, Ignored as ElementTextIgnored,
    Start as ElementTextStart, Text as ElementTextText,
};
use quick_xml::events::Event;

mod event;

use event::{ElementTextEvent, element_text_event};

fn element_text_collector<'a, F>(
    element_name: &'a [u8],
    map_text: &'a mut F,
) -> ElementTextCollector<'a, F>
where
    F: FnMut(&str) -> Option<String>,
{
    ElementTextCollector {
        element_name,
        in_element: false,
        versions: vec![],
        map_text,
    }
}

pub(super) fn collect_element_texts<F>(
    body: &str,
    element_name: &[u8],
    mut map_text: F,
) -> Option<Vec<String>>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut reader = crate::xml_reader(body);
    let mut collector = element_text_collector(element_name, &mut map_text);

    loop {
        let event = reader.read_event().ok()?;
        if collector.event_finished(event) {
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
    fn event_finished(&mut self, event: Event<'_>) -> bool {
        match element_text_event(event, self.in_element, self.element_name) {
            ElementTextStart => self.in_element = true,
            ElementTextText(text) => self.collect_text(&text),
            ElementTextEnd => self.in_element = false,
            ElementTextFinished => return true,
            ElementTextIgnored => {}
        }

        false
    }

    fn collect_text(&mut self, text: &str) {
        if let Some(version) = (self.map_text)(text) {
            self.versions.push(version);
        }
    }
}
