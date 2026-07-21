use tree_sitter::Language;

use crate::span;

pub trait LanguageAdapter: Send + Sync {
    fn language(&self) -> Language;

    fn expand_remove_span(
        &self,
        source: &str,
        start: usize,
        end: usize,
        include_leading_trivia: bool,
    ) -> (usize, usize) {
        let (start, end) = if include_leading_trivia {
            span::expand_declaration_span(source, start, end)
        } else {
            (span::line_start_offset(source, start), end)
        };
        (start, span::expand_trailing_semicolon(source, end))
    }
}

/// Default adapter: Dart-style `///` and `//` leading trivia expansion.
pub struct DefaultLanguageAdapter {
    language: Language,
}

impl DefaultLanguageAdapter {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

impl LanguageAdapter for DefaultLanguageAdapter {
    fn language(&self) -> Language {
        self.language.clone()
    }
}

/// Dart uses the same span rules as the default adapter today.
pub type DartLanguageAdapter = DefaultLanguageAdapter;

/// Java, Kotlin, and Rust: block comments (`/*`, `/**`) in addition to line comments.
pub struct CStyleLanguageAdapter {
    language: Language,
}

impl CStyleLanguageAdapter {
    pub fn new(language: Language) -> Self {
        Self { language }
    }
}

impl LanguageAdapter for CStyleLanguageAdapter {
    fn language(&self) -> Language {
        self.language.clone()
    }

    fn expand_remove_span(
        &self,
        source: &str,
        start: usize,
        end: usize,
        include_leading_trivia: bool,
    ) -> (usize, usize) {
        let (start, end) = if include_leading_trivia {
            span::expand_cstyle_declaration_span(source, start, end)
        } else {
            (span::line_start_offset(source, start), end)
        };
        (start, span::expand_trailing_semicolon(source, end))
    }
}

pub fn adapter_for_language(language_id: &str, language: Language) -> Box<dyn LanguageAdapter> {
    match language_id {
        "rust" | "java" | "kotlin" => Box::new(CStyleLanguageAdapter::new(language)),
        _ => Box::new(DefaultLanguageAdapter::new(language)),
    }
}
