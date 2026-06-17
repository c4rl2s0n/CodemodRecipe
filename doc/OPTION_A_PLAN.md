# Option A Implementation Plan: Split Binaries

## Overview

This plan outlines the steps to split the monolithic `bin/codemod_host.dart` into two focused entry points:
- **`bin/codemod.dart`**: CLI-only entry point for direct recipe execution
- **`bin/codemod_host.dart`**: VS Code extension only (stdio-server mode)

**Problem being solved:**
- CLI mode currently fails when there are errors in unrelated recipes (e.g., duplicate IDs)
- The binary tries to do both jobs (CLI and VS Code server), leading to complex conditional logic
- Debugging individual recipes is difficult because all recipes must validate

**Goal:**
- CLI loads only the requested recipe file
- VS Code extension continues to work with all recipes loaded
- Cleaner architecture with separated concerns

---

## Priority Levels

- **🔴 High**: Critical path, must be done first
- **🟡 Medium**: Important but can wait
- **🟢 Low**: Nice to have, non-blocking

---

## Phase 1: Create New CLI Entry Point (bin/codemod.dart)

### High Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-1.1 | Create `bin/codemod.dart` with basic structure | 🔴 High | None | 30 min | File exists, imports correct |
| option-a-1.2 | Implement single-recipe loading from file path | 🔴 High | 1.1 | 1 hour | Can load a YAML recipe from path |
| option-a-1.3 | Add argument parsing for recipe-specific flags | 🔴 High | 1.2 | 1 hour | `--file`, `--className`, etc. passed to recipe |
| option-a-1.7 | Add error handling and validation | 🔴 High | 1.2 | 1 hour | Clear errors for missing files, invalid YAML |

### Medium Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-1.4 | Add `--help` flag with usage information | 🟡 Medium | 1.1 | 30 min | Displays usage with examples |
| option-a-1.5 | Add `--map-root` flag for custom map directory | 🟡 Medium | 1.3 | 30 min | Maps loaded from custom directory |
| option-a-1.6 | Add dry-run and `--apply` support | 🟡 Medium | 1.3 | 1 hour | Can preview or apply changes |

### Task Details

#### 1.1: Create bin/codemod.dart with basic structure

**File:** `bin/codemod.dart`

**Skeleton:**
```dart
#!/usr/bin/env dart

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';

Future<void> main(List<String> arguments) async {
  // Implementation will go here
}
```

**Acceptance:**
- File compiles without errors
- Can be run with `dart run bin/codemod.dart`

---

#### 1.2: Implement single-recipe loading from file path

**Goal:** Load a YAML recipe directly from a file path without going through the registry.

**Implementation approach:**
```dart
Future<YamlRecipe?> loadRecipe(String path) async {
  final file = File(path);
  if (!await file.exists()) {
    stderr.writeln('Recipe file not found: $path');
    return null;
  }
  
  final content = await file.readAsString();
  final yaml = loadYaml(content) as YamlMap;
  
  return YamlRecipeCompiler.compile(yaml, path);
}
```

**Acceptance:**
- Can load any test recipe from `test/fixtures/yaml_recipes/`
- Returns null with clear error for invalid files

---

#### 1.3: Add argument parsing for recipe-specific flags

**Goal:** Parse command-line arguments and pass them to the recipe.

**Implementation:**
```dart
// Use args package for parsing
final parser = ArgParser()
  ..addOption('file', abbr: 'f', help: 'Path to the Dart file to modify')
  ..addOption('className', help: 'Name of the class')
  ..addOption('methodName', help: 'Name of the method')
  ..addFlag('apply', abbr: 'a', negatable: false, help: 'Apply changes (default is dry-run)')
  ..addFlag('help', abbr: 'h', negatable: false, help: 'Show usage');

final results = parser.parse(arguments);

// Extract recipe-specific args
final recipeArgs = {
  'file': results['file'] as String?,
  'className': results['className'] as String?,
  'methodName': results['methodName'] as String?,
  'apply': results['apply'] as bool?,
};
```

**Acceptance:**
- All recipe arguments are parsed correctly
- Help flag displays usage information

---

#### 1.4: Add --help flag with usage information

