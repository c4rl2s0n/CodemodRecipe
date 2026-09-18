# Parse tree

Root `parse` returns a **node** with optional **`children`** and **`source`** (`path`, optional `span`) for tooling. Scan JSON exposes the tree as **`node`** (not `parsed`). Agents read **`summary`** / shallow fields; `source` spans are for extract/edit.
