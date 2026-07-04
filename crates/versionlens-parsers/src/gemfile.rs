mod github;
mod line;
mod sources;
mod standard;
mod syntax;

use crate::model::Dependency;
use github::{gem_github_default_dependency, gem_github_ref_dependency, gem_github_tag_dependency};
use line::{GemLineContext, GemNameSpan};
use sources::source_block_url_from_line;
use standard::standard_gem_dependency;
use syntax::{quoted_strings, strip_comment};

pub use sources::parse_gemfile_source_urls;

pub(crate) fn parse_gemfile(text: &str) -> Vec<Dependency> {
    let mut parser = GemfileParser::default();

    for (line_index, line) in text.lines().enumerate() {
        parser.collect_line(line_index, line);
    }

    parser.dependencies
}

#[derive(Default)]
struct GemfileParser<'a> {
    block_stack: Vec<GemfileBlock<'a>>,
    dependencies: Vec<Dependency>,
}

enum GemfileBlock<'a> {
    Group(&'a str),
    Source(String),
}

impl<'a> GemfileParser<'a> {
    fn collect_line(&mut self, line_index: usize, line: &'a str) {
        let trimmed = line.trim_start();
        if trimmed.trim_end() == "end" {
            self.block_stack.pop();
            return;
        }
        if let Some(url) = source_block_url_from_line(trimmed) {
            self.block_stack.push(GemfileBlock::Source(url));
            return;
        }
        if trimmed.starts_with("group ") && trimmed.ends_with(" do") {
            self.block_stack
                .push(GemfileBlock::Group(trimmed.trim_end_matches(" do")));
            return;
        }

        if let Some(dependency) = parse_gem_line(
            line_index,
            line,
            self.current_group(),
            self.current_source(),
        ) {
            self.dependencies.push(dependency);
        }
    }

    fn current_group(&self) -> &str {
        self.block_stack
            .iter()
            .rev()
            .find_map(|block| match block {
                GemfileBlock::Group(group) => Some(*group),
                GemfileBlock::Source(_) => None,
            })
            .unwrap_or("dependencies")
    }

    fn current_source(&self) -> Option<&str> {
        self.block_stack.iter().rev().find_map(|block| match block {
            GemfileBlock::Source(url) => Some(url.as_str()),
            GemfileBlock::Group(_) => None,
        })
    }
}

fn parse_gem_line(
    line_index: usize,
    line: &str,
    group: &str,
    source_url: Option<&str>,
) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("gem ") {
        return None;
    }

    let context = GemLineContext {
        line_index,
        line,
        offset: line.len() - trimmed.len(),
        content: strip_comment(trimmed).trim_end(),
        group,
    };
    let mut strings = quoted_strings(context.content);
    let (name, _, end) = strings.next()?;
    let name = GemNameSpan { name, end };

    gem_github_tag_dependency(&context, &name)
        .or_else(|| gem_github_ref_dependency(&context, &name))
        .or_else(|| gem_github_default_dependency(&context, &name))
        .or_else(|| {
            Some(standard_gem_dependency(
                &context,
                &name,
                strings.next(),
                source_url,
            ))
        })
}

#[cfg(test)]
mod tests;
