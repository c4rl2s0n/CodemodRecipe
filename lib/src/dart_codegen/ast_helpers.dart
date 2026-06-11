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


/// Infers the constructor parameter list kind from existing parameters.
///
/// Returns null when the list is empty `()` and no delimiter is present.
ConstructorParamStyle? inferConstructorParamStyle(
  ConstructorDeclaration constructor,
  String source,
) {
  final params = constructor.parameters;
  if (params.parameters.isNotEmpty) {
    for (final parameter in params.parameters) {
      if (parameter is DefaultFormalParameter) {
        return parameter.isPositional
            ? ConstructorParamStyle.optionalPositional
            : ConstructorParamStyle.named;
      }
    }
    return ConstructorParamStyle.positional;
  }

  final inner = source
      .substring(params.leftParenthesis.end, params.rightParenthesis.offset)
      .trim();
  if (inner.startsWith('{')) return ConstructorParamStyle.named;
  if (inner.startsWith('[')) return ConstructorParamStyle.optionalPositional;
  if (inner.isEmpty) return null;

  return ConstructorParamStyle.positional;
}

/// Resolves the constructor parameter list kind, using [emptyStyle] for `()`.
ConstructorParamStyle resolveConstructorParamStyle(
  ConstructorDeclaration constructor,
  String source, {
  ConstructorParamStyle? emptyStyle,
}) {
  final inferred = inferConstructorParamStyle(constructor, source);
  if (inferred != null) return inferred;

  return emptyStyle ?? ConstructorParamStyle.named;
}

/// Builds the constructor parameter text for a [FieldSpec].
String buildConstructorFieldParam(
  FieldSpec spec,
  ConstructorParamStyle kind, {
  bool thisPrefix = true,
}) {
  final paramName = thisPrefix
      ? 'this.${spec.name}'
      : '${spec.declarationType} ${spec.name}';

  switch (kind) {
    case ConstructorParamStyle.named:
      if (spec.defaultValue != null) {
        return '$paramName = ${spec.defaultValue}';
      }
      if (!spec.isNullable) {
        return 'required $paramName';
      }
      return paramName;
    case ConstructorParamStyle.positional:
      if (spec.defaultValue != null) {
        throw StateError(
          'Positional constructor parameters cannot have default values',
        );
      }
      return paramName;
    case ConstructorParamStyle.optionalPositional:
      if (!spec.isNullable && spec.defaultValue == null) {
        throw StateError(
          'Non-nullable optional positional parameters require a defaultValue',
        );
      }
      if (spec.defaultValue != null) {
        return '$paramName = ${spec.defaultValue}';
      }
      return paramName;
  }
}

/// Describes a patch to insert or replace constructor parameter text.
class ConstructorParamInsertion {
  /// Start offset in the source.
  final int offset;

  /// Number of characters to replace, or 0 for pure insertion.
  final int length;

  /// Text to insert or substitute.
  final String text;

  /// Creates a constructor parameter insertion plan.
  const ConstructorParamInsertion({
    required this.offset,
    required this.length,
    required this.text,
  });
}

/// Plans where and how to insert [paramText] into a constructor parameter list.
ConstructorParamInsertion planConstructorParamInsertion(
  ConstructorDeclaration constructor,
  String source,
  ConstructorParamStyle kind,
  String paramText,
) {
  final params = constructor.parameters;
  final hasParams = params.parameters.isNotEmpty;

  if (!hasParams) {
    final innerStart = params.leftParenthesis.end;
    final innerEnd = params.rightParenthesis.offset;
    final inner = source.substring(innerStart, innerEnd).trim();

    if (inner.isEmpty) {
      switch (kind) {
        case ConstructorParamStyle.named:
          return ConstructorParamInsertion(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: '{$paramText}',
          );
        case ConstructorParamStyle.optionalPositional:
          return ConstructorParamInsertion(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: '[$paramText]',
          );
        case ConstructorParamStyle.positional:
          return ConstructorParamInsertion(
            offset: innerStart,
            length: innerEnd - innerStart,
            text: paramText,
          );
      }
    }

    if (inner.startsWith('{')) {
      final braceOffset = source.indexOf('{', params.leftParenthesis.offset);
      return ConstructorParamInsertion(
        offset: braceOffset + 1,
        length: 0,
        text: paramText,
      );
    }

    if (inner.startsWith('[')) {
      final bracketOffset = source.indexOf('[', params.leftParenthesis.offset);
      return ConstructorParamInsertion(
        offset: bracketOffset + 1,
        length: 0,
        text: paramText,
      );
    }
  }

  final buffer = StringBuffer();
  if (hasParams) {
    buffer.writeln(',');
    buffer.write('    ');
  }
  buffer.write(paramText);

  return ConstructorParamInsertion(
    offset: findLastParameterOffset(constructor),
    length: 0,
    text: buffer.toString(),
  );
}
