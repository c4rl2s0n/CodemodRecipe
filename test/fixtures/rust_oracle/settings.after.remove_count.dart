import 'package:foo/bar.dart';

/// Application settings container.
class Settings {
  Settings({this.name});

  final String name;
  

  void update() {
    print('hi');
  }
}

class Widget {
  Widget({required this.key});

  final String key;
}

class DerivedSettings extends BaseSettings {
  DerivedSettings();

  final int id = 0;
}

void main() {
  print('main function');
}

void build() {
  return MaterialApp(home: Container());
}

class BaseSettings {}
