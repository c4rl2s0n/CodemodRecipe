part of '../code_editor.dart';

extension FieldEditor on CodeEditor {
  /// Returns whether the class declares a field named [name].
  bool hasField(AstFocus focus, String name) {
    return focus.classHasField(name);
  }

  /// Returns whether the class constructor has a parameter named [name].
  bool hasConstructorParam(AstFocus focus, String name) {
    final classDecl = focus.asClass;
    final constructor = findConstructor(classDecl);
    if (constructor == null) return false;

    return constructorDeclHasParam(constructor, source, name);
  }

  /// Adds a field declaration to the focused class.
  CodeEditor addField(
    AstFocus focus,
    String name,
    String type, {
    bool isNullable = false,
    String? defaultValue,
    bool addToConstructor = true,
    bool isFinal = true,
    bool isConst = false,
    bool isStatic = false,
    FieldConstructorArgs? constructorArgs,
  }) {
    final spec = FieldSpec(
      name: name,
      type: type,
      isNullable: isNullable,
      defaultValue: defaultValue,
      isFinal: isFinal,
      isConst: isConst,
      isStatic: isStatic,
    );
    return _addFieldFromSpec(
      focus,
      spec,
      addToConstructor: addToConstructor,
      constructorArgs: constructorArgs,
    );
  }

  /// Adds a field only when the class has no field named [name].
  CodeEditor addFieldUnlessExists(
    AstFocus focus,
    String name,
    String type, {
    bool isNullable = false,
    String? defaultValue,
    bool addToConstructor = true,
    bool isFinal = true,
    bool isConst = false,
    bool isStatic = false,
    FieldConstructorArgs? constructorArgs,
  }) {
    if (!hasField(focus, name)) {
      addField(
        focus,
        name,
        type,
        isNullable: isNullable,
        defaultValue: defaultValue,
        addToConstructor: addToConstructor,
        isFinal: isFinal,
        isConst: isConst,
        isStatic: isStatic,
        constructorArgs: constructorArgs,
      );
    }
    return this;
  }

  /// Wires [spec] to the focused class's unnamed constructor.
  CodeEditor addFieldToConstructor(
    AstFocus focus,
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    if (spec.isStatic) return this;

    _wireFieldToConstructor(focus, spec, constructorArgs: constructorArgs);
    return this;
  }

  /// Wires [spec] to the constructor only when the parameter is not present.
  CodeEditor addFieldToConstructorUnlessExists(
    AstFocus focus,
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    if (!hasConstructorParam(focus, spec.name)) {
      addFieldToConstructor(
        focus,
        spec,
        constructorArgs: constructorArgs,
      );
    }
    return this;
  }

  /// Adds a parameter to the focused class's unnamed constructor.
  CodeEditor addConstructorParam(
    AstFocus focus,
    String name,
    String type, {
    bool isNullable = false,
    String? defaultValue,
    bool thisPrefix = true,
    FieldConstructorArgs? constructorArgs,
  }) {
    final spec = FieldSpec(
      name: name,
      type: type,
      isNullable: isNullable,
      defaultValue: defaultValue,
      isFinal: false,
    );
    return addFieldToConstructor(
      focus,
      spec,
      constructorArgs: FieldConstructorArgs(
        style: constructorArgs?.style,
        thisPrefix: thisPrefix,
      ),
    );
  }

  /// Adds a constructor parameter only when it is not already present.
  CodeEditor addConstructorParamUnlessExists(
    AstFocus focus,
    String name,
    String type, {
    bool isNullable = false,
    String? defaultValue,
    bool thisPrefix = true,
    FieldConstructorArgs? constructorArgs,
  }) {
    if (!hasConstructorParam(focus, name)) {
      addConstructorParam(
        focus,
        name,
        type,
        isNullable: isNullable,
        defaultValue: defaultValue,
        thisPrefix: thisPrefix,
        constructorArgs: constructorArgs,
      );
    }
    return this;
  }

  CodeEditor _addFieldFromSpec(
    AstFocus focus,
    FieldSpec spec, {
    required bool addToConstructor,
    FieldConstructorArgs? constructorArgs,
  }) {
    final classDecl = focus.asClass;
    final effectiveAddToConstructor = spec.isStatic ? false : addToConstructor;

    final modifiers = <String>[];
    if (spec.isStatic) modifiers.add('static');
    if (spec.isConst) {
      modifiers.add('const');
    } else if (spec.isFinal) {
      modifiers.add('final');
    }
    final prefix = modifiers.isEmpty ? '' : '${modifiers.join(' ')} ';

    final fieldDeclaration = StringBuffer()
      ..write('\n\n  $prefix${spec.declarationType} ${spec.name}');
    if (spec.isConst && spec.defaultValue != null) {
      fieldDeclaration.write(' = ${spec.defaultValue}');
    }
    fieldDeclaration.write(';');

    final fields = getFields(classDecl);
    final fieldInsertOffset = fields.isNotEmpty
        ? fields.last.end
        : findClassBodyStartOffset(classDecl);

    insert(
      fieldInsertOffset,
      fieldDeclaration.toString(),
      description: 'Add field ${spec.name} to ${classDecl.name.lexeme}',
    );

    if (effectiveAddToConstructor && findConstructor(classDecl) != null) {
      _wireFieldToConstructor(
        focus,
        spec,
        constructorArgs: constructorArgs,
      );
    }

    return this;
  }

  void _wireFieldToConstructor(
    AstFocus focus,
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    final classDecl = focus.asClass;
    final constructor = findConstructor(classDecl);
    if (constructor == null) {
      throw StateError('No constructor found in ${classDecl.name.lexeme}');
    }

    final args = constructorArgs ?? const FieldConstructorArgs();
    final kind = resolveConstructorParamStyle(
      constructor,
      source,
      emptyStyle: args.style ?? preferences.emptyConstructorStyle,
    );
    final paramText = buildConstructorFieldParam(
      spec,
      kind,
      thisPrefix: args.thisPrefix,
    );
    final plan = planConstructorParamInsertion(
      constructor,
      source,
      kind,
      paramText,
    );

    insertPlan(
      plan,
      description: 'Add ${spec.name} parameter to constructor',
    );
  }
}
