# Runtime vs IDE

Luau types are **IDE-only**. The host passes plain tables at runtime. Components validate parse context with ordinary `if not context.source then return nil end` checks. Runtime `require` loads real library modules; it does not execute `.d.luau` stubs.
