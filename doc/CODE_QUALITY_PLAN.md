# Code Quality Improvement Plan

## Overview

This document outlines systematic improvements to the codemod_recipe codebase to enhance maintainability, readability, and adherence to software engineering best practices. The plan follows SOLID, DRY, KISS principles, and Dart-specific conventions.

## Priority Levels

- **🔴 High**: Critical for maintainability, affects multiple areas
- **🟡 Medium**: Important improvements, nice to have
- **🟢 Low**: Nice to have, can wait

---

## High Priority Tasks

### 1. Extract Utilities and Constants

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-H-1 | Create `lib/src/constants.dart` for all magic strings and numbers | New file | DRY | Centralizes configuration |
| Q-H-2 | Extract argument parsing into `HostArgsParser` class | `bin/codemod_host.dart` | SRP | Reusable, testable |
| Q-H-3 | Move helper functions to utility classes | `bin/codemod_host.dart` | SRP | Cleaner separation |
| Q-H-4 | Create `lib/src/utils/file_utils.dart` for file operations | New file | DRY | Centralizes FS logic |
| Q-H-5 | Standardize error handling with custom exception hierarchy | `lib/src/` | Consistency | Better error reporting |

**Deliverables:**
- New `constants.dart` with all hardcoded values
- Dedicated parser classes
- Reusable utility functions

---

### 2. Refactor AST Path System

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-H-6 | Extract anchor validation into strategy pattern | `anchors.dart` | OCP | Extensible for new anchors |
| Q-H-7 | Refactor `_applyStep` using polymorphism | `interpreter.dart` | Polymorphism | Cleaner, more maintainable |
| Q-H-8 | Consolidate duplicate navigation parsing | `parser.dart`, `class_focus.dart` | DRY | Eliminates redundancy |
| Q-H-9 | Create `AstPathBuilder` for fluent construction | `ast_path/` | Fluent API | Easier path creation |
| Q-H-10 | Centralize offset resolution helpers | `offsets.dart` | SRP | Single source of truth |

**Deliverables:**
- Strategy pattern for anchor validation
- Polymorphic step resolution
- Unified parsing logic
- Fluent path builder API

---

### 3. Improve YAML Recipe Processing

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-H-11 | Extract YAML schema validation | `recipe_compiler.dart` | Separation of Concerns | Independent validation |
| Q-H-12 | Create typed DSL classes for recipe elements | `yaml/` | Type Safety | Stronger typing |
| Q-H-13 | Standardize template rendering | Across YAML processing | Consistency | Uniform behavior |
| Q-H-14 | Centralize diagnostic message formatting | `diagnostics.dart` | DRY | Consistent errors |

**Deliverables:**
- Separate validation module
- Typed DSL representation
- Consistent template handling

---

## Medium Priority Tasks

### 1. Code Organization

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-M-1 | Standardize path handling with `path` package | Across codebase | Consistency | No manual string manipulation |
| Q-M-2 | Add logging framework integration | Across codebase | Debuggability | Better troubleshooting |
| Q-M-3 | Extract string helpers (casing, etc.) | `context.dart` | DRY | Reusable transformations |
| Q-M-4 | Review and standardize test patterns | `test/` | Consistency | Maintainable tests |

### 2. Type Safety

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-M-5 | Add more specific types instead of `dynamic` | Across codebase | Type Safety | Fewer runtime errors |
| Q-M-6 | Use `typedef` for complex function signatures | Where applicable | Readability | Clearer interfaces |
| Q-M-7 | Add null safety annotations | Across codebase | Null Safety | More robust code |

### 3. Documentation

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-M-8 | Add DartDoc to all public APIs | Public methods/classes | Maintainability | Better developer experience |
| Q-M-9 | Document architecture decisions | New `ARCHITECTURE.md` | Knowledge Sharing | Easier onboarding |
| Q-M-10 | Add code examples to complex classes | Across codebase | Clarity | Easier to understand |

---

## Low Priority Tasks

| ID | Task | Location | Principle | Impact |
|----|------|----------|-----------|--------|
| Q-L-1 | Create migration guide for breaking changes | New file | Documentation | User support |
| Q-L-2 | Add performance benchmarks | New file | Performance | Optimization targets |
| Q-L-3 | Review and refactor test fixtures | `test/fixtures/` | Maintainability | Cleaner tests |
| Q-L-4 | Add contribution guidelines for code quality | `CONTRIBUTING.md` | Onboarding | Consistent contributions |

---

## Best Practices Reference

### SOLID Principles

1. **Single Responsibility Principle (SRP)**
   - Each class should have one reason to change
   - Example: `AstPathParser` should only parse, not validate or resolve

2. **Open/Closed Principle (OCP)**
   - Open for extension, closed for modification
   - Example: Use strategy pattern for anchor validation so new anchors don't require modifying existing code

3. **Liskov Substitution Principle (LSP)**
   - Subtypes must be substitutable for their base types
   - Example: All `NavigateStep` implementations should work interchangeably

4. **Interface Segregation Principle (ISP)**
   - Clients shouldn't depend on interfaces they don't use
   - Example: Split large interfaces into smaller, focused ones

5. **Dependency Inversion Principle (DIP)**
   - Depend on abstractions, not concretions
   - Example: `AstPathInterpreter` should depend on `AstFocus` abstraction, not concrete implementations

### DRY (Don't Repeat Yourself)

- Centralize common logic in utility classes
- Extract duplicate code into reusable functions
- Use extension methods for common operations on types

### KISS (Keep It Simple, Stupid)

- Prefer simple solutions over complex ones
- Avoid over-engineering
- Write code that's easy to understand and maintain

### Dart-Specific Best Practices

- Use null safety (`?`, `!`, `late`) appropriately
- Prefer immutable objects where possible
- Use Dart's built-in collections effectively
- Follow Dart naming conventions (camelCase, PascalCase)
- Use DartDoc comments for public APIs

---

## Quality Metrics

Track these metrics to measure improvement:

| Metric | Current | Target | Tool |
|--------|---------|--------|------|
| Cyclomatic Complexity | ? | < 10 per method | `dart_analyzer` |
| Code Coverage | ? | > 80% | `coverage` package |
| Lines of Code per Method | ? | < 50 | Manual review |
| Number of Parameters | ? | < 4 | Manual review |
| Comment-to-Code Ratio | ? | > 15% | Manual review |

---

## Execution Strategy

### Phase 1: Foundation (Week 1-2)
- Q-H-1 through Q-H-5 (constants and utilities)
- Q-H-11 through Q-H-14 (YAML processing)

### Phase 2: Core Refactoring (Week 3-4)
- Q-H-6 through Q-H-10 (AST path system)
- Q-M-1 through Q-M-7 (organization and type safety)

### Phase 3: Documentation (Week 5)
- Q-M-8 through Q-M-10 (documentation)
- Q-L-1 through Q-L-4 (low priority)

---

## Acceptance Criteria

Each task should:
1. ✅ Include tests for new functionality
2. ✅ Maintain backward compatibility (unless breaking change is approved)
3. ✅ Pass existing test suite
4. ✅ Follow Dart formatting (`dart format`)
5. ✅ Pass static analysis (`dart analyze`)

---

## Resources

- [Dart Style Guide](https://dart.dev/guides/language/effective-dart/style)
- [Clean Code Dart](https://github.com/williambarreiro/clean-code-dart)
- [SOLID Principles](https://en.wikipedia.org/wiki/SOLID)
- [Effective Dart](https://dart.dev/guides/language/effective-dart)

---

## Version History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-17 | Vibe | Initial plan created |
