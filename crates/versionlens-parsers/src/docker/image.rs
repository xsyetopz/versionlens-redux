pub(crate) struct ImageRef<'a> {
    pub(crate) registry: &'a str,
    pub(crate) name: &'a str,
    pub(crate) tag: &'a str,
    pub(crate) name_offset: usize,
    pub(crate) tag_offset: usize,
}

pub(crate) fn split_image_reference(input: &str) -> ImageRef<'_> {
    let digest_start = input.find('@').unwrap_or(input.len());
    let without_digest = &input[..digest_start];
    let registry_len = registry_len(without_digest);
    let registry = registry_len
        .checked_sub(1)
        .and_then(|end| without_digest.get(..end))
        .unwrap_or("");
    let image_with_tag = &without_digest[registry_len..];
    let name_end = image_with_tag
        .rfind(':')
        .filter(|index| !image_with_tag[..*index].contains(':'))
        .unwrap_or(image_with_tag.len());
    let tag = image_with_tag
        .get(name_end + 1..)
        .filter(|_| name_end < image_with_tag.len())
        .unwrap_or("");

    ImageRef {
        registry,
        name: &image_with_tag[..name_end],
        tag,
        name_offset: registry_len,
        tag_offset: registry_len + name_end + usize::from(!tag.is_empty()),
    }
}

fn registry_len(input: &str) -> usize {
    let Some(slash) = input.find('/') else {
        return 0;
    };
    let first = &input[..slash];
    usize::from(first.contains('.') || first.contains(':') || first == "localhost") * (slash + 1)
}
