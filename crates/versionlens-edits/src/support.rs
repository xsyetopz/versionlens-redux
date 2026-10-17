pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

#[cfg(test)]
pub(crate) mod tests;
