use jsonc_parser::ast::{Array, Object, Value};

use crate::manifest::path_query::PathNode;

/// A JSON value as a node of a path query. Object and array children are
/// walked by key and by index; a `*` over an array keys each element by its
/// `name` when it has one, and by its index otherwise.
#[derive(Clone, Copy)]
pub(super) enum JsonNode<'a> {
    Object(&'a Object<'a>),
    Array(&'a Array<'a>),
    Scalar,
}

impl<'a> JsonNode<'a> {
    fn of(value: &'a Value<'a>) -> Self {
        match value {
            Value::Object(object) => Self::Object(object),
            Value::Array(array) => Self::Array(array),
            _ => Self::Scalar,
        }
    }
}

impl<'a> PathNode for JsonNode<'a> {
    fn child(&self, key: &str) -> Option<Self> {
        let Self::Object(object) = *self else {
            return None;
        };
        object.get(key).map(|prop| Self::of(&prop.value))
    }

    fn children(&self) -> Vec<(String, Self)> {
        match *self {
            Self::Object(object) => object
                .properties
                .iter()
                .map(|prop| (prop.name.as_str().to_owned(), Self::of(&prop.value)))
                .collect(),
            Self::Array(array) => array
                .elements
                .iter()
                .enumerate()
                .map(|(index, element)| (array_element_key(element, index), Self::of(element)))
                .collect(),
            Self::Scalar => Vec::new(),
        }
    }
}

fn array_element_key(element: &Value<'_>, index: usize) -> String {
    match element {
        Value::Object(object) => object
            .get_string("name")
            .map(|name| name.value.to_string())
            .unwrap_or_else(|| index.to_string()),
        _ => index.to_string(),
    }
}
