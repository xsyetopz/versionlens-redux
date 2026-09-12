use std::path::Path;

type ActiveStates<'a> = &'a mut [bool];
type GlobChoices<'a> = Vec<&'a [char]>;

#[derive(Debug, Clone)]
pub(super) struct WorkspacePattern {
    segments: Vec<SegmentPattern>,
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
                    Some(SegmentPattern::Recursive)
                } else {
                    ComponentPattern::new(segment).map(SegmentPattern::Component)
                }
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self { segments })
    }

    pub(super) fn matches(&self, path: &[&str]) -> bool {
        matches_segments(&self.segments, path, false)
    }

    pub(super) fn could_match_descendant(&self, path: &[&str]) -> bool {
        matches_segments(&self.segments, path, true)
    }

    pub(super) fn explicitly_names(&self, name: &str) -> bool {
        self.segments.iter().any(|segment| match segment {
            SegmentPattern::Recursive => false,
            SegmentPattern::Component(pattern) => pattern.explicitly_names(name),
        })
    }
}

#[derive(Debug, Clone)]
enum SegmentPattern {
    Recursive,
    Component(ComponentPattern),
}

fn matches_segments(pattern: &[SegmentPattern], path: &[&str], prefix: bool) -> bool {
    fn close_recursive(pattern: &[SegmentPattern], active: ActiveStates<'_>) {
        for index in 0..pattern.len() {
            if active[index] && matches!(pattern[index], SegmentPattern::Recursive) {
                active[index + 1] = true;
            }
        }
    }

    let mut active = vec![false; pattern.len() + 1];
    active[0] = true;
    close_recursive(pattern, &mut active);
    for component in path {
        let mut next = vec![false; pattern.len() + 1];
        for (index, segment) in pattern.iter().enumerate() {
            if !active[index] {
                continue;
            }
            match segment {
                SegmentPattern::Recursive => next[index] = true,
                SegmentPattern::Component(segment) if segment.matches(component) => {
                    next[index + 1] = true;
                }
                SegmentPattern::Component(_) => {}
            }
        }
        close_recursive(pattern, &mut next);
        active = next;
    }
    active[pattern.len()] || (prefix && active.into_iter().any(|state| state))
}

#[derive(Debug, Clone)]
struct ComponentPattern {
    matcher: ComponentMatcher,
    has_wildcards: bool,
}

impl ComponentPattern {
    fn new(pattern: &str) -> Option<Self> {
        let pattern = pattern.chars().collect::<Vec<_>>();
        if let Some(choices) = whole_negative_extglob(&pattern) {
            if choices
                .iter()
                .any(|choice| contains_negative_extglob(choice))
            {
                return None;
            }
            return Some(Self {
                matcher: ComponentMatcher::Negative(
                    choices
                        .into_iter()
                        .map(glob_sequence)
                        .map(GlobAutomaton::new)
                        .collect(),
                ),
                has_wildcards: true,
            });
        }
        if contains_negative_extglob(&pattern) {
            return None;
        }
        let expression = glob_sequence(&pattern);
        let has_wildcards = expression.has_wildcards();
        Some(Self {
            matcher: ComponentMatcher::Positive(GlobAutomaton::new(expression)),
            has_wildcards,
        })
    }

    fn matches(&self, value: &str) -> bool {
        match &self.matcher {
            ComponentMatcher::Positive(matcher) => matcher.matches(value),
            ComponentMatcher::Negative(excluded) => {
                !excluded.iter().any(|matcher| matcher.matches(value))
            }
        }
    }

    fn explicitly_names(&self, name: &str) -> bool {
        !self.has_wildcards && self.matches(name)
    }
}

#[derive(Debug, Clone)]
enum ComponentMatcher {
    Positive(GlobAutomaton),
    Negative(Vec<GlobAutomaton>),
}

#[derive(Debug, Clone)]
struct GlobAutomaton {
    states: Vec<Vec<GlobEdge>>,
    accepting: usize,
}

impl GlobAutomaton {
    fn new(expression: GlobExpression) -> Self {
        let mut compiler = GlobCompiler::default();
        let start = compiler.state();
        let accepting = compiler.state();
        compiler.compile(&expression, start, accepting);
        Self {
            states: compiler.states,
            accepting,
        }
    }

