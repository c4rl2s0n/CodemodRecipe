import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';

export 'offsets.dart' show
  findClassEndOffset,
  findClassBodyStartOffset,
  findOptimalInsertionOffset;

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

/// Returns the field declaration containing variable [fieldName] in [classNode].
FieldDeclaration? findFieldByName(
  ClassDeclaration classNode,
  String fieldName,
) {
  for (final field in getFields(classNode)) {
    for (final variable in field.fields.variables) {
      if (variable.name.lexeme == fieldName) {
        return field;
      }
    }
  }
  return null;
}

/// Returns all class declarations named [className] in [unit].
List<ClassDeclaration> findClassesByName(
  CompilationUnit unit,
  String className,
) {
  return findAllClasses(
    unit,
  ).where((node) => node.name.lexeme == className).toList();
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
