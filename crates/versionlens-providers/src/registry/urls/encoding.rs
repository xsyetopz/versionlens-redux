pub(super) fn encode_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                [byte as char, '\0', '\0']
            }
            _ => {
                const HEX: &[u8; 16] = b"0123456789ABCDEF";
                [
                    '%',
                    HEX[(byte >> 4) as usize] as char,
                    HEX[(byte & 0x0F) as usize] as char,
                ]
            }
        })
        .filter(|char| *char != '\0')
        .collect()
}
