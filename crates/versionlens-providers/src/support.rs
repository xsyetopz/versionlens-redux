use quick_xml::Reader as XmlReader;
use serde_json::Value as JsonValue;
use std::path::Path as StdPath;
use std::string::FromUtf8Error as StringFromUtf8Error;

pub(crate) fn path(value: &str) -> &StdPath {
    value.as_ref()
}

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn json_bool(value: &JsonValue) -> Option<bool> {
    value.as_bool()
}

pub(crate) fn xml_reader(body: &str) -> XmlReader<&[u8]> {
    quick_xml::Reader::from_str(body)
}

pub(crate) fn string_from_utf8(value: Vec<u8>) -> Result<String, StringFromUtf8Error> {
    value.try_into()
}

pub(crate) fn json_array_mut(value: &mut JsonValue) -> Option<&mut Vec<JsonValue>> {
    value.as_array_mut()
}
