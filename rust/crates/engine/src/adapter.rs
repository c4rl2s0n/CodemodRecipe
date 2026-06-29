use tree_sitter::Language;

use crate::span;

pub trait LanguageAdapter {
    fn language(&self) -> Language;

    fn expand_remove_span(
        &self,
        source: &str,
        start: usize,
        end: usize,
        include_leading_trivia: bool,
    ) -> (usize, usize) {
        if include_leading_trivia {
            span::expand_declaration_span(source, start, end)
        } else {
            (start, end)
        }
    }
}

pub struct DartAdapter;

impl LanguageAdapter for DartAdapter {
    fn language(&self) -> Language {
        crate::dart::language()
    }
}
