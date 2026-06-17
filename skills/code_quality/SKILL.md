---
name: code_quality
description: >-
  Guidelines and best practices for writing maintainable, clean Dart code in the
  codemod_recipe project. Covers SOLID principles, DRY, KISS, and Dart-specific
  conventions. Use this skill when contributing to ensure consistent code quality.
---

# Code Quality Guidelines for Codemod Recipe

> **Applies to:** All Dart code in the codemod_recipe package
> **When to use:** Code reviews, new feature development, refactoring

## 🎯 Core Principles

### 1. SOLID Design Principles

#### Single Responsibility Principle (SRP)
**Guideline:** Each class, function, or module should have exactly one reason to change.

**✅ Do:**
```dart
// Each class has a single, clear responsibility
class AstPathParser {
  // Only parsing logic
  static AstPath parsePathString(String input) { ... }
}

class AstPathValidator {
  // Only validation logic
  static bool isValid(AstPath path) { ... }
}
```

**❌ Don't:**
```dart
// Class does too much
class AstPathManager {
  // Parsing
  static AstPath parse(String input) { ... }
  
  // Validation
  static bool validate(AstPath path) { ... }
  
  // Resolution
  static int resolve(AstPath path) { ... }
  
  // Formatting
  static String format(AstPath path) { ... }
}
```

**Checklist:**
- [ ] Class name clearly describes its single responsibility
- [ ] Methods are cohesive (all serve the same purpose)
- [ ] No "god classes" with many unrelated methods

---

#### Open/Closed Principle (OCP)
**Guideline:** Software entities should be open for extension but closed for modification.

**✅ Do:**
```dart
// Strategy pattern allows adding new anchors without modifying validation
abstract class AnchorValidator {
  bool isValid(AstNode node, Anchor anchor);
}

class StmtLastValidator extends AnchorValidator {
  @override
  bool isValid(AstNode node, Anchor anchor) => node is MethodDeclaration;
}

class BodyEndValidator extends AnchorValidator {
  @override
  bool isValid(AstNode node, Anchor anchor) => node is ClassDeclaration;
}

// Registry of validators
final validators = {
  AnchorKind.stmtLast: StmtLastValidator(),
  AnchorKind.bodyEnd: BodyEndValidator(),
};
```

**❌ Don't:**
```dart
// Adding new anchor requires modifying this function
bool isAnchorValidFor(AstNode node, Anchor anchor) {
  if (anchor.kind == AnchorKind.stmtLast) return node is MethodDeclaration;
  if (anchor.kind == AnchorKind.bodyEnd) return node is ClassDeclaration;
  if (anchor.kind == AnchorKind.paramLast) return node is ConstructorDeclaration;
  // ... new anchor requires adding another if statement
}
```

**Checklist:**
- [ ] New functionality can be added without modifying existing code
- [ ] Use abstractions (abstract classes, interfaces) for extensibility
- [ ] Prefer composition over inheritance

---

#### Liskov Substitution Principle (LSP)
**Guideline:** Subtypes must be substitutable for their base types without breaking behavior.

**✅ Do:**
```dart
// All NavigateStep implementations can be used interchangeably
abstract class NavigateStepResolver {
  AstFocus resolve(AstFocus focus, NavigateStep step);
}

class ClassResolver extends NavigateStepResolver { ... }
class MethodResolver extends NavigateStepResolver { ... }

// Client code doesn't need to know the concrete type
AstFocus resolveStep(AstFocus focus, NavigateStep step) {
  final resolver = getResolver(step.kind);
  return resolver.resolve(focus, step);
}
```

**Checklist:**
- [ ] Subclasses don't weaken preconditions
- [ ] Subclasses don't strengthen postconditions
- [ ] All implementations of an interface are usable in the same way

---

#### Interface Segregation Principle (ISP)
**Guideline:** Clients should not be forced to depend on interfaces they do not use.

**✅ Do:**
```dart
// Small, focused interfaces
abstract class CanSerialize {
  Map<String, dynamic> toJson();
}

abstract class CanDeserialize<T> {
  T fromJson(Map<String, dynamic> json);
}

// Client only needs serialization
class JsonWriter {
  void write(CanSerialize obj) { ... }
}
```

**❌ Don't:**
```dart
// Large interface with methods not all clients need
abstract class Serializable {
  Map<String, dynamic> toJson();
  T fromJson<T>(Map<String, dynamic> json);
  String toYaml();
  T fromYaml<T>(String yaml);
}
```

**Checklist:**
- [ ] Interfaces are small and focused
- [ ] Clients only depend on methods they actually use
- [ ] No "fat" interfaces with many unrelated methods

---

