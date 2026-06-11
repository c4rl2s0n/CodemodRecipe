/// Public API for building deterministic, recipe-based Dart codemods.
///
/// Import this library from project-specific codemod entrypoints and transforms.
/// Keep project naming conventions and domain-specific transforms outside the
/// package.
library;

export 'src/arg_codec.dart';
export 'src/context.dart';
export 'src/dart_codegen/ast_helpers/ast_helpers.dart';
export 'src/dart_codegen/code_editor.dart';
export 'src/dart_codegen/field_spec.dart';
export 'src/dart_codegen/naming.dart';
export 'src/generic/post_execution.dart';
export 'src/generic/transforms.dart';
export 'src/operation.dart';
export 'src/patch_helpers.dart';
export 'src/post_execution.dart';
export 'src/recipe.dart';
export 'src/runner.dart';
export 'src/step.dart';
export 'src/template.dart';
export 'src/transform.dart';
