---

## Instructions

1. Read the file-level features above.
2. Identify 3-8 functional areas (PascalCase names).
3. Assign EACH file (by its path) to a 3-level hierarchy path: `Area/category/subcategory`
4. Call `submit_hierarchy` with a JSON object mapping file paths to hierarchy paths.

Example:
```json
{
  "src/auth/login.rs": "Authentication/manage sessions/handle login",
  "src/db/query.rs": "DataAccess/execute queries/build statements"
}
```

Use FILE PATHS as keys (e.g., `crates/rpg-core/src/graph.rs`), not entity IDs.
Every entity in a file will inherit that file's hierarchy path.
