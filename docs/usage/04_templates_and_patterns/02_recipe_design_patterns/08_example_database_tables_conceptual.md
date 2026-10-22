# Example: Database tables (conceptual)

The same taxonomy applies when schema lives in Dart (Drift, Floor, Isar) or when
you set `language` on ops for `.sql` files ([language-support.md](../../03_queries_and_languages/02_language_support/00_index.md)).

| User intent | Recipe kind | Notes |
|-------------|-------------|-------|
| New table | `create_table_users` or `scaffold_drift_table` | `c:create` with table template |
| Add column | `add_column_users_email` | `c:insert` targeting the table definition |
| Remove column | `remove_column_users_email` | `c:remove` on the field member |
