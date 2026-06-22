import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';

import '../dart_codegen/ast_helpers/ast_focus.dart';
import 'model.dart';
import 'ast_path_builder.dart';

/// Utility class for finding AST nodes and generating paths from offsets.
class AstNodeFinder {
  /// Finds the most specific AST node that contains the given [offset] within [unit].
  /// 
  /// Uses depth-first traversal to find the smallest node containing the offset.
  /// Returns null if no node contains the offset or if the offset is invalid.
  static AstNode? findNodeAtOffset(CompilationUnit unit, int offset) {
    // Validate offset bounds
    if (offset < 0 || offset > unit.end) {
      return null;
    }
    
    // Start depth-first search from the compilation unit
    return _findContainingNode(unit, offset);
  }
  
  /// Recursively finds the most specific node containing the offset.
  static AstNode? _findContainingNode(AstNode node, int offset) {
    // Check if this node contains the offset
    if (offset < node.offset || offset > node.end) {
      return null;
    }
    
    // Check children first (depth-first) to find the most specific node
    AstNode? mostSpecificChild;
    for (final child in node.childEntities) {
      if (child is AstNode) {
        final result = _findContainingNode(child, offset);
        if (result != null) {
          // If the child is a declaration node, prefer it over non-declaration parents
          if (_isDeclarationNode(result) && !_isDeclarationNode(node)) {
            return result; // Return declaration node immediately
          }
          // If both are declaration nodes, prefer the child (more specific)
          if (_isDeclarationNode(result) && _isDeclarationNode(node)) {
            return result; // Return the more specific declaration
          }
          mostSpecificChild = result;
        }
      }
    }
    
    // If we found a child that's not a declaration (e.g., method name within method declaration)
    // and we're at a declaration node, return the declaration node
    if (mostSpecificChild != null && _isDeclarationNode(node) && !_isDeclarationNode(mostSpecificChild)) {
      return node;
    }
    
    // If we found a child, return it
    if (mostSpecificChild != null) {
      return mostSpecificChild;
    }
    
    // If no child contains the offset, return this node
    return node;
  }
  
  /// Checks if a node is a "declaration" node that should be preferred.
  static bool _isDeclarationNode(AstNode node) {
    return node is ClassDeclaration ||
           node is MethodDeclaration ||
           node is FunctionDeclaration ||
           node is FieldDeclaration ||
           node is VariableDeclaration ||
           node is ConstructorDeclaration;
  }
  
  /// Creates an AST path from a node back to the compilation unit root.
  /// 
  /// Builds navigation steps by traversing from the node up to the root,
  /// then determines the most appropriate anchor for the node type.
  static AstPath createPathFromNode(AstNode node, CompilationUnit unit) {
    final pathBuilder = AstPathBuilder();
    final steps = <NavigateStep>[];
    
    // Build navigation path from node up to root
    AstNode? current = node;
    while (current != null && current != unit) {
      final step = _createNavigationStep(current);
      if (step != null) {
        steps.add(step);
      }
      current = getParentNode(current, unit);
    }
    
    // Reverse to get root-to-node order and add to builder
    for (final step in steps.reversed) {
      _addStepToBuilder(pathBuilder, step);
    }
    
    // Add appropriate anchor based on node type
    final anchor = _determineBestAnchor(node);
    pathBuilder.atAnchor(anchor.kind, name: anchor.name, index: anchor.index);
    
    return pathBuilder.build();
  }
  
  /// Adds a navigation step to the builder using the appropriate method.
  static void _addStepToBuilder(AstPathBuilder builder, NavigateStep step) {
    if (step.kind == null) {
      builder.navigateToInferred(step.name!, match: step.match);
    } else {
      switch (step.kind!) {
        case NavigateKind.root:
          builder.root();
          break;
        case NavigateKind.classDecl:
          builder.navigateToClass(step.name!, match: step.match);
          break;
        case NavigateKind.method:
          builder.navigateToMethod(step.name!, match: step.match);
          break;
        case NavigateKind.constructor:
          builder.navigateToConstructor(name: step.name, match: step.match);
          break;
        case NavigateKind.call:
          builder.navigateToCall(step.name!, match: step.match);
          break;
        case NavigateKind.import:
          builder.navigateToImport(step.name!);
          break;
        case NavigateKind.field:
          builder.navigateToField(step.name!, match: step.match);
          break;
        case NavigateKind.function:
          builder.navigateToFunction(step.name!, match: step.match);
          break;
        case NavigateKind.variable:
          builder.navigateToVariable(step.name!, match: step.match);
          break;
        case NavigateKind.initializer:
          builder.navigateToInitializer(match: step.match);
          break;
        case NavigateKind.redirection:
          builder.navigateToRedirection(match: step.match);
          break;
      }
    }
  }
  
