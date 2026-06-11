part of '../code_editor.dart';

extension FieldEditor on CodeEditor {
  /// Returns whether the selected class declares a field named [name].
  ///
  /// Only fields declared directly in the selected class are considered.
  bool hasField(String name) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }
    return hasFieldInClass(_currentClass!, name);
  }

  /// Returns whether the selected class constructor has a parameter named [name].
  bool hasConstructorParam(String name) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

    final constructor = findConstructor(_currentClass!);
    if (constructor == null) return false;

    final paramsSource = _source.substring(
      constructor.parameters.offset,
      constructor.parameters.end,
    );
    return RegExp(
      r'(\bthis\.|\b)' + RegExp.escape(name) + r'\b',
    ).hasMatch(paramsSource);
  }


  /// Adds a field declaration to the selected class.
  ///
  /// Modifiers are controlled by [isFinal], [isConst], and [isStatic].
  /// When [addToConstructor] is true, also wires the field to the first
  /// unnamed constructor via [addFieldToConstructor]. [isStatic] forces
  /// [addToConstructor] to false.
  CodeEditor addField(
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
      spec,
      addToConstructor: addToConstructor,
      constructorArgs: constructorArgs,
    );
  }

  /// Adds a field only when the selected class has no field named [spec.name].
  ///
  /// This is the preferred helper for idempotent field transforms.
  CodeEditor addFieldUnlessExists(
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
    if (!hasField(name)) {
      addField(
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

  /// Wires [spec] to the selected class's unnamed constructor.
  ///
  /// Skips when [spec.isStatic] is true. Throws when no constructor exists.
  CodeEditor addFieldToConstructor(
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }
    if (spec.isStatic) return this;

    _wireFieldToConstructor(spec, constructorArgs: constructorArgs);
    return this;
  }

  /// Wires [spec] to the constructor only when the parameter is not present.
  CodeEditor addFieldToConstructorUnlessExists(
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    if (!hasConstructorParam(spec.name)) {
      addFieldToConstructor(spec, constructorArgs: constructorArgs);
    }
    return this;
  }

  /// Adds a parameter to the selected class's unnamed constructor.
  ///
  /// When [thisPrefix] is true, emits `this.name`; otherwise emits
  /// `type name`.
  CodeEditor addConstructorParam(
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
      spec,
      constructorArgs: FieldConstructorArgs(
        style: constructorArgs?.style,
        thisPrefix: thisPrefix,
      ),
    );
  }

  /// Adds a constructor parameter only when it is not already present.
  ///
  /// This is the preferred helper for idempotent constructor transforms.
  CodeEditor addConstructorParamUnlessExists(
    String name,
    String type, {
    bool isNullable = false,
    String? defaultValue,
    bool thisPrefix = true,
    FieldConstructorArgs? constructorArgs,
  }) {
    if (!hasConstructorParam(name)) {
      addConstructorParam(
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
    FieldSpec spec, {
    required bool addToConstructor,
    FieldConstructorArgs? constructorArgs,
  }) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

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

    final fields = getFields(_currentClass!);
    final fieldInsertOffset = fields.isNotEmpty
        ? fields.last.end
        : findClassBodyStartOffset(_currentClass!);

    _patches.add(
      SourcePatch(
        fieldInsertOffset,
        0,
        fieldDeclaration.toString(),
        description: 'Add field ${spec.name} to ${_currentClass!.name.lexeme}',
      ),
    );

    if (effectiveAddToConstructor &&
        findConstructor(_currentClass!) != null) {
      _wireFieldToConstructor(spec, constructorArgs: constructorArgs);
    }

    return this;
  }

  void _wireFieldToConstructor(
    FieldSpec spec, {
    FieldConstructorArgs? constructorArgs,
  }) {
    final constructor = findConstructor(_currentClass!);
    if (constructor == null) {
      throw StateError('No constructor found in ${_currentClass!.name.lexeme}');
    }

    final args = constructorArgs ?? const FieldConstructorArgs();
    final kind = resolveConstructorParamStyle(
      constructor,
      _source,
      emptyStyle: args.style ?? _preferences.emptyConstructorStyle,
    );
    final paramText = buildConstructorFieldParam(
      spec,
      kind,
      thisPrefix: args.thisPrefix,
    );
    final insertion = planConstructorParamInsertion(
      constructor,
      _source,
      kind,
      paramText,
    );

    _patches.add(
      SourcePatch(
        insertion.offset,
        insertion.length,
        insertion.text,
        description: 'Add ${spec.name} parameter to constructor',
      ),
    );
  }
}

/// Describes how constructor parameters are added to empty constructors.
enum ConstructorParamStyle {
  /// Named parameters in braces, e.g. `({required this.foo})`.
  named,

  /// Required positional parameters, e.g. `(this.foo)`.
  positional,

  /// Optional positional parameters in brackets, e.g. `([this.foo])`.
  optionalPositional,
}


/// Describes a field to add and how it maps to a constructor parameter.
class FieldSpec {
  /// Field and constructor parameter name.
  final String name;

  /// Base type without a nullable suffix.
  final String type;

  /// When true, appends `?` to [type] for declarations unless already present.
  final bool isNullable;

  /// Optional initializer expression (source text, not quoted).
  final String? defaultValue;

  /// Whether the field is declared `final`.
  final bool isFinal;

  /// Whether the field is declared `const`.
  final bool isConst;

  /// Whether the field is declared `static`.
  final bool isStatic;

  /// Creates a field specification.
  const FieldSpec({
    required this.name,
    required this.type,
    this.isNullable = false,
    this.defaultValue,
    this.isFinal = true,
    this.isConst = false,
    this.isStatic = false,
  });

  /// Type string for field and non-`this` constructor parameters.
  String get declarationType {
    final base = type.trim();
    if (!isNullable) return base;
    if (base.endsWith('?')) return base;
    return '$base?';
  }
}

/// Per-call overrides when wiring a field to a constructor.
class FieldConstructorArgs {
  /// Overrides [CodemodPreferences.emptyConstructorStyle] for empty constructors.
  final ConstructorParamStyle style;

  /// When true, emits `this.name`; otherwise emits `type name`.
  final bool thisPrefix;

  /// Creates constructor wiring overrides.
  const FieldConstructorArgs({this.style = ConstructorParamStyle.named, this.thisPrefix = true});
  const FieldConstructorArgs.optional({ConstructorParamStyle? style, bool? thisPrefix}) : 
  this(style: style ?? ConstructorParamStyle.named, thisPrefix: thisPrefix ?? true);
}

