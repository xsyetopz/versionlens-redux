use quick_xml::events::Event;

pub(super) enum ElementTextEvent {
    Start,
    Text(String),
    End,
    Finished,
    Ignored,
}

pub(super) fn element_text_event(
    event: Event<'_>,
    in_element: bool,
    element_name: &[u8],
) -> Option<ElementTextEvent> {
    match event {
        Event::Start(event) if event.name().as_ref() == element_name => {
            Some(ElementTextEvent::Start)
        }
        Event::Text(event) if in_element => {
            Some(ElementTextEvent::Text(event.decode().ok()?.into_owned()))
        }
        Event::End(event) if event.name().as_ref() == element_name => Some(ElementTextEvent::End),
        Event::Eof => Some(ElementTextEvent::Finished),
        _ => Some(ElementTextEvent::Ignored),
    }
}
