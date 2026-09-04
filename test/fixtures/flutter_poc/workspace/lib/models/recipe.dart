class Recipe {
  final String title;
  final String subtitle;

  Recipe copy() => Recipe(title: title, subtitle: subtitle);
}
