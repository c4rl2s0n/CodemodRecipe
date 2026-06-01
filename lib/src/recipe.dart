import 'context.dart';
import 'operation.dart';
import 'post_execution.dart';

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
  const CodemodArg.required(this.name, {this.abbr, this.help, this.validate})
    : required = true,
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
    this.validate,
  }) : required = false;
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
class CodemodRecipe {
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

  /// Creates a recipe from explicit arguments and target file edits.
  const CodemodRecipe({
    required this.name,
    this.description = '',
    this.args = const [],
    required this.operations,
    this.postExecution = const [],
  });

  /// Creates a recipe by concatenating other recipes.
  ///
  /// Composing recipes allows building complex workflows from simpler parts.
  /// Arguments are merged by name, with explicit [args] taking precedence
  /// over definitions from composed recipes. When composed recipes define the
  /// same argument, the first recipe's definition is kept. Operations are
  /// concatenated in recipe order.
  ///
  /// ## Example
  ///
  /// ```dart
  /// // Define shared arguments
  /// final fileArg = CodemodArg.required('file');
  /// final classArg = CodemodArg.required('class');
  ///
  /// // Create individual recipes
  /// final addImportRecipe = CodemodRecipe(
  ///   name: 'add_import',
  ///   args: [fileArg],
  ///   operations: [/* ... */],
  /// );
  ///
  /// final addMethodRecipe = CodemodRecipe(
  ///   name: 'add_method',
  ///   args: [fileArg, classArg],
  ///   operations: [/* ... */],
  /// );
  ///
  /// // Compose them into a single recipe
  /// final composed = CodemodRecipe.compose(
  ///   name: 'enhance_class',
  ///   recipes: [addImportRecipe, addMethodRecipe],
  /// );
  ///
  /// // The composed recipe has 2 args: file, class
  /// // The composed recipe has 2 operations: add import, add method
  /// ```
  ///
  /// Argument definitions are merged by name, with explicit [args] taking
  /// precedence over definitions from composed recipes.
  factory CodemodRecipe.compose({
    required String name,
    String description = '',
    List<CodemodArg> args = const [],
    required List<CodemodRecipe> recipes,
    List<PostExecution> postExecution = const [],
  }) {
    final mergedArgs = <String, CodemodArg>{};
    for (final arg in args) {
      mergedArgs[arg.name] = arg;
    }
    for (final recipe in recipes) {
      for (final arg in recipe.args) {
        mergedArgs.putIfAbsent(arg.name, () => arg);
      }
    }

    return CodemodRecipe(
      name: name,
      description: description,
      args: mergedArgs.values.toList(),
      operations: [for (final recipe in recipes) ...recipe.operations],
      postExecution: [
        for (final recipe in recipes) ...recipe.postExecution,
        ...postExecution,
      ],
    );
  }
}
