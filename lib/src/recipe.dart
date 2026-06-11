import 'context.dart';
import 'operation.dart';
import 'post_execution.dart';
import 'step.dart';
import 'template.dart';

/// Preferred UI control for a recipe argument.
enum CodemodArgInputKind {
  /// Plain text input.
  text,

  /// File path input, ideally backed by a file picker.
  file,

  /// Directory path input, ideally backed by a folder picker.
  directory,

  /// Dropdown or combobox with enum-like values.
  enumeration,

  /// Dart type input, usually backed by suggestions but allowing custom values.
  dartType,

  /// Symbol name input such as class, method, field, or variable.
  symbol,
}

/// Well-known editor context values that can pre-fill recipe arguments.
class CodemodContextKey {
  /// Relative path of the active editor file.
  static const file = 'file';

  /// Selected text in the active editor.
  static const selection = 'selection';

  /// Word under the active editor cursor.
  static const word = 'word';

  /// Enclosing Dart class name at the active editor cursor.
  static const dartClass = 'dartClass';

  const CodemodContextKey._();
}

/// Describes one command-line value accepted by a codemod recipe.
///
/// Arguments define the CLI interface for a codemod. Each argument can be
/// required or optional, with optional support for default values and
/// custom validation.
///
/// ## Example
///
/// ```dart
/// final args = [
///   CodemodArg.required('file', help: 'Path to the Dart file'),
///   CodemodArg.required('class', abbr: 'c', help: 'Class name'),
///   CodemodArg.optional(
///     'format',
///     defaultsTo: 'true',
///     help: 'Whether to format after changes',
///   ),
/// ];
/// ```
class CodemodArg {
  /// Long option name, without the leading `--`.
  ///
  /// This is the primary way users specify the argument on the command line.
  /// Must be a valid Dart identifier (letters, numbers, underscores, starting
  /// with a letter or underscore).
  final String name;

  /// Optional one-character abbreviation for CLI usage.
  ///
  /// When provided, users can specify the argument with a single dash followed
  /// by this character (e.g., `-f` for a file argument).
  final String? abbr;

  /// User-facing help text shown by the runner.
  ///
  /// Displayed in the `--help` output to explain what this argument is for.
  final String? help;

  /// Whether the runner should reject missing or empty values.
  ///
  /// When true, the codemod will fail if this argument is not provided.
  final bool required;

  /// Default value passed to the CLI parser for optional args.
  ///
  /// Only used when [required] is false. The value is used when the user
  /// does not provide this argument on the command line.
  final String? defaultsTo;

  /// Preferred UI control for this argument in editor integrations.
  final CodemodArgInputKind inputKind;

  /// Suggested values shown by editor integrations.
  ///
  /// When non-empty, the extension should present these as dropdown or
  /// combobox suggestions.
  final List<String> options;

  /// Whether editor integrations should allow values outside [options].
  final bool allowCustomValue;

  /// Optional key used to pre-fill this argument from active editor context.
  ///
  /// Common keys are exposed via [CodemodContextKey].
  final String? contextKey;

  /// Optional validation hook run after CLI values are collected.
  ///
  /// Return a non-null string to report an error message. Return null to
  /// indicate the value is valid.
  ///
  /// ## Example
  ///
  /// ```dart
  /// CodemodArg(
  ///   name: 'email',
  ///   validate: (value, context) {
  ///     if (value != null && !value.contains('@')) {
  ///       return 'Invalid email address';
  ///     }
  ///     return null;
  ///   },
  /// )
  /// ```
  final String? Function(String? value, CodemodContext context)? validate;

  /// Creates a custom argument descriptor.
  const CodemodArg({
    required this.name,
    this.abbr,
    this.help,
    this.required = false,
    this.defaultsTo,
    this.inputKind = CodemodArgInputKind.text,
    this.options = const [],
    this.allowCustomValue = true,
    this.contextKey,
    this.validate,
  });

  /// Creates a required command-line option.
  ///
  /// The codemod will fail execution if this argument is not provided.
  ///
  /// ## Example
  ///
  /// ```dart
  /// CodemodArg.required('file', help: 'Path to the Dart file')
  /// ```
  const CodemodArg.required(
    this.name, {
    this.abbr,
    this.help,
    this.inputKind = CodemodArgInputKind.text,
    this.options = const [],
    this.allowCustomValue = true,
    this.contextKey,
    this.validate,
  }) : required = true,
       defaultsTo = null;

  /// Creates an optional command-line option.
  ///
  /// Use [defaultsTo] to specify a default value when the user does not
  /// provide this argument.
  ///
  /// ## Example
  ///
  /// ```dart
  /// CodemodArg.optional(
  ///   'output',
  ///   defaultsTo: 'lib/generated',
  ///   help: 'Output directory',
  /// )
  /// ```
  const CodemodArg.optional(
    this.name, {
    this.abbr,
    this.help,
    this.defaultsTo,
    this.inputKind = CodemodArgInputKind.text,
    this.options = const [],
    this.allowCustomValue = true,
    this.contextKey,
    this.validate,
  }) : required = false;
}