#### Dependency Inversion Principle (DIP)
**Guideline:** High-level modules should not depend on low-level modules. Both should depend on abstractions.

**✅ Do:**
```dart
// High-level module depends on abstraction
class AstPathInterpreter {
  final AstFocusFactory focusFactory;
  
  AstPathInterpreter(this.focusFactory);
  
  AstFocus navigateTo(String source, List<NavigateStep> steps) {
    var focus = focusFactory.create(source);
    // ...
  }
}

// Low-level implementation
class AstFocusFactoryImpl implements AstFocusFactory {
  AstFocus create(String source) => AstFocus.parse(source);
}

// Injection at composition root
final interpreter = AstPathInterpreter(AstFocusFactoryImpl());
```

**❌ Don't:**
```dart
// High-level module directly instantiates low-level class
class AstPathInterpreter {
  AstFocus navigateTo(String source, List<NavigateStep> steps) {
    var focus = AstFocus.parse(source); // Direct dependency on concrete class
    // ...
  }
}
```

**Checklist:**
- [ ] High-level modules depend on abstractions (interfaces/abstract classes)
- [ ] Dependency injection is used for concrete implementations
- [ ] No `new` keywords in high-level modules for low-level types

---

## 🔄 DRY (Don't Repeat Yourself)

**Guideline:** Avoid code duplication. If you write the same logic twice, extract it.

### Centralize Magic Values

**✅ Do:**
```dart
// constants.dart
const kDefaultCodemodRoot = '.codemod';
const kDefaultWorkspaceRoot = '.';
const kMaxParameterCount = 10;
const kDefaultTimeoutSeconds = 30;

// Usage
final codemodRoot = config.codemodRoot ?? kDefaultCodemodRoot;
```

**❌ Don't:**
```dart
// Same value repeated in multiple places
final codemodRoot = config.codemodRoot ?? '.codemod';
// ... later in another file
final defaultRoot = '.codemod';
```

### Extract Common Logic

**✅ Do:**
```dart
// In a utility file
extension StringExtensions on String {
  String toCamelCase() => ...;
  String toSnakeCase() => ...;
  String toPascalCase() => ...;
}

// Usage anywhere
final camelName = 'my_variable'.toCamelCase();
```

**❌ Don't:**
```dart
// Same transformation logic in multiple places
final name1 = input.replaceAll('_', ' ').split(' ').map(...).join();
// ... later
final name2 = other.replaceAll('_', ' ').split(' ').map(...).join();
```

### Shared Validation Logic

**✅ Do:**
```dart
// validators.dart
bool isValidIdentifier(String identifier) {
  return identifier.isNotEmpty && 
         RegExp(r'^[a-zA-Z_][a-zA-Z0-9_]*$').hasMatch(identifier);
}

// Usage in multiple places
if (!isValidIdentifier(className)) throw ...;
```

**Checklist:**
- [ ] Magic strings/numbers are in `constants.dart`
- [ ] Common utility functions are extracted to utility classes
- [ ] Duplicate code is eliminated (2+ instances = extract)
- [ ] Extension methods are used for common type operations

---

## 😌 KISS (Keep It Simple, Stupid)

**Guideline:** Prefer simplicity over cleverness. Code should be easy to understand.

### Method Complexity

**✅ Do:**
```dart
// Simple, focused method
int calculateOffset(AstNode node) {
  if (node is MethodDeclaration) {
    return findLastStatementOffset(node);
  }
  return node.offset;
}
```

**❌ Don't:**
```dart
// Complex, hard to understand
int calculateOffset(AstNode node) => node is MethodDeclaration
    ? node.body is BlockFunctionBody
        ? node.body.block.statements.isNotEmpty
            ? node.body.block.statements.last.end
            : node.body.block.leftBracket.end
        : node.body.offset
    : node is ClassDeclaration
        ? ... // nested ternary
        : node.offset;
```

### Avoid Over-Engineering

**✅ Do:**
```dart
// Simple solution for current needs
class RecipeRegistry {
  final Map<String, YamlRecipe> _recipes = {};
  
  void addRecipe(YamlRecipe recipe) {
    _recipes[recipe.id] = recipe;
  }
  
  YamlRecipe? getRecipe(String id) => _recipes[id];
}
```

**❌ Don't:**
```dart
// Over-engineered for future needs that may never come
abstract class RecipeRepository<T extends Recipe> {
  final Map<String, T> _store;
  final CacheStrategy<T> _cache;
  final ValidationStrategy<T> _validator;
  
  RecipeRepository(this._store, this._cache, this._validator);
  
  Future<Either<ValidationError, T>> addAsync(T recipe) async { ... }
}
```

### Clear Naming