**Implementation:**
```dart
if (results['help'] == true) {
  print('''
Usage: dart run bin/codemod.dart <recipe-file.yaml> [options]

Arguments:
  <recipe-file.yaml>    Path to the YAML recipe file

Options:
  -f, --file           Path to the Dart file to modify
  --className          Name of the class
  --methodName         Name of the method
  --type               Type for template variables
  -a, --apply          Apply changes (default: dry-run)
  -h, --help           Show this help message

Examples:
  dart run bin/codemod.dart add_counter_field.yaml --file lib/main.dart --className MyClass
  dart run bin/codemod.dart add_log_line.yaml --file lib/main.dart --className MyClass --methodName myMethod --apply
''');
  exit(0);
}
```

**Acceptance:**
- Help displays correctly
- Includes examples

---

#### 1.5: Add --map-root flag for custom map directory

**Implementation:**
```dart
..addOption('map-root', 
    abbr: 'm', 
    help: 'Directory containing map files (default: .codemod/maps)',
    defaultsTo: '.codemod/maps');
```

**Acceptance:**
- Maps can be loaded from custom directory
- Defaults to standard location

---

#### 1.6: Add dry-run and --apply support

**Implementation:**
```dart
final apply = results['apply'] as bool? ?? false;

final runner = CodemodRunner(recipe, preferences: config.preferences);

if (apply) {
  await runner.run(args);
} else {
  await runner.run(args, dryRun: true);
}
```

**Acceptance:**
- Default is dry-run (preview)
- `--apply` or `-a` applies changes

---

#### 1.7: Add error handling and validation

**Implementation:**
```dart
try {
  final recipe = await loadRecipe(recipePath);
  if (recipe == null) {
    stderr.writeln('Failed to load recipe: $recipePath');
    exit(1);
  }
  
  // Validate required args
  final missingArgs = <String>[];
  for (final arg in recipe.args.where((a) => a.required)) {
    if (!recipeArgs.containsKey(arg.name)) {
      missingArgs.add(arg.name);
    }
  }
  
  if (missingArgs.isNotEmpty) {
    stderr.writeln('Missing required arguments: ${missingArgs.join(", ")}');
    exit(1);
  }
  
  // Run the recipe
  await runner.run(args);
  
} on FormatException catch (e) {
  stderr.writeln('Invalid YAML: ${e.message}');
  exit(1);
} on AstPathResolutionException catch (e) {
  stderr.writeln('AST path error: ${e.message}');
  exit(1);
} on CodemodException catch (e) {
  stderr.writeln('Codemod error: ${e.message}');
  exit(1);
}
```

**Acceptance:**
- Clear error messages for all failure cases
- Non-zero exit codes on errors

---

## Phase 2: Refactor codemod_host.dart

### High Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-2.1 | Remove CLI mode logic | 🔴 High | Phase 1 complete | 1 hour | Only stdio-server and validate modes remain |

### Medium Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-2.2 | Remove `_separateHostAndRecipeArgs` helper | 🟡 Medium | 2.1 | 30 min | Code is cleaner |
| option-a-2.3 | Update documentation comments | 🟡 Medium | 2.1 | 30 min | Comments reflect VS Code-only purpose |

### Low Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-2.4 | Add deprecation warning for CLI usage | 🟢 Low | 2.1 | 30 min | Users guided to `codemod.dart` |

### Task Details

#### 2.1: Remove CLI mode logic

**Current code to remove:**
- The `_separateHostAndRecipeArgs` function
- The recipe argument parsing logic in main
- The direct recipe execution path (lines that call `CodemodRunner` directly)

**Keep:**
- `--stdio-server` mode
- `--validate` mode
- `--help` flag
- Host configuration flags (`--workspace-root`, `--codemod-root`, etc.)

**Resulting structure:**
```dart
Future<void> main(List<String> arguments) async {
  final parser = HostConfig.buildArgParser()..addFlag('help', abbr: 'h');
  final results = parser.parse(arguments);
  
  if (results['help'] == true) { _printUsage(); exit(0); }
  
  final config = HostConfig.fromArgResults(results);
  
  if (results['validate'] == true) { /* validate logic */ }
  
  if (results['stdio-server'] == true || _looksLikeJsonCommand(arguments)) {
    await CodemodHost.fromConfig(config).run(arguments);
    return;
  }
  
  // No CLI mode - direct users to codemod.dart
  stderr.writeln('For CLI usage, use: dart run bin/codemod.dart <recipe.yaml> [args]');
  stderr.writeln('For VS Code extension, use: dart run bin/codemod_host.dart --stdio-server');
  exit(1);
}
```

**Acceptance:**
- Only stdio-server mode works
- Direct recipe execution removed
- Clear error message for CLI users

