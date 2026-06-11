import 'package:analyzer/dart/ast/ast.dart';

import 'localizers.dart';

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

/// Returns whether [constructor] has a parameter named [name].
bool constructorDeclHasParam(
  ConstructorDeclaration constructor,
  String source,
  String name,
) {
  final paramsSource = source.substring(
    constructor.parameters.offset,
    constructor.parameters.end,
  );
  return RegExp(
    r'(\bthis\.|\b)' + RegExp.escape(name) + r'\b',
  ).hasMatch(paramsSource);
}
