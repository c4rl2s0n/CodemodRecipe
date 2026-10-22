# mdBook (human navigation)

| Book root | Audience | Contents |
|-----------|----------|----------|
| **`docs/`** (`book.toml`) | This repo | Usage chapters + `develop/` |
| **`docs/usage/`** (`book.toml`) | Consumers (bootstrapped) | Usage only |

Every docs folder has a short **`00_index.md`** landing (not an mdBook requirement —
`SUMMARY.md` is the nav SSOT). Topic pages are bite-sized: one former `##` section
per file.

```bash
cargo install mdbook mdbook-mermaid

# This repository (includes develop):
cd docs
mdbook-mermaid install .       # local mermaid*.js only (gitignored)
mdbook serve

# Consumer workspace (usage only):
cd .agents/skills/codemod-recipe/docs
mdbook-mermaid install .
mdbook serve
```

You need **both** binaries: `mdbook` builds/serves; `mdbook-mermaid` preprocesses
fenced ` ```mermaid ` blocks. Do **not** commit `mermaid*.js` or `book/` HTML.
Bootstrap never embeds `*.js` or a nested `book/` directory from usage docs.

Regenerate both `docs/SUMMARY.md` and `docs/usage/SUMMARY.md` after adding or
renaming chapter pages:

```bash
python3 scripts/generate_mdbook_summary.py
```
