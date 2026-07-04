use toml_edit::{Item, Key, Table, Value as TomlValue};

pub(crate) fn walk_toml_values<'a>(
    table: &'a Table,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    for (key_name, item) in table.iter() {
        let Some(key) = table.key(key_name) else {
            continue;
        };

        keys.push(key);
        visit_toml_item(item, keys, visit);
        keys.pop();
    }
}

fn visit_toml_item<'a>(
    item: &'a Item,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    match item {
        Item::Value(value) => visit(keys, value),
        Item::Table(child) => walk_toml_values(child, keys, visit),
        Item::ArrayOfTables(_) | Item::None => {}
    }
}
