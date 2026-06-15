# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- YAML recipe front-end: AST path DSL, YAML compiler/registry, generic `bin/codemod_host.dart`, VS Code extension YAML host wiring.
- YAML steps: `addConstructorParam`; post-execution `buildRunner`.
- Example YAML recipes under `.codemod/recipes/`.
- Built-in transforms accept optional `navigate` steps via `resolveClassFocus`; `StringResolver` API restored on transform types.
- `dart run` executables for all examples.
- Extended test suite with edge-case tests for template rendering.
- Enhanced documentation with inline code examples in docstrings.

### Fixed
- `FieldConstructorArgs.style` is nullable so `CodemodPreferences.emptyConstructorStyle` applies when no per-field override is set.

## [0.1.0] - 2026-06-01

### Added
- Initial release of `codemod_recipe`.
- `CodemodRecipe` for declaring arguments, ordered operations, and post-execution actions.
- `CodemodRunner` for CLI parsing, dry-run/apply workflows, and error handling.
- `CodemodContext` for storing raw argument values and providing generic case helpers (snake, camel, pascal).
- `CodemodTemplate` for rendering inline or file-backed templates with Mustache-style placeholders and casing filters.
- `CodeEditor` for AST-guided Dart source modifications including adding methods, fields, and constructor parameters.
- `CodemodOperation` abstractions:
  - `EditDartFileOperation` for editing existing Dart files with transforms.
  - `CreateFileOperation` for scaffolding new files from templates.
- `CodeTransform` interface for deterministic source-to-patches operations.
- Generic transforms:
  - `AddImportTransform` for adding import directives.
  - `AddMethodTransform` for adding methods to classes.
  - `AddFieldTransform` for adding fields to classes.
  - `AddConstructorParamTransform` for adding constructor parameters.
  - `AddClassAnnotationTransform` for adding annotations to classes.
  - `TemplateTransform` for inserting rendered snippets at custom offsets.
- Post-execution actions:
  - `DartFormatPostExecution` for running `dart format` on changed files.
  - `BuildRunnerPostExecution` for running `build_runner` after changes.
  - `ProcessPostExecution` for running custom processes.
- `FileChange` abstractions:
  - `PatchFileChange` for patch-based edits to existing files.
  - `CreateFileChange` for full-content file creation or overwriting.
- Patch helper utilities including `applyPatches`, `validateNonOverlappingPatches`, and AST-based patch builders.
- Comprehensive test coverage for all core components.
