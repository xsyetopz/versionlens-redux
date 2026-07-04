pub(in crate::gemfile) fn strip_comment(input: &str) -> &str {
    let mut quote = None;

    for (index, byte) in input.bytes().enumerate() {
        if byte == b'#' && quote.is_none() {
            return &input[..index];
        }

        update_comment_quote(byte, &mut quote);
    }

    input
}

fn update_comment_quote(byte: u8, quote: &mut Option<u8>) {
    match (*quote, byte) {
        (None, b'\'' | b'"') => *quote = Some(byte),
        (Some(open), current) if current == open => *quote = None,
        _ => {}
    }
}