  /// Creates a navigation step for the given node based on its type.
  static NavigateStep? _createNavigationStep(AstNode node) {
    // Handle different node types
    if (node is ClassDeclaration) {
      return NavigateStep(NavigateKind.classDecl, name: node.name.lexeme);
    } else if (node is MethodDeclaration) {
      return NavigateStep(NavigateKind.method, name: node.name.lexeme);
    } else if (node is ConstructorDeclaration) {
      final name = node.name?.lexeme;
      return NavigateStep(NavigateKind.constructor, name: name);
    } else if (node is FieldDeclaration) {
      // For field declarations, use the first variable name
      final firstVar = node.fields.variables.firstOrNull;
      if (firstVar != null) {
        return NavigateStep(NavigateKind.field, name: firstVar.name.lexeme);
      }
    } else if (node is FunctionDeclaration) {
      return NavigateStep(NavigateKind.function, name: node.name.lexeme);
    } else if (node is VariableDeclaration) {
      // Top-level variable
      return NavigateStep(NavigateKind.variable, name: node.name.lexeme);
    } else if (node is InstanceCreationExpression) {
      // Constructor call
      final typeName = node.constructorName.type.name?.lexeme;
      if (typeName != null) {
        return NavigateStep(NavigateKind.call, name: typeName);
      }
    } else if (node is MethodInvocation) {
      // Method call
      final methodName = node.methodName.name;
      return NavigateStep(NavigateKind.call, name: methodName);
    }
    
    // For unsupported node types, return null (will be skipped in path building)
    return null;
  }
  
  /// Determines the best anchor for insertion relative to the given node.
  static Anchor _determineBestAnchor(AstNode node) {
    // Default to bodyEnd for most nodes
    if (node is ClassDeclaration) {
      return const Anchor(AnchorKind.memberLast);
    } else if (node is MethodDeclaration || node is FunctionDeclaration) {
      return const Anchor(AnchorKind.stmtLast);
    } else if (node is ConstructorDeclaration) {
      return const Anchor(AnchorKind.paramLast);
    } else if (node is FieldDeclaration) {
      return const Anchor(AnchorKind.memberLast);
    } else if (node is InstanceCreationExpression || node is MethodInvocation) {
      // For method/constructor calls, try to find more specific anchors
      return _determineCallAnchor(node);
    }
    
    // Fallback for other node types
    return const Anchor(AnchorKind.bodyEnd);
  }
  
  /// Determines the most specific anchor for method/constructor calls.
  static Anchor _determineCallAnchor(AstNode node) {
    if (node is MethodInvocation) {
      final argList = node.argumentList;
      if (argList != null && argList.arguments.isNotEmpty) {
        // Check if we have named arguments
        for (int i = 0; i < argList.arguments.length; i++) {
          final arg = argList.arguments[i];
          if (arg is NamedExpression) {
            // Return anchor for this specific named argument
            return Anchor(AnchorKind.argName, name: arg.name.label.name);
          }
        }
        // Return anchor for last argument position
        return Anchor(AnchorKind.argIndex, index: argList.arguments.length - 1);
      }
    } else if (node is InstanceCreationExpression) {
      final argList = node.argumentList;
      if (argList != null && argList.arguments.isNotEmpty) {
        // Check for named arguments in constructor calls
        for (int i = 0; i < argList.arguments.length; i++) {
          final arg = argList.arguments[i];
          if (arg is NamedExpression) {
            return Anchor(AnchorKind.argName, name: arg.name.label.name);
          }
        }
        return Anchor(AnchorKind.argIndex, index: argList.arguments.length - 1);
      }
    }
    
    // Fallback to argLast for any call
    return const Anchor(AnchorKind.argLast);
  }
  
  /// Gets the parent node of the given node by traversing the AST.
  /// 
  /// This is a fallback method when parent references aren't available.
  static AstNode? getParentNode(AstNode node, CompilationUnit unit) {
    // Use parent reference if available (analyzer 5.0+)
    if (node.parent != null) {
      return node.parent;
    }
    
    // Fallback: find parent by searching from root
    // This is less efficient but works with older analyzer versions
    final parentFinder = _ParentFinder(node);
    unit.accept(parentFinder);
    return parentFinder.parent;
  }
}

/// Visitor to find the parent of a specific node.
class _ParentFinder extends RecursiveAstVisitor<void> {
  final AstNode targetNode;
  AstNode? parent;
  
  _ParentFinder(this.targetNode);
  
  @override
  void visitClassDeclaration(ClassDeclaration node) {
    _checkChildren(node);
    super.visitClassDeclaration(node);
  }
  
  @override
  void visitMethodDeclaration(MethodDeclaration node) {
    _checkChildren(node);
    super.visitMethodDeclaration(node);
  }
  
  @override
  void visitFieldDeclaration(FieldDeclaration node) {
    _checkChildren(node);
    super.visitFieldDeclaration(node);
  }
  
  @override
  void visitFunctionDeclaration(FunctionDeclaration node) {
    _checkChildren(node);
    super.visitFunctionDeclaration(node);
  }
  
  @override
  void visitVariableDeclaration(VariableDeclaration node) {
    _checkChildren(node);
    super.visitVariableDeclaration(node);
  }
  
  @override
  void visitInstanceCreationExpression(InstanceCreationExpression node) {
    _checkChildren(node);
    super.visitInstanceCreationExpression(node);
  }
  
  @override
  void visitMethodInvocation(MethodInvocation node) {
    _checkChildren(node);
    super.visitMethodInvocation(node);
  }
  
  @override
  void visitConstructorDeclaration(ConstructorDeclaration node) {
    _checkChildren(node);
    super.visitConstructorDeclaration(node);
  }
  
  void _checkChildren(AstNode node) {
    // Check if any child of this node is our target
    for (final child in node.childEntities) {
      if (child is AstNode && identical(child, targetNode)) {
        parent = node;
        return; // Found the parent
      }
    }
  }
}