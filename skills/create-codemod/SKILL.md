---
name: create-codemod
description: >-
  Create deterministic, recipe-based Dart codemods using the codemod_recipe
  package. Use when adding codemod recipes, transforms, project integration
  helpers, or AST-guided Dart source edits.
---

# Creating Codemods With codemod_recipe

Use `package:codemod_recipe/codemod_recipe.dart` for reusable codemod infrastructure. Keep project-specific naming, paths, and domain transforms in the consuming project.

## Architecture

```text
CLI args -> CodemodContext -> CodemodRecipe -> CodemodOperation -> FileChange -> preview/apply -> PostExecution
```

## Recipe Entrypoint

Keep command files small:

```dart
import 'package:codemod_recipe/codemod_recipe.dart';

Future<void> main(List<String> args) {
  return CodemodRunner(myRecipe).run(args);
}

final myRecipe = CodemodRecipe(
  name: 'my_codemod',
  args: [
    CodemodArg.required('file'),
    CodemodArg.required('class'),
    CodemodArg.required('name'),
  ],
  operations: [
    EditDartFileOperation(
      path: (context) => context.require('file'),
      transforms: (context) => [
        MyTransform.fromContext(context),
      ],
    ),
  ],
  postExecution: const [DartFormatPostExecution()],
);
```

Do not reimplement CLI parsing, dry-run, apply, formatting, or error handling. Use `CodemodRunner`.

## Templates

Use `CodemodTemplate` for generated snippets and files. Templates use
Mustache-style placeholders plus package-defined casing filters:

```mustache
{{feature}} {{feature:snake}} {{feature:camel}} {{feature:pascal}}
```

Unknown variables and unsupported casing filters should fail the run. Use
`context.render(...)` for inline path templates.

## File Creation

Use `CreateFileOperation` for new files:

```dart
CreateFileOperation(
  path: (context) => context.render('lib/{{feature:snake}}/view.dart'),
  template: const CodemodTemplate.inline('class {{feature:pascal}}View {}'),
)
```

Default behavior is to fail when the file already exists.

## Transform Pattern

Transforms are pure source-to-patches operations:

```dart
class MyTransform implements CodeTransform {
  final String className;
  final String methodName;

  const MyTransform({
    required this.className,
    required this.methodName,
  });

  factory MyTransform.fromContext(CodemodContext context) {
    return MyTransform(
      className: context.require('class'),
      methodName: context.require('name'),
    );
  }

  @override
  Future<List<SourcePatch>> apply(
    String source,
    CodemodContext context,
  ) async {
    return CodeEditor(source)
        .inClass(className)
        .addMethodUnlessExists(methodName, '''
  void $methodName() {}
''')
        .patches;
  }
}
```

## Project Integration

Put project conventions in local extensions:

```dart
extension ProjectCodemodContext on CodemodContext {
  String get featureName => require('feature');
  String get featureFile => 'lib/features/${snake('feature')}.dart';
}
```

The reusable package must not contain consuming-project concepts, paths, generated snippets, or transforms.

## Composition

Prefer `CodemodRecipe.compose` for workflows that combine recipes, inline
operations, and post-execution actions in one ordered list:

```dart
final composedRecipe = CodemodRecipe.compose(
  name: 'add_feature_property',
  args: sharedArgs,
  steps: [
    addStatePropertyRecipe,
    addSetterRecipe,
    addModelPropertyRecipe,
    const DartFormatPostExecution(),
  ],
);
```

`steps` accepts any mix of `CodemodRecipe`, `CodemodOperation`, and
`PostExecution` values via the shared `CodemodStep` marker interface.

## Tests

Package tests should cover generic behavior only:
- patch ordering
- overlap validation
- recipe composition
- template rendering
- file operations
- post-execution actions
- `CodeEditor` primitives

Project tests should cover project-specific transforms and composed project recipes.

## Idempotency

Every transform should be safe to run repeatedly. Prefer the `UnlessExists`
helpers on `CodeEditor` and write tests that apply a transform, apply the
patches, then run the same transform against the generated source.

```dart
final patches = await transform.apply(source, context);
final generated = applyPatches(source, patches);

expect(await transform.apply(generated, context), isEmpty);
```

## Troubleshooting

- `Required variable "name" is not set`: verify the recipe declares a matching
  `CodemodArg` and the user passes `--name=value`.
- `File not found`: ensure `EditDartFileOperation.path` resolves relative to
  the working directory where the codemod is executed.
- `File already exists`: use `FileExistsStrategy.skip` or
  `FileExistsStrategy.overwrite` only when the recipe deliberately supports
  re-running scaffold operations.
- `Overlapping patches`: split transforms so they do not replace the same
  source range, or combine nearby edits into one `SourcePatch`.
- `Class "X" not found in source`: make sure the requested class name is the
  actual declaration name, not a generated file name or path segment.

## Testing Patterns

Use temporary directories for file operations:

```dart
final tempDir = await Directory.systemTemp.createTemp('codemod_');
addTearDown(() => tempDir.delete(recursive: true));
```

Test generated files by collecting changes, checking `preview()`, applying the
change, and reading the resulting file. Test transforms directly by applying
patches to a source string.

## Performance

Keep transforms local to a single file and avoid scanning unrelated directories
inside `CodeTransform.apply`. Resolve target paths in operations, then let each
transform inspect only the source string it receives. For large workflows,
compose small recipes instead of building one transform that understands the
entire project.