    fn matches(&self, value: &str) -> bool {
        let mut active = vec![false; self.states.len()];
        active[0] = true;
        self.epsilon_closure(&mut active);
        for character in value.chars() {
            let mut next = vec![false; self.states.len()];
            for (state, edges) in self.states.iter().enumerate() {
                if !active[state] {
                    continue;
                }
                for edge in edges {
                    if let GlobEdge::Character(matcher, target) = edge
                        && matcher.matches(character)
                    {
                        next[*target] = true;
                    }
                }
            }
            self.epsilon_closure(&mut next);
            active = next;
        }
        active[self.accepting]
    }

    fn epsilon_closure(&self, active: ActiveStates<'_>) {
        let mut pending = active
            .iter()
            .enumerate()
            .filter_map(|(state, active)| active.then_some(state))
            .collect::<Vec<_>>();
        while let Some(state) = pending.pop() {
            for edge in &self.states[state] {
                if let GlobEdge::Epsilon(target) = edge
                    && !active[*target]
                {
                    active[*target] = true;
                    pending.push(*target);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum GlobExpression {
    Sequence(Vec<Self>),
    Alternative(Vec<Self>),
    Repeated {
        expression: Box<Self>,
        repetition: GlobRepetition,
    },
    Character(GlobCharacter),
    Star,
}

#[derive(Debug, Clone, Copy)]
enum GlobRepetition {
    Optional,
    ZeroOrMore,
    OneOrMore,
}

impl GlobExpression {
    fn has_wildcards(&self) -> bool {
        match self {
            Self::Sequence(values) | Self::Alternative(values) => {
                values.iter().any(Self::has_wildcards)
            }
            Self::Repeated { .. } | Self::Star => true,
            Self::Character(character) => character.is_wildcard(),
        }
    }
}

fn glob_sequence(pattern: &[char]) -> GlobExpression {
    let mut expressions = Vec::new();
    let mut index = 0;
    while index < pattern.len() {
        if let Some((expression, next)) = positive_extglob(pattern, index) {
            expressions.push(expression);
            index = next;
            continue;
        }
        match pattern[index] {
            '*' => {
                expressions.push(GlobExpression::Star);
                index += 1;
            }
            '?' => {
                expressions.push(GlobExpression::Character(GlobCharacter::Any));
                index += 1;
            }
            '[' => {
                if let Some(end) = pattern[index + 1..]
                    .iter()
                    .position(|character| *character == ']')
                    .map(|end| index + end + 1)
                {
                    expressions.push(GlobExpression::Character(character_class(
                        &pattern[index + 1..end],
                    )));
                    index = end + 1;
                } else {
                    expressions.push(literal_expression('['));
                    index += 1;
                }
            }
            '{' => {
                if let Some(end) = matching_brace(pattern, index) {
                    let choices = pattern_choices(&pattern[index + 1..end], ',');
                    if choices.len() > 1 {
                        expressions.push(GlobExpression::Alternative(
                            choices.into_iter().map(glob_sequence).collect(),
                        ));
                        index = end + 1;
                    } else {
                        expressions.push(literal_expression('{'));
                        index += 1;
                    }
                } else {
                    expressions.push(literal_expression('{'));
                    index += 1;
                }
            }
            character => {
                expressions.push(literal_expression(character));
                index += 1;
            }
        }
    }
    GlobExpression::Sequence(expressions)
}

fn positive_extglob(pattern: &[char], index: usize) -> Option<(GlobExpression, usize)> {
    let operator = *pattern.get(index)?;
    if !matches!(operator, '@' | '*' | '+' | '?') || pattern.get(index + 1) != Some(&'(') {
        return None;
    }
    let end = matching_parenthesis(pattern, index + 1)?;
    let choices = pattern_choices(&pattern[index + 2..end], '|');
    let expression = GlobExpression::Alternative(choices.into_iter().map(glob_sequence).collect());
    let expression = match operator {
        '@' => expression,
        '?' => repeated(expression, GlobRepetition::Optional),
        '*' => repeated(expression, GlobRepetition::ZeroOrMore),
        '+' => repeated(expression, GlobRepetition::OneOrMore),
        _ => return None,
    };
    Some((expression, end + 1))
}

fn repeated(expression: GlobExpression, repetition: GlobRepetition) -> GlobExpression {
    GlobExpression::Repeated {
        expression: Box::new(expression),
        repetition,
    }
}

fn whole_negative_extglob(pattern: &[char]) -> Option<GlobChoices<'_>> {
    if !pattern.starts_with(&['!', '(']) {
        return None;
    }
    let end = matching_parenthesis(pattern, 1)?;
    (end + 1 == pattern.len()).then(|| pattern_choices(&pattern[2..end], '|'))
}

fn contains_negative_extglob(pattern: &[char]) -> bool {
    pattern
        .windows(2)
        .any(|characters| characters == ['!', '('])
}

fn matching_parenthesis(pattern: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (offset, character) in pattern[open..].iter().enumerate() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn pattern_choices(pattern: &[char], separator: char) -> GlobChoices<'_> {
    let mut choices = Vec::new();
    let mut parentheses = 0;
    let mut braces = 0;
    let mut brackets = 0;
    let mut start = 0;
    for (offset, character) in pattern.iter().enumerate() {
        match character {
            '(' => parentheses += 1,
            ')' => parentheses -= 1,
            '{' => braces += 1,
            '}' => braces -= 1,
            '[' => brackets += 1,
            ']' => brackets -= 1,
            character
                if *character == separator && parentheses == 0 && braces == 0 && brackets == 0 =>
            {
                choices.push(&pattern[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    choices.push(&pattern[start..]);
    choices
}

fn matching_brace(pattern: &[char], open: usize) -> Option<usize> {
    let mut depth = 0;
    for (offset, character) in pattern[open..].iter().enumerate() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn literal_expression(character: char) -> GlobExpression {
    GlobExpression::Character(GlobCharacter::Literal(character))
}

fn character_class(pattern: &[char]) -> GlobCharacter {
    let (negated, pattern) = pattern
        .strip_prefix(&['!'])
        .or_else(|| pattern.strip_prefix(&['^']))
        .map_or((false, pattern), |pattern| (true, pattern));
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < pattern.len() {
        if index + 2 < pattern.len() && pattern[index + 1] == '-' {
            ranges.push((pattern[index], pattern[index + 2]));
            index += 3;
        } else {
            ranges.push((pattern[index], pattern[index]));
            index += 1;
        }
    }
    GlobCharacter::Class { negated, ranges }
}

#[derive(Debug, Clone)]
enum GlobCharacter {
    Literal(char),
    Any,
    Class {
        negated: bool,
        ranges: Vec<(char, char)>,
    },
}

impl GlobCharacter {
    fn matches(&self, character: char) -> bool {
        match self {
            Self::Literal(expected) => character == *expected,
            Self::Any => true,
            Self::Class { negated, ranges } => {
                let matched = ranges
                    .iter()
                    .any(|(start, end)| *start <= character && character <= *end);
                matched != *negated
            }
        }
    }

    fn is_wildcard(&self) -> bool {
        !matches!(self, Self::Literal(_))
    }
}

#[derive(Debug, Clone)]
enum GlobEdge {
    Epsilon(usize),
    Character(GlobCharacter, usize),
}

#[derive(Default)]
struct GlobCompiler {
    states: Vec<Vec<GlobEdge>>,
}

impl GlobCompiler {
    fn state(&mut self) -> usize {
        self.states.push(Vec::new());
        self.states.len() - 1
    }

    fn compile(&mut self, expression: &GlobExpression, start: usize, end: usize) {
        match expression {
            GlobExpression::Sequence(expressions) => {
                if expressions.is_empty() {
                    self.states[start].push(GlobEdge::Epsilon(end));
                    return;
                }
                let mut current = start;
                for (index, expression) in expressions.iter().enumerate() {
                    let next = if index + 1 == expressions.len() {
                        end
                    } else {
                        self.state()
                    };
                    self.compile(expression, current, next);
                    current = next;
                }
            }
            GlobExpression::Alternative(expressions) => {
                for expression in expressions {
                    self.compile(expression, start, end);
                }
            }
            GlobExpression::Repeated {
                expression,
                repetition,
            } => match repetition {
                GlobRepetition::Optional => {
                    self.states[start].push(GlobEdge::Epsilon(end));
                    self.compile(expression, start, end);
                }
                GlobRepetition::ZeroOrMore => {
                    self.states[start].push(GlobEdge::Epsilon(end));
                    self.compile(expression, start, start);
                }
                GlobRepetition::OneOrMore => {
                    let repeated = self.state();
                    self.compile(expression, start, repeated);
                    self.states[repeated].push(GlobEdge::Epsilon(end));
                    self.compile(expression, repeated, repeated);
                }
            },
            GlobExpression::Character(character) => {
                self.states[start].push(GlobEdge::Character(character.clone(), end));
            }
            GlobExpression::Star => {
                self.states[start].push(GlobEdge::Epsilon(end));
                self.states[start].push(GlobEdge::Character(GlobCharacter::Any, start));
            }
        }
    }
}
