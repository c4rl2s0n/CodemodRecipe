/// AST navigation utilities for Dart source code analysis.
///
/// Provides helper functions to locate and inspect AST nodes using the
/// analyzer package.
// ignore_for_file: dangling_library_doc_comments, deprecated_member_use

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';

/// Parses Dart [source] into an unresolved analyzer compilation unit.
CompilationUnit parseSource(String source, {String path = '<unknown>'}) {
  final result = parseString(content: source, path: path);
  return result.unit;
}

/// Returns the first class declaration named [className] in [unit].
ClassDeclaration? findClassByName(CompilationUnit unit, String className) {
  ClassDeclaration? found;

  unit.accept(
    ClassVisitor((node) {
      if (node.name.lexeme == className) {
        found = node;
      }
    }),
  );

  return found;
}

/// Returns the method named [methodName] directly declared in [classNode].
MethodDeclaration? findMethodByName(
  ClassDeclaration classNode,
  String methodName,
) {
  for (final member in classNode.members) {
    if (member is MethodDeclaration && member.name.lexeme == methodName) {
      return member;
    }
  }
  return null;
}

/// Finds a constructor in [classNode].
///
/// When [name] is omitted, prefers the unnamed constructor and falls back to
/// the first constructor declaration.
ConstructorDeclaration? findConstructor(
  ClassDeclaration classNode, {
  String? name,
}) {
  for (final member in classNode.members) {
    if (member is ConstructorDeclaration) {
      final constructorName = member.name?.lexeme;
      if (name == null && constructorName == null) {
        return member;
      }
      if (constructorName == name) {
        return member;
      }
    }
  }

  if (name == null) {
    for (final member in classNode.members) {
      if (member is ConstructorDeclaration) {
        return member;
      }
    }
  }

  return null;
}

/// Returns the source offset of [classNode]'s closing brace token.
int findClassEndOffset(ClassDeclaration classNode) {
  return classNode.endToken.offset;
}

/// Returns the insertion offset after the last constructor parameter.
int findLastParameterOffset(ConstructorDeclaration constructor) {
  final parameters = constructor.parameters;
  if (parameters.parameters.isEmpty) {
    return parameters.leftParenthesis.end;
  }

  final lastParam = parameters.parameters.last;
  return lastParam.end;
}

/// Returns abstract methods directly declared in [classNode].
List<MethodDeclaration> getAbstractMethods(ClassDeclaration classNode) {
  return classNode.members
      .whereType<MethodDeclaration>()
      .where((m) => m.isAbstract)
      .toList();
}

/// Returns field declarations directly declared in [classNode].
List<FieldDeclaration> getFields(ClassDeclaration classNode) {
  return classNode.members.whereType<FieldDeclaration>().toList();
}

/// Returns whether [classNode] extends a class named [baseClassName].
bool extendsClass(ClassDeclaration classNode, String baseClassName) {
  final extendsClause = classNode.extendsClause;
  if (extendsClause == null) return false;

  final superClass = extendsClause.superclass;
  final name = superClass.name.lexeme;

  return name == baseClassName || name.startsWith('$baseClassName<');
}

/// Returns method declarations directly declared in [classNode].
List<MethodDeclaration> getMethods(ClassDeclaration classNode) {
  return classNode.members.whereType<MethodDeclaration>().toList();
}

/// Returns the insertion offset immediately after the class opening brace.
int findClassBodyStartOffset(ClassDeclaration classNode) {
  return classNode.leftBracket.end;
}

/// Recursive visitor that invokes [onClass] for each class declaration.
class ClassVisitor extends RecursiveAstVisitor<void> {
  /// Callback invoked for each visited class declaration.
  final void Function(ClassDeclaration) onClass;

  /// Creates a visitor that reports visited class declarations.
  ClassVisitor(this.onClass);

  @override
  void visitClassDeclaration(ClassDeclaration node) {
    onClass(node);
    super.visitClassDeclaration(node);
  }
}

/// Returns all class declarations found in [unit].
List<ClassDeclaration> findAllClasses(CompilationUnit unit) {
  final classes = <ClassDeclaration>[];
  unit.accept(ClassVisitor(classes.add));
  return classes;
}

/// Returns whether [classNode] has a method named [methodName].
bool hasMethodInClass(ClassDeclaration classNode, String methodName) {
  return findMethodByName(classNode, methodName) != null;
}

/// Returns whether [classNode] has a field named [fieldName].
bool hasFieldInClass(ClassDeclaration classNode, String fieldName) {
  for (final member in classNode.members) {
    if (member is FieldDeclaration) {
      for (final variable in member.fields.variables) {
        if (variable.name.lexeme == fieldName) {
          return true;
        }
      }
    }
  }
  return false;
}

/// Returns a stable insertion offset for adding a new class member.
int findOptimalInsertionOffset(ClassDeclaration classNode) {
  final methods = getMethods(classNode);
  if (methods.isNotEmpty) {
    return methods.last.end;
  }

  final fields = getFields(classNode);
  if (fields.isNotEmpty) {
    return fields.last.end;
  }

  return findClassBodyStartOffset(classNode);
}
