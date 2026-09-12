use std::path::{Path, PathBuf};

pub fn workspace_file_uri(path: &Path) -> Option<String> {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    if !path.is_absolute() {
        return None;
    }
    let path = path.to_str()?;
    #[cfg(windows)]
    let path = windows_uri_path(path)?;
    let mut uri = String::from("file://");
    #[cfg(windows)]
    if path.as_bytes().get(1) == Some(&b':') {
        uri.push('/');
    }
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'_' | b'.' | b'~') {
            uri.push(char::from(byte));
        } else {
            uri.push('%');
            uri.push(char::from(HEX[usize::from(byte >> 4)]));
            uri.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    Some(uri)
}

#[cfg(any(windows, test))]
fn windows_uri_path(path: &str) -> Option<String> {
    let normalized = path.replace('\\', "/");
    let path = if let Some(extended) = normalized.strip_prefix("//?/") {
        if extended
            .get(..4)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("UNC/"))
        {
            return Some(extended[4..].to_owned());
        }
        if !extended
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
            || extended.as_bytes().get(1..3) != Some(b":/")
        {
            return None;
        }
        extended
    } else if normalized.starts_with("//./") {
        return None;
    } else {
        normalized.strip_prefix("//").unwrap_or(&normalized)
    };
    Some(path.to_owned())
}

pub fn workspace_path(value: &str) -> Option<PathBuf> {
    if value
        .get(..7)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("file://"))
    {
        return file_path(&value[7..]);
    }
    let has_scheme = value.split_once(':').is_some_and(|(scheme, rest)| {
        let drive = scheme.len() == 1 && rest.starts_with(['/', '\\']);
        !drive
            && scheme.starts_with(|character: char| character.is_ascii_alphabetic())
            && scheme.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '+' | '-' | '.')
            })
    });
    if value.is_empty() || has_scheme {
        return None;
    }
    Some(PathBuf::from(value))
}

fn file_path(value: &str) -> Option<PathBuf> {
    if value.contains(['?', '#']) {
        return None;
    }
    let (authority, path) = value.split_once('/')?;
    let decoded = decode_path(&format!("/{path}"))?;
    if !authority.is_empty() && !authority.eq_ignore_ascii_case("localhost") {
        #[cfg(windows)]
        {
            if authority.contains(['@', ':', '\\']) {
                return None;
            }
            return Some(PathBuf::from(format!("//{authority}{decoded}")));
        }
        #[cfg(not(windows))]
        return None;
    }
    #[cfg(windows)]
    let decoded = if decoded.as_bytes().get(2) == Some(&b':')
        && decoded
            .as_bytes()
            .get(1)
            .is_some_and(u8::is_ascii_alphabetic)
    {
        decoded[1..].to_owned()
    } else {
        decoded
    };
    Some(PathBuf::from(decoded))
}

fn decode_path(value: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(value.len());
    let mut input = value.bytes();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let high = char::from(input.next()?).to_digit(16)?;
            let low = char::from(input.next()?).to_digit(16)?;
            bytes.push(u8::try_from(high * 16 + low).ok()?);
        } else {
            bytes.push(byte);
        }
    }
    if bytes.contains(&0) {
        return None;
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests;
