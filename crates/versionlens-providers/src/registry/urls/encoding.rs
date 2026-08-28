const HEX: &[u8; 16] = b"0123456789ABCDEF";

fn percent_encoded(byte: u8) -> [char; 3] {
    [
        '%',
        HEX[(byte >> 4) as usize] as char,
        HEX[(byte & 0x0F) as usize] as char,
    ]
}

pub(super) fn encode_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                [byte as char, '\0', '\0']
            }
            _ => percent_encoded(byte),
        })
        .filter(|char| *char != '\0')
        .collect()
}

pub(super) fn encode_unsafe_url_bytes(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| {
            if byte.is_ascii_graphic()
                && !matches!(
                    byte,
                    b'"' | b'<' | b'>' | b'\\' | b'^' | b'`' | b'{' | b'|' | b'}'
                )
            {
                [byte as char, '\0', '\0']
            } else {
                percent_encoded(byte)
            }
        })
        .filter(|char| *char != '\0')
        .collect()
}
