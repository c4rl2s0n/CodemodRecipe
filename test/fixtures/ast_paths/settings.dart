import 'package:foo/bar.dart';

/// Application settings container.
class Settings {
  Settings({this.name});

  final String name;
  final int count = 0;

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

void build() {
  return MaterialApp(home: Container());
}
