//! LRU cache of parsed tree-sitter trees keyed by workspace path + content hash.

use tree_sitter::{Language, Tree};

const MAX_ENTRIES: usize = 8;

#[derive(Clone)]
struct CachedParse {
    content_hash: String,
    source: String,
    tree: Tree,
}

/// Parsed AST cache shared by Query Tools host commands.
#[derive(Default)]
pub struct AstParseCache {
    /// LRU order: front = oldest, back = newest.
    entries: Vec<(String, CachedParse)>,
    /// Number of actual parse operations (for tests / diagnostics).
    parse_count: u64,
}

pub struct CachedTree {
    pub source: String,
    pub tree: Tree,
    pub cache_hit: bool,
}

impl AstParseCache {
    pub fn parse_count(&self) -> u64 {
        self.parse_count
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn get_or_parse(
        &mut self,
        path_key: &str,
        source: &str,
        language: &Language,
    ) -> Result<CachedTree, String> {
        let hash = content_hash(source);
        if let Some(idx) = self.entries.iter().position(|(p, _)| p == path_key) {
            let entry = &self.entries[idx].1;
            if entry.content_hash == hash {
                let (_, entry) = self.entries.remove(idx);
                self.entries.push((path_key.to_string(), entry.clone()));
                let back = self.entries.last().expect("just pushed");
                return Ok(CachedTree {
                    source: back.1.source.clone(),
                    tree: back.1.tree.clone(),
                    cache_hit: true,
                });
            }
            self.entries.remove(idx);
        }

        let tree = codemod_recipe_query_tools::parse_tree(language, source)
            .map_err(|e| e.to_string())?;
        self.parse_count += 1;
        let cached = CachedParse {
            content_hash: hash,
            source: source.to_string(),
            tree: tree.clone(),
        };
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.remove(0);
        }
        self.entries.push((path_key.to_string(), cached));
        Ok(CachedTree {
            source: source.to_string(),
            tree,
            cache_hit: false,
        })
    }
}

fn content_hash(source: &str) -> String {
    format!("{:x}", md5::compute(source.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter_language_pack::get_language;

    fn dart() -> Language {
        get_language("dart").expect("dart")
    }

    #[test]
    fn cache_hit_on_same_path_and_source() {
        let lang = dart();
        let mut cache = AstParseCache::default();
        let src = "void main() {}";
        let a = cache.get_or_parse("lib/a.dart", src, &lang).unwrap();
        assert!(!a.cache_hit);
        assert_eq!(cache.parse_count(), 1);
        let b = cache.get_or_parse("lib/a.dart", src, &lang).unwrap();
        assert!(b.cache_hit);
        assert_eq!(cache.parse_count(), 1);
        assert_eq!(a.tree.root_node().kind(), b.tree.root_node().kind());
    }

    #[test]
    fn cache_miss_when_source_changes() {
        let lang = dart();
        let mut cache = AstParseCache::default();
        cache
            .get_or_parse("lib/a.dart", "void main() {}", &lang)
            .unwrap();
        cache
            .get_or_parse("lib/a.dart", "void main() { print(1); }", &lang)
            .unwrap();
        assert_eq!(cache.parse_count(), 2);
    }

    #[test]
    fn lru_evicts_oldest() {
        let lang = dart();
        let mut cache = AstParseCache::default();
        for i in 0..=MAX_ENTRIES {
            cache
                .get_or_parse(&format!("lib/f{i}.dart"), &format!("void f{i}() {{}}"), &lang)
                .unwrap();
        }
        assert_eq!(cache.parse_count(), MAX_ENTRIES as u64 + 1);
        // Oldest key evicted — re-parsing it counts as a new parse.
        cache
            .get_or_parse("lib/f0.dart", "void f0() {}", &lang)
            .unwrap();
        assert_eq!(cache.parse_count(), MAX_ENTRIES as u64 + 2);
    }
}
