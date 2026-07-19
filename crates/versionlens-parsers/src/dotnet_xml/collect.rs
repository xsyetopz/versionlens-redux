mod event;
mod state;

use crate::positions::to_usize;
use versionlens_model::Dependency;

use super::{DotnetEventContext, DotnetTagSpan};
use event::dotnet_xml_event_finished;
use state::dotnet_xml_collector;

pub(super) fn collect_dotnet_xml_dependencies<'a>(
    text: &str,
    dependency_paths: Vec<&'a str>,
) -> Vec<Dependency> {
    let mut reader = crate::xml_reader(text);
    let mut collector = dotnet_xml_collector(dependency_paths);

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
                return vec![];
            }
            break;
        }
    }

    collector.finish()
}