/// User-facing template preview metadata for editor integrations.
///
/// This is intentionally explicit because recipes can perform arbitrary
/// operation logic, so integrations cannot reliably infer useful templates from
/// operation closures.
class RecipeTemplatePreview {
  /// Label shown above this template preview.
  final String label;

  /// Target path template shown with rendered placeholder values.
  final String path;

  /// Template content shown with rendered placeholder values.
  final CodemodTemplate content;

  /// Creates a template preview descriptor.
  const RecipeTemplatePreview({
    required this.label,
    required this.path,
    required this.content,
  });
}

/// A complete codemod command made of arguments and target files.
///
/// A recipe declares what arguments it accepts, what file operations to
/// perform, and what actions to run after successful completion. Recipes can
/// be composed together to build complex workflows from simpler parts.
///
/// ## Example
///
/// ```dart
/// final recipe = CodemodRecipe(
///   name: 'add_method',
///   description: 'Adds a method to a Dart class',
///   args: [
///     CodemodArg.required('file', help: 'Path to the file'),
///     CodemodArg.required('class', help: 'Class name'),
///   ],
///   operations: [
///     EditDartFileOperation(
///       path: (context) => context.require('file'),
///       transforms: (context) => [
///         AddMethodTransform(
///           className: (c) => c.require('class'),
///           methodName: (_) => 'newMethod',
///           body: const CodemodTemplate.inline('void newMethod() {}'),
///         ),
///       ],
///     ),
///   ],
///   postExecution: const [DartFormatPostExecution()],
/// );
/// ```
class CodemodRecipe with CodemodStep {
  /// Stable command name used in help output.
  ///
  /// This should be a short, descriptive identifier without spaces.
  final String name;

  /// User-facing summary shown above CLI usage.
  ///
  /// Displayed in the `--help` output to explain what this recipe does.
  final String description;

  /// Command-line options accepted by this recipe.
  ///
  /// These define the arguments users can pass to configure the codemod.
  final List<CodemodArg> args;

  /// Ordered file operations this recipe applies.
  ///
  /// Operations are executed in the order they appear in this list. Each
  /// operation can read the current state of files and plan changes.
  final List<CodemodOperation> operations;

  /// Actions to run after changes are successfully applied.
  ///
  /// These only run when the codemod is executed with `--apply`. Common
  /// actions include formatting code (`DartFormatPostExecution`) or
  /// running code generation (`BuildRunnerPostExecution`).
  final List<PostExecution> postExecution;

  /// Optional templates shown by editor integrations before preview/apply.
  final List<RecipeTemplatePreview> previewTemplates;

  /// Creates a recipe from explicit arguments and target file edits.
  const CodemodRecipe({
    required this.name,
    this.description = '',
    this.args = const [],
    required this.operations,
    this.postExecution = const [],
    this.previewTemplates = const [],
  });


  /// Creates a recipe from an ordered mix of recipes, operations, and
  /// post-execution actions.
  ///
  /// [steps] may contain [CodemodRecipe], [CodemodOperation], and
  /// [PostExecution] values in any order. Recipe steps contribute their
  /// arguments, operations, post-execution actions, and template previews.
  /// Operation and post-execution steps are appended directly.
  ///
  /// Arguments are merged by name, with explicit [args] taking precedence over
  /// definitions from recipe steps. When recipe steps define the same argument,
  /// the first definition encountered in [steps] is kept.
  ///
  /// Post-execution steps control ordering in the composed recipe's
  /// [postExecution] list. All post-execution actions still run after every
  /// file change has been applied.
  ///
  /// ## Example
  ///
  /// ```dart
  /// final composed = CodemodRecipe.compose(
  ///   name: 'enhance_class',
  ///   steps: [
  ///     addImportRecipe,
  ///     EditDartFileOperation(...),
  ///     const DartFormatPostExecution(),
  ///     addMethodRecipe,
  ///   ],
  /// );
  /// ```
  factory CodemodRecipe.compose({
    required String name,
    String description = '',
    List<CodemodArg> args = const [],
    required List<CodemodStep> steps,
    List<RecipeTemplatePreview> previewTemplates = const [],
  }) {
    final mergedArgs = <String, CodemodArg>{};
    for (final arg in args) {
      mergedArgs[arg.name] = arg;
    }

    final operations = <CodemodOperation>[];
    final postExecution = <PostExecution>[];
    final mergedPreviewTemplates = <RecipeTemplatePreview>[];

    for (final step in steps) {
      for (final arg in step.args) {
            mergedArgs.putIfAbsent(arg.name, () => arg);
          }
          operations.addAll(step.operations);
          postExecution.addAll(step.postExecution);
          mergedPreviewTemplates.addAll(step.previewTemplates);
    }

    return CodemodRecipe(
      name: name,
      description: description,
      args: mergedArgs.values.toList(),
      operations: operations,
      postExecution: postExecution,
      previewTemplates: [...mergedPreviewTemplates, ...previewTemplates],
    );
  }
}