---

## Phase 3: Testing & Documentation

### High Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-3.1 | Test bin/codemod.dart with existing test recipes | 🔴 High | Phase 1 complete | 1 hour | All test recipes work |
| option-a-3.2 | Test bin/codemod_host.dart in stdio-server mode | 🔴 High | Phase 2 complete | 1 hour | VS Code extension still works |
| option-a-3.5 | Verify launch configuration works | 🔴 High | 3.1, 3.2 | 30 min | Your debug scenario is fixed |

### Medium Priority Tasks

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-3.3 | Update CONTRIBUTING.md with new CLI usage | 🟡 Medium | 3.1 | 30 min | Documentation is accurate |
| option-a-3.4 | Update README.md with both entry points | 🟡 Medium | 3.3 | 30 min | Users know which to use |

---

## Phase 4: Cleanup (Optional)

| ID | Task | Priority | Dependencies | Estimated Time | Acceptance Criteria |
|----|------|----------|--------------|----------------|---------------------|
| option-a-4.1 | Create migration guide for CLI users | 🟢 Low | Phase 3 complete | 1 hour | Users can update their scripts |
| option-a-4.2 | Define deprecation timeline | 🟢 Low | 4.1 | 30 min | Clear communication |

---

## Execution Order

### Recommended Sequence

1. **Phase 1** (Create new CLI): Tasks 1.1 → 1.2 → 1.3 → 1.7 → 1.4 → 1.5 → 1.6
2. **Phase 2** (Refactor host): Tasks 2.1 → 2.2 → 2.3 → 2.4
3. **Phase 3** (Test & Document): Tasks 3.1 → 3.2 → 3.5 → 3.3 → 3.4
4. **Phase 4** (Cleanup): Tasks 4.1 → 4.2

### Parallel Work
- Phases 1 and 2 can be worked on in parallel by different people
- Phase 3 requires Phases 1 and 2 to be complete

---

## Testing Strategy

### Unit Tests
- Add tests for new argument parsing in `bin/codemod.dart`
- Verify existing tests still pass after refactoring

### Integration Tests
- Test with all existing YAML recipes
- Test error cases (missing files, invalid YAML, missing args)

### Manual Testing
- Test with your launch configuration
- Test with various recipe types
- Test dry-run and apply modes

---

## Rollback Plan

If issues arise:
1. The old `codemod_host.dart` behavior is preserved until Phase 2
2. Users can continue using the old CLI mode until migration is complete
3. New `codemod.dart` is additive, doesn't break existing functionality

---

## Success Criteria

✅ **Phase 1 Complete:**
- `bin/codemod.dart` can load and run a single recipe
- All recipe arguments are parsed and passed correctly
- Error handling is robust

✅ **Phase 2 Complete:**
- `bin/codemod_host.dart` only supports stdio-server and validate modes
- Code is cleaner and more focused

✅ **Phase 3 Complete:**
- Both entry points are tested
- Documentation is updated
- Your debugging scenario works

✅ **Phase 4 Complete:**
- Migration guide exists
- Deprecation timeline is defined

---

## Dependencies

| Task | Depends On | Blocks |
|------|------------|--------|
| option-a-1.1 | None | option-a-1.2, option-a-1.3 |
| option-a-1.2 | 1.1 | option-a-1.3, option-a-1.7 |
| option-a-1.3 | 1.2 | option-a-1.4, option-a-1.5, option-a-1.6 |
| option-a-1.4 | 1.3 | None |
| option-a-1.5 | 1.3 | None |
| option-a-1.6 | 1.3 | None |
| option-a-1.7 | 1.2 | None |
| option-a-2.1 | Phase 1 | option-a-2.2, option-a-2.3, option-a-2.4 |
| option-a-2.2 | 2.1 | None |
| option-a-2.3 | 2.1 | None |
| option-a-2.4 | 2.1 | None |
| option-a-3.1 | Phase 1 | option-a-3.5 |
| option-a-3.2 | Phase 2 | None |
| option-a-3.3 | 3.1 | None |
| option-a-3.4 | 3.3 | None |
| option-a-3.5 | 3.1, 3.2 | None |

---

## Resources

- [Dart Command-Line Apps](https://dart.dev/tutorials/server/cmdline)
- [args package documentation](https://pub.dev/packages/args)
- [Dart File I/O](https://dart.dev/guides/libraries/library-tour#files-and-directories)

---

## Version History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-17 | Vibe | Initial plan created |
