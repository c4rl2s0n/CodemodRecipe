# Scaffold

`flutter.model` also defines top-level `scaffold.apply(c, args)` — create a new model + store from `args.name` via host `componentScaffold` (no `instanceId`).

Child nodes include `parseContext` (the context table passed to `c:component(...):parse`) for future child-level incremental reparse.
