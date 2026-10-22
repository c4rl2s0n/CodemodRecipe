# Semantic delta in preview (spike)

Phase 3 from the relations plan: after a preview `apply` on the WorkingTree, rescan the
affected paths and diff instance/edge sets against the pre-change scan, surfacing an
"architectural delta" in the preview response alongside the file diffs.

Status: **design only** — no code yet. This spike maps the problem onto existing
machinery and lists what must be built.

## Motivation

File diffs answer "what text changes"; agents also need "what does this change *mean*":
does the recipe create a new model instance, remove a `stores` edge, resolve a drift
finding, introduce a cycle? Today the preview response contains only file diffs +
transcript, so the agent must re-run `component_scan` mentally (or actually, after
apply) to learn the architectural effect. The relations layer (edges, kindGraph,
graph-aware findings) makes that delta computable and worth surfacing.

## Current machinery (what we build on)

- Preview runs the recipe against an in-memory `WorkingTree`
  ([dispatch.rs](../../../rust/crates/host/src/protocol/dispatch.rs) `preview`); reads
  inside the recipe already see prior creates/edits in the same run (tree-overlaid
  reads). `finalize()` yields the `FileChange`s that become the file diffs.
- `previewToken` binds recipe + args + snapshot paths of the planned changes. The delta
  is informational and derived from the *same* collected changes, so it always agrees
  with the file diffs; the token is unaffected.
- Component scans read through the same host — but `make_host` hardcodes a fresh
  `WorkingTree::new()`, so scans always see **disk** state.
- Operations already do a post-operation rescan
  (`component_scan_after_operation` → `component_incremental_scan` with dirty paths =
  changed paths) and return fresh instance/findings — but **not a diff**, and in
  preview mode (`flush_to_disk` skipped) that rescan reads pre-change disk state. So
  today a previewed operation reports the *old* findings. The delta work fixes this
  fidelity gap as a side effect.

## Proposed design

### 1. Tree-overlaid scan (the real work)

Add a scan entry point that runs the full pipeline (identify → parse → relate →
verify) against a `WorkingTree` seeded from a `Vec<FileChange>`:

- `make_host` variant taking a seeded tree (or a `seed_from_changes(&WorkingTree)`
  helper that replays `FileChange`s as tree entries).
- `c:matchFiles` / `c:read` / `c:query` must resolve through the overlay: created
  files visible, deleted files gone, modified content read from the tree. (Read
  overlap mostly exists for in-recipe sequencing; `identify` globbing over tree
  entries needs an audit — it is the likely gap.)
- The post-change scan is **ephemeral**: it must not touch the real scan cache, which
  reflects disk. It borrows the cache's unaffected roots where possible (same
  merge logic as incremental scan, but sourcing dirty-root content from the tree).

### 2. Diff computation (pure function, host-owned)

`diff_scans(before: &ScanData, after: &ScanData) -> SemanticDelta`:

```json
{
  "instances": {
    "added":   ["flutter.model/order"],
    "removed": [],
    "changed": ["flutter.model/recipe"]
  },
  "edges": {
    "added":   [{ "from": "flutter.state/order", "kind": "stores", "to": "flutter.model/order" }],
    "removed": []
  },
  "findings": {
    "added":    [{ "code": "model.field_missing_in_store", "instanceId": "flutter.model/order", "message": "…" }],
    "resolved": [{ "code": "model.field_missing_in_store", "instanceId": "flutter.model/recipe", "message": "…" }]
  },
  "kindGraph": {
    "added": [], "removed": [],
    "countChanged": [{ "from": "flutter.state", "kind": "stores", "to": "flutter.model", "before": 2, "after": 3 }]
  }
}
```

Identity keys: flat instance id for instances (`changed` = same id, different node
JSON); `(from, kind, to|symbol)` for edges; `(code, instanceId, message)` for
findings; `(from, kind, to)` for kindGraph counts. All sets are already computed by
the scan — the diff is a cheap by-product.

### 3. Response surface

- `PreviewResponse` gains optional `semanticDelta` (camelCase), next to `files`.
- `ComponentInvokeResponse` (preview kind) gains the same; for apply kind the
  post-operation rescan already returns fresh state, and the delta can be reported
  against the pre-operation cache cheaply.
- **Opt-in flag** on preview/invoke requests (`semanticDelta: true`), default off:
  the post-change scan costs one extra scan cycle and most previews are pure text
  edits. MCP tools expose the flag; agents opt in when the change is structural.