**✅ Do:**
```dart
// Names clearly indicate purpose
class AstPathInterpreter {
  int resolveOffset(String source, AstPath path) { ... }
}

class AnchorResolver {
  AnchorSpan resolveSpan({required String source, required AstNode node, required Anchor anchor}) { ... }
}
```

**❌ Don't:**
```dart
// Unclear names
class PathHandler {
  int process(String s, Path p) { ... }  // What does "process" do?
}

class AHelper {
  Span doStuff(Node n, Anchor a) { ... }  // What stuff?
}
```

**Checklist:**
- [ ] Methods do one thing and do it well
- [ ] No unnecessary abstraction
- [ ] Names clearly indicate purpose and behavior
- [ ] Code is readable without comments explaining what it does

---

## 🎨 Dart-Specific Best Practices

### Null Safety

**✅ Do:**
```dart
// Proper null safety
String getName(CodemodContext context) {
  final name = context['name'];
  if (name == null) {
    throw ArgumentError('name is required');
  }
  return name;
}

// Or with required
String getName(CodemodContext context) => context.require('name');
```

**❌ Don't:**
```dart
// Unsafe null handling
String getName(CodemodContext context) => context['name']!; // Will crash if null
```

### Immutable Objects

**✅ Do:**
```dart
// Immutable data class
@immutable
class NavigateStep {
  final NavigateKind? kind;
  final String? name;
  final String? match;
  
  const NavigateStep(this.kind, {this.name, this.match});
  
  // Copy with modifications
  NavigateStep copyWith({NavigateKind? kind, String? name, String? match}) {
    return NavigateStep(kind ?? this.kind, name: name ?? this.name, match: match ?? this.match);
  }
}
```

**❌ Don't:**
```dart
// Mutable class with side effects
class NavigateStep {
  NavigateKind? kind;
  String? name;
  String? match;
  
  void setName(String newName) {
    name = newName;  // Mutable state
  }
}
```

### Error Handling

**✅ Do:**
```dart
// Custom exceptions with clear messages
class AstPathResolutionException implements Exception {
  final String message;
  final String? code;
  
  const AstPathResolutionException(this.message, {this.code});
  
  @override
  String toString() => code != null ? '[$code] $message' : message;
}

// Throw with context
throw AstPathResolutionException(
  'Class "$name" not found in source',
  code: 'E_NODE_NOT_FOUND',
);
```

**❌ Don't:**
```dart
// Generic error with no context
if (classDecl == null) {
  throw Exception('Not found');  // No useful information
}
```

### Collections

**✅ Do:**
```dart
// Use built-in collection methods
final classes = unit.declarations.whereType<ClassDeclaration>();
final names = classes.map((c) => c.name.lexeme).toList();
final hasMatch = names.contains('Settings');
```

**❌ Don't:**
```dart
// Manual iteration
final names = [];
for (final decl in unit.declarations) {
  if (decl is ClassDeclaration) {
    names.add(decl.name.lexeme);
  }
}
```

### Testing

**✅ Do:**
```dart
// Clear test structure
group('AstPathParser', () {
  test('parses class navigation', () {
    final path = parsePathString('class:Settings @ member:last');
    
    expect(path.navigate.length, 1);
    expect(path.navigate[0].kind, NavigateKind.classDecl);
    expect(path.navigate[0].name, 'Settings');
  });
  
  test('throws on empty path', () {
    expect(() => parsePathString(''), throwsA<AstPathParseException>());
  });
});
```

**❌ Don't:**
```dart
// Poor test structure
void testParser() {
  // No clear separation between test cases
  final path1 = parsePathString('class:A @ member:last');
  assert(path1.navigate[0].name == 'A');
  
  final path2 = parsePathString('method:B @ stmt:last');
  assert(path2.navigate[0].name == 'B');
  
  // No error handling in tests
}
```

---

## 📊 Code Review Checklist

Use this checklist when reviewing code:

### General
- [ ] Code follows Dart style guide (`dart format` passes)
- [ ] No static analysis warnings (`dart analyze` passes)
- [ ] All public APIs have DartDoc comments
- [ ] Code is properly organized (right file, right package)

### Design
- [ ] Single Responsibility Principle is followed
- [ ] Open/Closed Principle is followed
- [ ] Dependencies are inverted (DIP)
- [ ] Interfaces are segregated (ISP)
- [ ] Liskov Substitution is valid (LSP)

### Quality
- [ ] DRY: No code duplication
- [ ] KISS: Code is simple and readable
- [ ] Magic values are in constants
- [ ] Error messages are clear and actionable
- [ ] Null safety is properly used

### Testing
- [ ] New functionality has tests
- [ ] Tests cover edge cases
- [ ] Tests are fast and deterministic
- [ ] Tests follow best practices

