/// Public API for building deterministic, recipe-based Dart codemods.
///
/// Import this library from project-specific codemod entrypoints and transforms.
/// Keep project naming conventions and domain-specific transforms outside the
/// package.
library;

export 'src/core/arg_codec.dart';
export 'src/core/args.dart';
export 'src/core/constants.dart';
export 'src/core/context.dart';
export 'src/core/errors.dart';
export 'src/core/logging.dart';
export 'src/core/operation.dart';
export 'src/core/patch_helpers.dart';
export 'src/core/post_execution.dart';
export 'src/core/recipe.dart';
export 'src/core/runner.dart';
export 'src/core/step.dart';
export 'src/core/template.dart';
export 'src/core/transform.dart';
export 'src/core/utils/file_utils.dart';
export 'src/ast_path/ast_path.dart';
export 'src/yaml/yaml.dart';
export 'src/dart_codegen/ast_helpers/ast_helpers.dart';
export 'src/dart_codegen/code_editor.dart';
export 'src/dart_codegen/field_spec.dart';
export 'src/dart_codegen/naming.dart';