### 4. Failure modes

- Post-change scan error (recipe produced unparseable code, broken query): never fail
  the preview — emit `semanticDelta: { "error": "…" }` (or omit + transcript note).
- No stack / no components loaded: field absent.
- No pre-change cache: build it on demand (same scan-on-demand rule as hook dry-run);
  if that scan fails, degrade as above.

## Phasing

1. **3a (operations first)**: `component_scan_after_operation` already knows the dirty
   paths and has the pre-state in cache. Add tree-overlaid rescan for preview kind +
   diff vs cache → `semanticDelta` on invoke responses. Smallest surface, fixes the
   preview-fidelity bug, exercises every piece.
2. **3b (recipe preview)**: seed tree from collected changes in `preview()`, reuse the
   same diff. Needs the `identify`-glob overlay audit (recipe previews create files
   more often than operations).
3. **3c (snapshots / drift over time)**: persist scan snapshots so drift detection
   survives sessions. Separate concern; only noted here because the diff function is
   the same. **Format decided 2026-09-28**: `.codemod/.cache/scan-snapshot.json`.
   **Committed by default** (2026-09-28): the snapshot reflects the architecture of
   the latest commit — its diff in a PR *is* the architectural delta, reviewable like
   `Cargo.lock` or insta snapshots. Two rules keep that clean: (1) deterministic
   serialization (sorted ordering, stable formatting) so diffs are minimal;
   (2) **rewrite only when the content changed** (compare ignoring `meta.timestamp`),
   so no-op scans never dirty the working tree. Teams that don't want it in git add
   one `.gitignore` line. Envelope with metadata, instances stored as a **structural
   projection without spans** (spans shift on any edit above the node, making
   full-fidelity snapshots both huge and noisy), edges without spans:

```json
{ "meta": { "version": 1, "timestamp": "...", "stack": "flutter-poc-stack" },
  "instances": [ { "id": "...", "componentId": "...", "ast": "<projection>" } ],
  "edges": [ { "from": "...", "kind": "...", "to": "...", "sourcePath": "..." } ],
  "kindGraph": [ ... ], "findings": [ ... ] }
```

   Written on full `component_scan`; when a previous snapshot exists, drift is
   reported against it. Full detail is always one rescan away — the snapshot answers
   "what changed since", not "reconstruct the state".

## Open questions

- `identify` overlay: does `matchFiles` already union tree entries into glob results?
  If not, what is the minimal sandbox change (tree-aware `exists`/`list` primitives)?
- Cost bound: full re-verify over the merged forest is the v1 price (same as
  incremental scan); acceptable for preview, but note it for huge workspaces.
- Should `semanticDelta` include `reachable`-style impact ("these instances lost their
  last incoming `stores` edge")? Deferred — findings already encode judgment; the
  delta is a set diff, not an inference engine.

## Resolved (2026-09-28 design discussion)

- **Duplicate edges**: merge into one canonical edge with `sources[]` + `count`;
  `relation.duplicate` info finding only for identical-provenance redundancy (a
  relate-hook bug). Distinct provenance merges silently — multiplicity is data. No
  duplicate-group ids: duplicates never exist as separate edges, so consumers
  (facade counts, kindGraph, exports) stay clean. (See
  [01_semantics.md](01_semantics.md).)
- **Facade surface**: keep the host-provided method API for common queries and add
  node-centric traversal (`inst:outgoing` / `:incoming` / `:follow` / `:neighbors`)
  via metatable on `byId`/`resolve*` results. Plain data needs no separate getter —
  `nodes()`/`edges()` already return the shared in-sandbox tables. Graph vocabulary:
  instances are the nodes (`all()` renamed to `nodes()`).
- **Operations graph access**: yes — already shipped for `operations.*.apply` (4th
  arg); extend to `scaffold.apply` (3rd arg) for symmetry. Always the pre-operation
  graph.
- **Diff granularity for `changed` instances**: structural projection without spans
  (same projection as the persisted snapshot).
- **Snapshot format/location**: `.codemod/.cache/scan-snapshot.json`, projection
  without spans, metadata envelope — see 3c above.

## Recommendation

Build 3a behind the opt-in flag. It is small, fixes a real preview-fidelity gap, and
validates tree-overlaid scanning before recipe preview (3b) depends on it.
