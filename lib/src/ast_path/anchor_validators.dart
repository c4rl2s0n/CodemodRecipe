// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'package:analyzer/dart/ast/ast.dart';

import 'model.dart';

/// Strategy interface for validating if an anchor is valid for a given node.
///
/// This pattern allows for extensible anchor validation where new anchor types
/// can be added without modifying existing validation logic.
abstract class AnchorValidator {
  /// Returns true if this validator can handle validation for the given [anchorKind].
  bool canValidate(AnchorKind anchorKind);

  /// Returns true if the [anchor] is valid for the given [node].
  ///
  /// The [anchor] parameter includes the full anchor information including
  /// any named or indexed parameters.
  bool isValidFor(AstNode node, Anchor anchor);
}

/// Strategy interface for resolving anchor offsets.
///
/// Each anchor kind has its own resolution strategy that knows how to
/// calculate the byte offset for a specific anchor relative to a node.
abstract class AnchorResolver {
  /// Returns true if this resolver can handle resolution for the given [anchorKind].
  bool canResolve(AnchorKind anchorKind);

  /// Resolves the [anchor] to a byte offset within [source] for the given [node].
  ///
  /// Returns the resolved offset, or throws a StateError if the anchor cannot
  /// be resolved for the given node type.
  int resolveOffset({
    required String source,
    required AstNode node,
    required Anchor anchor,
  });

  /// Resolves the [anchor] to an AnchorSpan within [source] for the given [node].
  ///
  /// Returns the resolved span, or throws a StateError if the anchor cannot
  /// be resolved for the given node type.
  AnchorSpan resolveSpan({
    required String source,
    required AstNode node,
    required Anchor anchor,
  });
}

/// Validator for body-related anchors (body:start, body:end).
class BodyAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.bodyStart ||
        anchorKind == AnchorKind.bodyEnd;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is ClassDeclaration || node is FunctionDeclaration;
  }
}

/// Validator for member-related anchors (member:last).
class MemberAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.memberLast;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is ClassDeclaration;
  }
}

/// Validator for statement-related anchors (stmt:last).
class StatementAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.stmtLast;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is MethodDeclaration || node is FunctionDeclaration;
  }
}

/// Validator for parameter-related anchors (param:last, param:name, param:index).
class ParameterAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.paramLast ||
        anchorKind == AnchorKind.paramName ||
        anchorKind == AnchorKind.paramIndex;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is ConstructorDeclaration || node is FunctionDeclaration;
  }
}

/// Validator for argument-related anchors (arg:last, arg:name, arg:index).
class ArgumentAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.argLast ||
        anchorKind == AnchorKind.argName ||
        anchorKind == AnchorKind.argIndex;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    // Arguments are valid for call-like nodes
    return _isCallLike(node);
  }

  /// Returns true if [node] is a call-like expression (constructor call or method invocation).
  static bool _isCallLike(AstNode node) {
    return node is InstanceCreationExpression || node is MethodInvocation;
  }
}

/// Validator for metadata/documentation anchors (meta:before, doc:before, doc:after).
class MetaAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.metaBefore ||
        anchorKind == AnchorKind.docBefore ||
        anchorKind == AnchorKind.docAfter;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is ClassDeclaration ||
        node is MethodDeclaration ||
        node is ConstructorDeclaration ||
        node is FieldDeclaration ||
        node is FunctionDeclaration;
  }
}

/// Validator for initializer-related anchors (initializer:replace, initializer:last, initializer:name).
class InitializerAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.initializerReplace ||
        anchorKind == AnchorKind.initializerLast ||
        anchorKind == AnchorKind.initializerName;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    if (anchor.kind == AnchorKind.initializerReplace) {
      return node is FieldDeclaration;
    }
    return node is ConstructorDeclaration;
  }
}

/// Validator for redirection-related anchors (redirection:arg:last, redirection:arg:name).
class RedirectionAnchorValidator implements AnchorValidator {
  @override
  bool canValidate(AnchorKind anchorKind) {
    return anchorKind == AnchorKind.redirectionArgLast ||
        anchorKind == AnchorKind.redirectionArgName;
  }

  @override
  bool isValidFor(AstNode node, Anchor anchor) {
    return node is ConstructorDeclaration;
  }
}

/// Registry of all known anchor validators.
class AnchorValidatorRegistry {
  static final List<AnchorValidator> _validators = [
    BodyAnchorValidator(),
    MemberAnchorValidator(),
    StatementAnchorValidator(),
    ParameterAnchorValidator(),
    ArgumentAnchorValidator(),
    MetaAnchorValidator(),
    InitializerAnchorValidator(),
    RedirectionAnchorValidator(),
  ];

  /// Returns whether the [anchor] is valid for the [node] of the given type.
  static bool isValidFor(AstNode node, Anchor anchor) {
    for (final validator in _validators) {
      if (validator.canValidate(anchor.kind)) {
        return validator.isValidFor(node, anchor);
      }
    }
    // If no validator found, assume it's not valid
    return false;
  }

  /// Adds a custom validator to the registry.
  ///
  /// This allows for extending the validation system with new anchor types
  /// without modifying the core validation logic.
  static void addValidator(AnchorValidator validator) {
    _validators.add(validator);
  }

  /// Removes a validator from the registry.
  static void removeValidator(AnchorValidator validator) {
    _validators.remove(validator);
  }
}
