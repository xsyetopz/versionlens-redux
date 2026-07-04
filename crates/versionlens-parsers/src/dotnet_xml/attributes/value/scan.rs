use std::ops::Range as ByteRange;

mod boundary;
mod name;
mod quoted;

use boundary::{skip_attribute_boundary, skip_spaces};
use name::attribute_name_range;
use quoted::{is_attribute_quote, quoted_attribute_value_range};

pub(super) fn attr_value_range_in_tag(tag: &str, name: &str) -> Option<ByteRange<usize>> {
    let bytes = tag.as_bytes();
    let mut index = 1;
    while index < bytes.len() {
        index = skip_attribute_boundary(bytes, index);

        let (attribute_name, after_name) = attribute_name_range(bytes, index);
        index = skip_spaces(bytes, after_name);
        if index >= bytes.len() || bytes[index] != b'=' {
            continue;
        }

        index = skip_spaces(bytes, index + 1);
        let quote = *bytes.get(index)?;
        if !is_attribute_quote(quote) {
            continue;
        }

        let Some((value, after_value)) = quoted_attribute_value_range(bytes, index + 1, quote)
        else {
            return None;
        };
        index = after_value;

        if tag[attribute_name].eq_ignore_ascii_case(name) {
            return Some(value);
        }
    }

    None
}
