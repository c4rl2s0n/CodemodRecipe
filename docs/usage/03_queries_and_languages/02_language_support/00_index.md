# Multi-language support reference

codemod-recipe resolves a **tree-sitter grammar per edit op**, runs queries against that grammar, and applies byte-range patches. Grammars come from **tree-sitter-language-pack** (lazy download + cache) plus small **native addons** where the pack lacks dialect fidelity.

| Page | |
|------|---|
| [How language resolution works](01_how_language_resolution_works.md) | |
| [The `language` field](02_the_language_field.md) | |
| [SQL dialects](03_sql_dialects.md) | |
| [Grammar loading (language-pack)](04_grammar_loading_language_pack.md) | |
| [Dart grammar note (language-pack)](05_dart_grammar_note_language_pack.md) | |
| [Other languages (sketch)](06_other_languages_sketch.md) | |
| [Span / trivia adapters](07_span_trivia_adapters.md) | |
| [Host configuration summary](08_host_configuration_summary.md) | |
| [Troubleshooting](09_troubleshooting.md) | |
| [Related docs](10_related_docs.md) | |

