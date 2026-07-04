use quick_xml::events::Event;

use super::state::DotnetXmlCollector;
use crate::dotnet_xml::DotnetEventContext;

pub(super) fn dotnet_xml_event_finished(
    context: &DotnetEventContext<'_>,
    event: Result<Event<'_>, quick_xml::Error>,
    collector: &mut DotnetXmlCollector<'_>,
) -> bool {
    let Ok(event) = event else {
        return true;
    };
    if matches!(event, Event::Eof) {
        return true;
    }

    collect_dotnet_xml_event(context, event, collector);
    false
}

fn collect_dotnet_xml_event(
    context: &DotnetEventContext<'_>,
    event: Event<'_>,
    collector: &mut DotnetXmlCollector<'_>,
) {
    if collect_dotnet_element_event(context, &event, collector) {
        return;
    }

    if let Event::Text(event) = event {
        collector.text(&event);
    }
}

fn collect_dotnet_element_event(
    context: &DotnetEventContext<'_>,
    event: &Event<'_>,
    collector: &mut DotnetXmlCollector<'_>,
) -> bool {
    if let Event::Start(event) = event {
        collector.start_tag(context, event);
        return true;
    }
    if let Event::Empty(event) = event {
        collector.empty_tag(context, event);
        return true;
    }
    if let Event::End(event) = event {
        collector.end_tag(context.text, event.name().as_ref());
        return true;
    }

    false
}
