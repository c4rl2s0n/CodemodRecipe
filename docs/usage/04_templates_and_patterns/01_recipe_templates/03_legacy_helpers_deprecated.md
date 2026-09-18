# Legacy helpers (deprecated)

These still work via a pre-pass shim but emit `W_LEGACY_TEMPLATE` during
validation:

| Legacy | Canonical |
|--------|-----------|
| `{{$snake x}}` | `{{ x \| snake_case }}` |
| `{{$camel x}}` | `{{ x \| camel_case }}` |
| `{{$pascal x}}` | `{{ x \| pascal_case }}` |
| `{{$map 'id' key}}` | `{{ key \| map('id') }}` |
