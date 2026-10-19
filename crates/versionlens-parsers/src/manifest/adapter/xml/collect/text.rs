use quick_xml::events::BytesText;

use super::state::XmlCollector;

pub(super) fn collect_xml_text(
    event: BytesText<'_>,
    collector: &mut XmlCollector,
    start: usize,
    end: usize,
) {
    let value = event.xml10_content();
    collector.append_text(value.as_ref(), start, end);
}
