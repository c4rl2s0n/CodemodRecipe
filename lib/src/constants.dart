// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

/// Centralized constants for the codemod_recipe package.
///
/// This file contains all magic strings, numbers, and configuration values
/// to improve maintainability and follow the DRY principle.

/// VS Code Extension Host Constants
/// {
const String kResultBegin = '__CODEMOD_RESULT_BEGIN__';
const String kResultEnd = '__CODEMOD_RESULT_END__';
/// }

/// CLI Argument Constants
/// {
const String kHelpFlag = 'help';
const String kHelpAbbr = 'h';
const String kApplyFlag = 'apply';
const String kApplyAbbr = 'a';
const String kMapRootFlag = 'map-root';
const String kMapRootAbbr = 'm';
const String kValidateFlag = 'validate';
const String kStdioServerFlag = 'stdio-server';
/// }

/// File System Constants
/// {
const String kDefaultMapRoot = '.codemod/maps';
const String kYamlExtension = '.yaml';
const String kYmlExtension = '.yml';
const String kDartExtension = '.dart';
/// }

/// YAML Schema Constants
/// {
const String kIdField = 'id';
const String kNameField = 'name';
const String kDescriptionField = 'description';
const String kArgsField = 'args';
const String kStepsField = 'steps';
const String kRecipeField = 'recipe';
const String kEditField = 'edit';
const String kCreateField = 'create';
const String kPathField = 'path';
const String kInsertField = 'insert';
const String kTextField = 'text';
const String kTemplateField = 'template';
const String kTemplateFileField = 'templateFile';
const String kMapsField = 'maps';
const String kEntriesField = 'entries';
const String kPostExecutionField = 'postExecution';
const String kRunField = 'run';
const String kRunScriptField = 'runScript';
const String kIfExistsField = 'ifExists';
const String kFormatField = 'format';
const String kAtField = 'at';
const String kAnchorField = 'anchor';
const String kNavigateField = 'navigate';
const String kMatchField = 'match';
/// }

/// Anchor String Constants
/// {
// Simple anchors
const String kBodyStartAnchor = 'body:start';
const String kBodyEndAnchor = 'body:end';
const String kStmtLastAnchor = 'stmt:last';
const String kStmtDollarAnchor = r'stmt:$';
const String kMemberLastAnchor = 'member:last';
const String kParamLastAnchor = 'param:last';
const String kArgLastAnchor = 'arg:last';
const String kMetaBeforeAnchor = 'meta:before';
const String kDocBeforeAnchor = 'doc:before';
const String kDocAfterAnchor = 'doc:after';
const String kInitializerReplaceAnchor = 'initializer:replace';
const String kInitializerLastAnchor = 'initializer:last';
const String kRedirectionArgLastAnchor = 'redirection:arg:last';

// Prefixes for parameterized anchors
const String kParamNamePrefix = 'param:name:';
const String kArgNamePrefix = 'arg:name:';
const String kParamIndexPrefix = 'param:';
const String kArgIndexPrefix = 'arg:';
const String kInitializerNamePrefix = 'initializer:name:';
const String kRedirectionArgNamePrefix = 'redirection:arg:name:';
/// }

/// Navigation Step Constants
/// {
const String kRootNav = 'root';
const String kDotNav = '.';
const String kClassNav = 'class';
const String kMethodNav = 'method';
const String kCtorNav = 'ctor';
const String kConstructorNav = 'constructor';
const String kCallNav = 'call';
const String kImportNav = 'import';
const String kFieldNav = 'field';
const String kFunctionNav = 'function';
const String kVarNav = 'var';
const String kVariableNav = 'variable';
const String kInitializerNav = 'initializer';
const String kRedirectionNav = 'redirection';
/// }

/// Input Kind Constants
/// {
const String kTextInputKind = 'text';
const String kFileInputKind = 'file';
const String kDirectoryInputKind = 'directory';
const String kEnumInputKind = 'enum';
const String kDartTypeInputKind = 'dartType';
const String kSymbolInputKind = 'symbol';
const String kBoolInputKind = 'bool';
const String kBooleanInputKind = 'boolean';
/// }

/// IfExists Strategy Constants
/// {
const String kSkipIfExists = 'skip';
const String kOverwriteIfExists = 'overwrite';
const String kFailIfExists = 'fail';
/// }

/// Post Execution Constants
/// {
const String kDartFormatPostExecution = 'dartFormat';
const String kBuildRunnerPostExecution = 'buildRunner';
/// }

/// Diagnostic Constants
/// {
const String kErrorSeverity = 'error';
const String kWarningSeverity = 'warning';
const String kInfoSeverity = 'info';
const String kHintSeverity = 'hint';

// Diagnostic codes
const String kYamlSchemaError = 'E_YAML_SCHEMA';
const String kYamlCompileError = 'E_YAML_COMPILE';
const String kAstPathParseError = 'E_AST_PATH_PARSE';
const String kRecipeRefNotFoundError = 'E_RECIPE_REF_NOT_FOUND';
const String kMapIdNotFoundWarning = 'W_MAP_ID_NOT_FOUND';
/// }

/// Command Constants
/// {
const String kListCommand = 'list';
const String kDescribeCommand = 'describe';
const String kPreviewCommand = 'preview';
const String kDiffCommand = 'diff';
const String kApplyCommand = 'apply';
const String kReloadCommand = 'reload';
const String kValidateCommand = 'validate';
/// }

/// Response Field Constants
/// {
const String kOkField = 'ok';
const String kErrorField = 'error';
const String kStackField = 'stack';
const String kRecipesField = 'recipes';
const String kDiagnosticsField = 'diagnostics';
const String kFilesField = 'files';
const String kFileField = 'file';
const String kRecipeFieldResponse = 'recipe';
const String kAppliedField = 'applied';
const String kTimingsField = '_timingsMs';
const String kHostMetricsField = '_hostMetrics';
const String kCommandField = 'command';
/// }
