mod event;
mod state;

use quick_xml::Reader;

use crate::model::Dependency;
use crate::positions::to_usize;

use super::{DotnetEventContext, DotnetTagSpan};
use event::dotnet_xml_event_finished;
use state::DotnetXmlCollector;

pub(super) fn collect_dotnet_xml_dependencies<'a>(
    text: &str,
    dependency_paths: Vec<&'a str>,
) -> Vec<Dependency> {
    let mut reader = Reader::from_str(text);
    let mut collector = DotnetXmlCollector::new(dependency_paths);

    loop {
        let start = to_usize(reader.buffer_position());
        let event = reader.read_event();
        let invalid_xml = event.is_err();
        let end = to_usize(reader.buffer_position());
        let context = DotnetEventContext {
            text,
            span: DotnetTagSpan { start, end },
        };
        if dotnet_xml_event_finished(&context, event, &mut collector) {
            if invalid_xml {
                return Vec::new();
            }
            break;
        }
    }

    collector.finish()
}