### Documentation
- [ ] Public APIs have documentation
- [ ] Complex logic has inline comments
- [ ] Architecture decisions are documented
- [ ] Examples are provided for non-obvious usage

---

## 🔧 Tooling

### Formatting
```bash
# Format all Dart files
dart format .
```

### Static Analysis
```bash
# Analyze code for issues
dart analyze
```

### Testing
```bash
# Run all tests
dart test

# Run specific test
dart test test/path/to/test.dart

# Run with coverage
dart test --coverage=coverage
```

### Code Metrics
```bash
# Check cyclomatic complexity, lines of code, etc.
dart pub global activate dart_code_metrics
.dart_code_metrics analyze
```

---

## 📚 Resources

- [Dart Style Guide](https://dart.dev/guides/language/effective-dart/style)
- [Effective Dart](https://dart.dev/guides/language/effective-dart)
- [Clean Code Dart](https://github.com/williambarreiro/clean-code-dart)
- [SOLID Principles in Dart](https://medium.com/@solomongetachew112/best-8-practices-for-writing-clean-and-scalable-code-in-flutter-in-2024-3e9e36adf634)
- [Dart Design Patterns](https://github.com/isani/hybrid-dart/tree/master/design_patterns)

---

## 🎯 Quick Reference: Common Patterns

### Factory Pattern
```dart
class AstPath {
  final List<NavigateStep> navigate;
  final Anchor anchor;
  
  AstPath._(this.navigate, this.anchor);
  
  factory AstPath.parse(String input) => AstPathParser.parsePathString(input);
  
  factory AstPath.fromMap(Map<String, dynamic> map) => AstPathParser.parseStructuredPath(map);
}
```

### Builder Pattern
```dart
class AstPathBuilder {
  final List<NavigateStep> _steps = [];
  Anchor? _anchor;
  
  AstPathBuilder classStep(String name) {
    _steps.add(NavigateStep(NavigateKind.classDecl, name: name));
    return this;
  }
  
  AstPathBuilder methodStep(String name) {
    _steps.add(NavigateStep(NavigateKind.method, name: name));
    return this;
  }
  
  AstPathBuilder anchor(Anchor anchor) {
    _anchor = anchor;
    return this;
  }
  
  AstPath build() {
    if (_anchor == null) throw StateError('Anchor is required');
    return AstPath(_steps, _anchor!);
  }
}

// Usage
final path = AstPathBuilder()
  .classStep('Settings')
  .methodStep('update')
  .anchor(Anchor(AnchorKind.stmtLast))
  .build();
```

### Strategy Pattern
```dart
abstract class NavigationStrategy {
  AstFocus navigate(AstFocus focus, NavigateStep step);
}

class ClassNavigationStrategy extends NavigationStrategy {
  @override
  AstFocus navigate(AstFocus focus, NavigateStep step) => ...;
}

class MethodNavigationStrategy extends NavigationStrategy {
  @override
  AstFocus navigate(AstFocus focus, NavigateStep step) => ...;
}

// Registry
final strategies = {
  NavigateKind.classDecl: ClassNavigationStrategy(),
  NavigateKind.method: MethodNavigationStrategy(),
};
```

---

## ⚠️ Anti-Patterns to Avoid

### God Classes
```dart
// ❌ Avoid: Class does too much
class CodemodManager {
  // Parsing
  static parseRecipe(String yaml) { ... }
  
  // Validation
  static validateRecipe(Recipe recipe) { ... }
  
  // Execution
  static runRecipe(Recipe recipe) { ... }
  
  // File I/O
  static readFile(String path) { ... }
  static writeFile(String path, String content) { ... }
  
  // Network
  static fetchRemoteRecipe(String url) { ... }
}
```

### Static Class Abuse
```dart
// ❌ Avoid: Everything is static, no testability
class PathUtils {
  static int calculateOffset(AstNode node) { ... }
  static bool isValidPath(String path) { ... }
  static String normalizePath(String path) { ... }
}
```

### Deep Inheritance Hierarchies
```dart
// ❌ Avoid: Deep inheritance is hard to maintain
class BaseTransform { ... }
class CodeTransform extends BaseTransform { ... }
class FileTransform extends CodeTransform { ... }
class DartFileTransform extends FileTransform { ... }
class EditDartFileTransform extends DartFileTransform { ... }
```

### Mutable Global State
```dart
// ❌ Avoid: Global mutable state
var currentRecipeRegistry = RecipeRegistry();
var currentWorkspaceRoot = '.';
```

---

## Version History

| Date | Author | Changes |
|------|--------|---------|
| 2026-06-17 | Vibe | Initial skill created with SOLID, DRY, KISS guidelines |
