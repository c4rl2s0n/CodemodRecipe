# `defaultsTo`

Recipe args may declare `defaultsTo`. The host applies defaults before required-arg
checks and before rendering. Child recipe defaults are visible after orchestrator
expansion (parent arg definitions win on name collision).
