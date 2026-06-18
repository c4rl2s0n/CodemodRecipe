import '../core/operation.dart';
import '../core/recipe.dart';

/// Serializes recipe metadata into JSON-friendly maps for the VS Code host.
class RecipeSchema {
  const RecipeSchema._();

  static Map<String, Object?> argToJson(CodemodArgDescriptor arg) {
    return {
      'name': arg.name,
      'abbr': arg.abbr,
      'help': arg.help,
      'required': arg.required,
      'defaultsTo': arg.serializedDefault,
      'inputKind': _inputKindToJson(arg.resolvedInputKind),
      'options': arg.options,
      'allowCustomValue': arg.allowCustomValue,
      'contextKey': arg.contextKey,
    };
  }

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

  static Map<String, Object?> recipeToJson(
    CodemodRecipe recipe, {
    bool includeTemplateContent = true,
  }) {
    return {
      'name': recipe.name,
      'description': recipe.description,
      'args': [
        for (final arg in recipe.args)
          if (arg.isUserFacing) argToJson(arg),
      ],
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
      ...recipeToJson(recipe, includeTemplateContent: includeTemplateContent),
    };
  }

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
      CodemodArgInputKind.boolean => 'bool',
    };
  }

  static String _labelFromPathTemplate(String pathTemplate) {
    final name = pathTemplate.split('/').last;
    return name.isEmpty ? 'Generated file' : name;
  }
}
