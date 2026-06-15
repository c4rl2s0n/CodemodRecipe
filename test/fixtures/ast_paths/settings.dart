import 'package:foo/bar.dart';

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

void build() {
  return MaterialApp(home: Container());
}
