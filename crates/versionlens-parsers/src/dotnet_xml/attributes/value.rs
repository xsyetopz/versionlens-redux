use std::ops::Range as ByteRange;

mod scan;

use scan::attr_value_range_in_tag;

pub(in crate::dotnet_xml) struct AttrValue {
    pub(in crate::dotnet_xml) value: String,
    pub(in crate::dotnet_xml) len: usize,
    pub(in crate::dotnet_xml) range: ByteRange<usize>,
}

pub(in crate::dotnet_xml) fn attr_value(tag: &str, name: &str) -> Option<AttrValue> {
    attr_value_range_in_tag(tag, name).map(|range| attr_value_from_range(tag, range))
}

fn attr_value_from_range(tag: &str, range: ByteRange<usize>) -> AttrValue {
    AttrValue {
        value: tag[range.start..range.end].to_owned(),
        len: range.end - range.start,
        range,
    }
}
