import '../operation.dart';
import '../recipe.dart';

/// Serializes recipe metadata into JSON-friendly maps for the VS Code host.
///
/// The VS Code extension uses these maps to render the recipe list and to
/// build the argument input form without any knowledge of Dart types.
class RecipeSchema {
  const RecipeSchema._();

  /// Serializes a single [CodemodArg] into a JSON-friendly map.
  static Map<String, Object?> argToJson(CodemodArg arg) {
    return {
      'name': arg.name,
      'abbr': arg.abbr,
      'help': arg.help,
      'required': arg.required,
      'defaultsTo': arg.defaultsTo,
      'inputKind': _inputKindToJson(arg.inputKind),
      'options': arg.options,
      'allowCustomValue': arg.allowCustomValue,
      'contextKey': arg.contextKey,
    };
  }

  /// Serializes a single template preview into a JSON-friendly map.
  static Map<String, Object?> templatePreviewToJson(
    RecipeTemplatePreview preview, {
    required bool includeContent,
  }) {
    return {
      'label': preview.label,
      'path': preview.path,
      if (includeContent) 'content': preview.content.source,
    };
  }

  /// Serializes a [CodemodRecipe] (without its operations) into a JSON map.
  ///
  /// Operations and transforms are intentionally omitted because they are
  /// closures that cannot be represented as data. The extension only needs
  /// the recipe identity and its argument definitions.
  static Map<String, Object?> recipeToJson(
    CodemodRecipe recipe, {
    bool includeTemplateContent = true,
  }) {
    return {
      'name': recipe.name,
      'description': recipe.description,
      'args': [for (final arg in recipe.args) argToJson(arg)],
      'templatesLoaded': includeTemplateContent,
      'previewTemplates': [
        for (final preview in recipe.previewTemplates)
          templatePreviewToJson(
            preview,
            includeContent: includeTemplateContent,
          ),
        for (final operation in recipe.operations)
          if (operation is CreateFileOperation &&
              operation.pathTemplate != null)
            {
              'label':
                  operation.previewLabel ??
                  _labelFromPathTemplate(operation.pathTemplate!),
              'path': operation.pathTemplate,
              if (includeTemplateContent) 'content': operation.template.source,
            },
      ],
    };
  }

  static Map<String, Object?> recipeEntryToJson(
    String id,
    CodemodRecipe recipe, {
    bool includeTemplateContent = true,
  }) {
    return {
      'id': id,
      ...recipeToJson(
        recipe,
        includeTemplateContent: includeTemplateContent,
      ),
    };
  }

  /// Serializes a named registry of recipes into a JSON list.
  static List<Map<String, Object?>> registryToJson(
    Map<String, CodemodRecipe> recipes,
  ) {
    return [
      for (final entry in recipes.entries)
        recipeEntryToJson(
          entry.key,
          entry.value,
          includeTemplateContent: false,
        ),
    ];
  }

  static String _inputKindToJson(CodemodArgInputKind kind) {
    return switch (kind) {
      CodemodArgInputKind.text => 'text',
      CodemodArgInputKind.file => 'file',
      CodemodArgInputKind.directory => 'directory',
      CodemodArgInputKind.enumeration => 'enum',
      CodemodArgInputKind.dartType => 'dartType',
      CodemodArgInputKind.symbol => 'symbol',
    };
  }

  static String _labelFromPathTemplate(String pathTemplate) {
    final name = pathTemplate.split('/').last;
    return name.isEmpty ? 'Generated file' : name;
  }
}
