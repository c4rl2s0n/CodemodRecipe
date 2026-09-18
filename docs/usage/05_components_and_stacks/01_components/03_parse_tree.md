# Parse tree

Root `parse` returns a **node** with optional **`children`** and **`source`** (`path`, optional `span`) for tooling. Scan JSON exposes the tree as **`node`** (not `parsed`). Agents read **`summary`** / shallow fields; `source` spans are for extract/edit.

**Coverage:** only **coverage-leaf** explicit `source.span` ranges count toward [span coverage](09_coverage.md) (a parent with spanned descendants does not paint coverage). A path without `span` does not cover the file — attach spans from query hits on the pieces you model.
