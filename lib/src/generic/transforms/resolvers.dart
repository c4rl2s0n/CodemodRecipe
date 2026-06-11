import '../../context.dart';

/// Resolves a string from codemod context.
// typedef StringResolver = String Function(CodemodContext context);

/// Resolves a source insertion offset.
typedef OffsetResolver = int Function(String source, CodemodContext context);
