#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TokenKind {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    String,
    Keyword,
    Symbol,
}

#[derive(Clone, Copy)]
pub(crate) struct Token<'a> {
    pub(crate) kind: TokenKind,
    pub(crate) text: &'a str,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) content_start: usize,
    pub(crate) content_end: usize,
}

pub(crate) type Tokens<'a> = [Token<'a>];

pub(crate) fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = vec![];
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'(' => push_single(&mut tokens, TokenKind::LParen, text, &mut index),
            b')' => push_single(&mut tokens, TokenKind::RParen, text, &mut index),
            b'[' => push_single(&mut tokens, TokenKind::LBracket, text, &mut index),
            b']' => push_single(&mut tokens, TokenKind::RBracket, text, &mut index),
            b'{' => push_single(&mut tokens, TokenKind::LBrace, text, &mut index),
            b'}' => push_single(&mut tokens, TokenKind::RBrace, text, &mut index),
            b'"' => {
                let start = index;
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == b'"' {
                        break;
                    } else {
                        index += 1;
                    }
                }
                let content_end = index.min(bytes.len());
                let end = (index + 1).min(bytes.len());
                if let Some(value) = text.get(start + 1..content_end) {
                    tokens.push(Token {
                        kind: TokenKind::String,
                        text: value,
                        start,
                        end,
                        content_start: start + 1,
                        content_end,
                    });
                }
                index = end;
            }
            b';' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            byte if byte.is_ascii_whitespace() || byte == b',' => index += 1,
            _ => {
                let start = index;
                while index < bytes.len() && !is_delimiter(bytes[index]) {
                    index += 1;
                }
                if let Some(value) = text.get(start..index) {
                    tokens.push(Token {
                        kind: if value.starts_with(':') {
                            TokenKind::Keyword
                        } else {
                            TokenKind::Symbol
                        },
                        text: value,
                        start,
                        end: index,
                        content_start: start,
                        content_end: index,
                    });
                }
            }
        }
    }

    tokens
}

fn push_single<'a>(tokens: &mut Vec<Token<'a>>, kind: TokenKind, text: &'a str, index: &mut usize) {
    let start = *index;
    tokens.push(Token {
        kind,
        text: text.get(start..start + 1).unwrap_or_default(),
        start,
        end: start + 1,
        content_start: start,
        content_end: start + 1,
    });
    *index += 1;
}

fn is_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(
            byte,
            b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'"' | b',' | b';'
        )
}

pub(crate) fn matching(
    tokens: &Tokens<'_>,
    start: usize,
    open: TokenKind,
    close: TokenKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind == open {
            depth += 1;
        } else if token.kind == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(crate) fn visit_delimited_groups(
    tokens: &Tokens<'_>,
    open: TokenKind,
    close: TokenKind,
    mut group_at: impl FnMut(&Tokens<'_>, usize) -> Option<String>,
    mut visit: impl FnMut(String, usize, usize),
) {
    for index in 0..tokens.len() {
        let Some(group) = group_at(tokens, index) else {
            continue;
        };
        let Some(open_token) = tokens.get(index + 1).map(|token| token.kind) else {
            continue;
        };
        if open_token != open {
            continue;
        }
        let Some(end) = matching(tokens, index + 1, open_token, close) else {
            continue;
        };
        visit(group, index + 2, end);
    }
}

pub(crate) fn field_value<'a>(
    tokens: &'a Tokens<'a>,
    start: usize,
    end: usize,
    key: &str,
) -> Option<&'a str> {
    let key_index = (start..end).find(|index| tokens[*index].text == key)?;
    tokens.get(key_index + 1).map(|token| token.text)
}
