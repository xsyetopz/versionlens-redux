use std::path::Path;

use regex_automata::meta::Regex;

/// A workspace member glob, matched one path segment at a time.
///
/// Each segment compiles to one anchored regex, so braces, classes and
/// pnpm extglobs (`@(..)`, `?(..)`, `*(..)`, `+(..)`, whole-segment `!(..)`)
/// match in linear time.
#[derive(Debug, Clone)]
pub(super) struct WorkspacePattern {
    segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
enum Segment {
    Recursive,
    Component(Component),
}

#[derive(Debug, Clone)]
struct Component {
    regex: Regex,
    negated: bool,
    has_wildcards: bool,
}

impl WorkspacePattern {
    pub(super) fn parse(value: &str) -> Option<Self> {
        let value = value.trim().trim_end_matches('/');
        if value.is_empty() || Path::new(value).is_absolute() {
            return None;
        }
        let raw_segments = value
            .split('/')
            .filter(|segment| !segment.is_empty() && *segment != ".")
            .collect::<Vec<_>>();
        if raw_segments.is_empty()
            || raw_segments
                .iter()
                .any(|segment| *segment == ".." || segment.contains('\0'))
        {
            return None;
        }
        let segments = raw_segments
            .into_iter()
            .map(|segment| {
                if segment == "**" {
                    Some(Segment::Recursive)
                } else {
                    Component::new(segment).map(Segment::Component)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self { segments })
    }

    pub(super) fn matches(&self, path: &[&str]) -> bool {
        self.match_states(path).last().copied().unwrap_or_default()
    }

    pub(super) fn could_match_descendant(&self, path: &[&str]) -> bool {
        self.match_states(path).into_iter().any(|state| state)
    }

    pub(super) fn explicitly_names(&self, name: &str) -> bool {
        self.segments.iter().any(|segment| match segment {
            Segment::Recursive => false,
            Segment::Component(component) => !component.has_wildcards && component.matches(name),
        })
    }

    /// Which pattern positions are reachable after `path`; the last one means a full match.
    fn match_states(&self, path: &[&str]) -> Vec<bool> {
        let segments = &self.segments;
        let close_recursive = |active: &mut [bool]| {
            for index in 0..segments.len() {
                if active[index] && matches!(segments[index], Segment::Recursive) {
                    active[index + 1] = true;
                }
            }
        };
        let mut active = vec![false; segments.len() + 1];
        active[0] = true;
        close_recursive(&mut active);
        for name in path {
            let mut next = vec![false; segments.len() + 1];
            for (index, segment) in segments.iter().enumerate() {
                if !active[index] {
                    continue;
                }
                match segment {
                    Segment::Recursive => next[index] = true,
                    Segment::Component(component) if component.matches(name) => {
                        next[index + 1] = true;
                    }
                    Segment::Component(_) => {}
                }
            }
            close_recursive(&mut next);
            active = next;
        }
        active
    }
}

impl Component {
    fn new(segment: &str) -> Option<Self> {
        let pattern = segment.chars().collect::<Vec<_>>();
        let negative = whole_negative_extglob(&pattern);
        let body = negative.map_or(&pattern[..], |end| &pattern[2..end]);
        if body.windows(2).any(|pair| pair == ['!', '(']) {
            return None;
        }
        let mut translation = Translation::default();
        let source = if negative.is_some() {
            translation.choices(body, '|')
        } else {
            translation.sequence(body)
        };
        Some(Self {
            regex: Regex::new(&format!("^(?s:{source})$")).ok()?,
            negated: negative.is_some(),
            has_wildcards: negative.is_some() || translation.has_wildcards,
        })
    }

    fn matches(&self, name: &str) -> bool {
        self.regex.is_match(name) != self.negated
    }
}

/// The closing parenthesis of a segment that is one `!(..)` group.
fn whole_negative_extglob(pattern: &[char]) -> Option<usize> {
    if !pattern.starts_with(&['!', '(']) {
        return None;
    }
    let end = matching(pattern, 1, '(', ')')?;
    (end + 1 == pattern.len()).then_some(end)
}

#[derive(Default)]
struct Translation {
    has_wildcards: bool,
}

impl Translation {
    fn sequence(&mut self, pattern: &[char]) -> String {
        let mut source = String::new();
        let mut index = 0;
        while index < pattern.len() {
            let character = pattern[index];
            let extglob =
                matches!(character, '@' | '*' | '+' | '?') && pattern.get(index + 1) == Some(&'(');
            if let Some(end) = extglob
                .then(|| matching(pattern, index + 1, '(', ')'))
                .flatten()
            {
                source.push_str(&self.choices(&pattern[index + 2..end], '|'));
                if character != '@' {
                    self.has_wildcards = true;
                    source.push(character);
                }
                index = end + 1;
                continue;
            }
            match character {
                '*' => {
                    self.has_wildcards = true;
                    source.push_str(".*");
                }
                '?' => {
                    self.has_wildcards = true;
                    source.push('.');
                }
                '[' if let Some(end) = pattern[index + 1..]
                    .iter()
                    .position(|character| *character == ']')
                    .map(|end| index + 1 + end) =>
                {
                    self.has_wildcards = true;
                    source.push_str(&character_class(&pattern[index + 1..end]));
                    index = end;
                }
                '{' if let Some(end) = matching(pattern, index, '{', '}')
                    && split_choices(&pattern[index + 1..end], ',').len() > 1 =>
                {
                    source.push_str(&self.choices(&pattern[index + 1..end], ','));
                    index = end;
                }
                character => {
                    regex_syntax::escape_into(character.encode_utf8(&mut [0; 4]), &mut source);
                }
            }
            index += 1;
        }
        source
    }

    fn choices(&mut self, pattern: &[char], separator: char) -> String {
        let choices = split_choices(pattern, separator)
            .into_iter()
            .map(|choice| self.sequence(choice))
            .collect::<Vec<_>>();
        format!("(?:{})", choices.join("|"))
    }
}

/// A `[..]` class body as a regex class; `!` or `^` first negates it.
fn character_class(pattern: &[char]) -> String {
    let (negated, pattern) = pattern
        .strip_prefix(&['!'])
        .or_else(|| pattern.strip_prefix(&['^']))
        .map_or((false, pattern), |pattern| (true, pattern));
    let mut ranges = String::new();
    let mut index = 0;
    while index < pattern.len() {
        let (start, end, width) = if index + 2 < pattern.len() && pattern[index + 1] == '-' {
            (pattern[index], pattern[index + 2], 3)
        } else {
            (pattern[index], pattern[index], 1)
        };
        if start <= end {
            regex_syntax::escape_into(start.encode_utf8(&mut [0; 4]), &mut ranges);
            ranges.push('-');
            regex_syntax::escape_into(end.encode_utf8(&mut [0; 4]), &mut ranges);
        }
        index += width;
    }
    match (ranges.is_empty(), negated) {
        (true, false) => r"[^\s\S]".to_owned(),
        (true, true) => ".".to_owned(),
        (false, false) => format!("[{ranges}]"),
        (false, true) => format!("[^{ranges}]"),
    }
}

fn matching(pattern: &[char], open: usize, opening: char, closing: char) -> Option<usize> {
    let mut depth = 0;
    for (offset, character) in pattern[open..].iter().enumerate() {
        if *character == opening {
            depth += 1;
        } else if *character == closing {
            depth -= 1;
            if depth == 0 {
                return Some(open + offset);
            }
        }
    }
    None
}

/// Splits on `separator` outside nested `()`, `{}` and `[]`.
fn split_choices(pattern: &[char], separator: char) -> Vec<&[char]> {
    let mut choices = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0;
    for (offset, character) in pattern.iter().enumerate() {
        match character {
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth -= 1,
            character if *character == separator && depth == 0 => {
                choices.push(&pattern[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    choices.push(&pattern[start..]);
    choices
}
