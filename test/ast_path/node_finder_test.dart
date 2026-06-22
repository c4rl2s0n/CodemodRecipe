import 'package:test/test.dart';
import 'package:analyzer/dart/ast/ast.dart';

import '../../lib/src/dart_codegen/ast_helpers/ast_focus.dart';
import '../../lib/src/ast_path/node_finder.dart';
import '../../lib/src/ast_path/model.dart';

void main() {
  group('AstNodeFinder', () {
    group('findNodeAtOffset', () {
      test('finds class declaration', () async {
        const source = '''
class MyClass {
  void method() {}
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find offset within "MyClass"
        final classOffset = source.indexOf('MyClass');
        final node = AstNodeFinder.findNodeAtOffset(unit, classOffset);
        
        expect(node, isA<ClassDeclaration>());
        expect((node as ClassDeclaration).name.lexeme, 'MyClass');
      });

      test('finds method declaration', () async {
        const source = '''
class MyClass {
  void myMethod() {
    print("hello");
  }
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find offset within "myMethod"
        final methodOffset = source.indexOf('myMethod');
        final node = AstNodeFinder.findNodeAtOffset(unit, methodOffset);
        
        expect(node, isA<MethodDeclaration>());
        expect((node as MethodDeclaration).name.lexeme, 'myMethod');
      });

      test('finds field declaration', () async {
        const source = '''
class MyClass {
  int myField = 42;
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find offset within "myField"
        final fieldOffset = source.indexOf('myField');
        final node = AstNodeFinder.findNodeAtOffset(unit, fieldOffset);
        
        // The most specific node is VariableDeclaration (the variable declaration)
        expect(node, isA<VariableDeclaration>());
        final varDecl = node as VariableDeclaration;
        expect(varDecl.name.lexeme, 'myField');
        
        // The parent should be VariableDeclarationList
        final parent = AstNodeFinder.getParentNode(node, unit);
        expect(parent, isA<VariableDeclarationList>());
        
        // The grandparent should be FieldDeclaration
        final grandparent = AstNodeFinder.getParentNode(parent!, unit);
        expect(grandparent, isA<FieldDeclaration>());
      });

      test('finds most specific node (method over class)', () async {
        const source = '''
class MyClass {
  void method() {}
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Offset within method name should find method, not class
        final methodOffset = source.indexOf('method');
        final node = AstNodeFinder.findNodeAtOffset(unit, methodOffset);
        
        expect(node, isA<MethodDeclaration>());
        expect((node as MethodDeclaration).name.lexeme, 'method');
      });

      test('returns null for invalid offset', () async {
        const source = 'class Test {}';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        final node = AstNodeFinder.findNodeAtOffset(unit, -1);
        expect(node, isNull);
        
        final node2 = AstNodeFinder.findNodeAtOffset(unit, 1000);
        expect(node2, isNull);
      });
    });

    group('createPathFromNode', () {
      test('creates path for class declaration', () async {
        const source = '''
class MyClass {
  void method() {}
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find the class node
        final classNode = unit.declarations.firstWhere(
          (decl) => decl is ClassDeclaration
        ) as ClassDeclaration;
        
        final path = AstNodeFinder.createPathFromNode(classNode, unit);
        
        expect(path.navigate, hasLength(1));
        expect(path.navigate.first.kind, NavigateKind.classDecl);
        expect(path.navigate.first.name, 'MyClass');
        expect(path.anchor.kind, AnchorKind.memberLast);
      });

      test('creates path for method declaration', () async {
        const source = '''
class MyClass {
  void myMethod() {}
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find the method node
        final classNode = unit.declarations.firstWhere(
          (decl) => decl is ClassDeclaration
        ) as ClassDeclaration;
        
        final methodNode = classNode.members.firstWhere(
          (member) => member is MethodDeclaration
        ) as MethodDeclaration;
        
        final path = AstNodeFinder.createPathFromNode(methodNode, unit);
        
        expect(path.navigate, hasLength(2));
        expect(path.navigate[0].kind, NavigateKind.classDecl);
        expect(path.navigate[0].name, 'MyClass');
        expect(path.navigate[1].kind, NavigateKind.method);
        expect(path.navigate[1].name, 'myMethod');
        expect(path.anchor.kind, AnchorKind.stmtLast);
      });

      test('creates path for field declaration', () async {
        const source = '''
class MyClass {
  int myField;
}
''';
        final focus = AstFocus.parse(source);
        final unit = focus.unit;
        
        // Find the field node
        final classNode = unit.declarations.firstWhere(
          (decl) => decl is ClassDeclaration
        ) as ClassDeclaration;
        
        final fieldNode = classNode.members.firstWhere(
          (member) => member is FieldDeclaration
        ) as FieldDeclaration;
        
        final path = AstNodeFinder.createPathFromNode(fieldNode, unit);
        
        expect(path.navigate, hasLength(2));
        expect(path.navigate[0].kind, NavigateKind.classDecl);
        expect(path.navigate[0].name, 'MyClass');
        expect(path.navigate[1].kind, NavigateKind.field);
        expect(path.navigate[1].name, 'myField');
        expect(path.anchor.kind, AnchorKind.memberLast);
      });
    });

    group('AstFocus extensions', () {
      test('focusAtOffset finds correct node', () async {
        const source = '''
class TestClass {
  void testMethod() {
    print("hello");
  }
}
''';
        final focus = AstFocus.parse(source);
        
        // Find offset within method name
        final methodOffset = source.indexOf('testMethod');
        final result = focus.focusAtOffset(methodOffset);
        
        expect(result, isNotNull);
        expect(result!.node, isA<MethodDeclaration>());
        expect((result.node as MethodDeclaration).name.lexeme, 'testMethod');
      });

      test('generatePathAtOffset creates valid path', () async {
        const source = '''
class TestClass {
  void testMethod() {}
}
''';
        final focus = AstFocus.parse(source);
        
        // Find offset within method name
        final methodOffset = source.indexOf('testMethod');
        final path = focus.generatePathAtOffset(methodOffset);
        
        expect(path, isNotNull);
        expect(path!.navigate, hasLength(2));
        expect(path.navigate[0].kind, NavigateKind.classDecl);
        expect(path.navigate[1].kind, NavigateKind.method);
      });

      test('returns null for invalid offset', () async {
        const source = 'class Test {}';
        final focus = AstFocus.parse(source);
        
        final result = focus.focusAtOffset(-1);
        expect(result, isNull);
        
        final path = focus.generatePathAtOffset(1000);
        expect(path, isNull);
      });
    });
  });
}