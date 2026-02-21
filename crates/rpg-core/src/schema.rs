//! JSON schema validation and version handling for RPG files.
//!
//! Uses semver-compatible version checking: graphs are accepted if their
//! major version matches the current schema. Minor/patch differences are
//! handled by `migrate()`.

use crate::graph::RPGraph;
use anyhow::{Context, Result};
use semver::Version;

const CURRENT_VERSION: &str = "2.2.0";

/// Validate an RPGraph's schema version using semver compatibility.
///
/// Accepts any version with the same major version as CURRENT_VERSION.
/// For example, if current is 2.0.0, accepts 2.0.0, 2.1.0, 2.0.3, etc.
/// Rejects 1.x.x or 3.x.x.
pub fn validate_version(graph: &RPGraph) -> Result<()> {
    let current = Version::parse(CURRENT_VERSION).context("invalid CURRENT_VERSION constant")?;
    let found = Version::parse(&graph.version)
        .with_context(|| format!("invalid RPG version string: {}", graph.version))?;

    if found.major != current.major {
        anyhow::bail!(
            "RPG major version mismatch: schema requires {}.x.x, found {}",
            current.major,
            graph.version
        );
    }

    Ok(())
}

/// Apply any necessary migrations to bring the graph up to the current version.
///
/// Called after deserialization when the version is compatible but not identical.
/// Currently a no-op — add transformation logic here when schema changes are made.
pub fn migrate(graph: &mut RPGraph) -> Result<()> {
    let current = Version::parse(CURRENT_VERSION)?;
    let found = Version::parse(&graph.version)?;

    if found < current {
        // v2.2.0: normalize backslash entity IDs to forward slashes (Windows compat fix)
        if found < Version::new(2, 2, 0) {
            migrate_normalize_entity_ids(graph);
        }
        graph.version = CURRENT_VERSION.to_string();
    }

    Ok(())
}

/// Migrate entity IDs from backslash paths to forward-slash paths.
///
/// On Windows, older graphs stored entity IDs with backslashes
/// (e.g. `src\auth\login.py:validate`). This normalizes them to
/// forward slashes so lookups from MCP tools match consistently.
fn migrate_normalize_entity_ids(graph: &mut RPGraph) {
    fn norm(id: &str) -> String {
        id.replace('\\', "/")
    }

    fn fix_hierarchy(
        node: &mut crate::graph::HierarchyNode,
        remap: &std::collections::HashMap<String, String>,
    ) {
        for id in &mut node.entities {
            if let Some(new_id) = remap.get(id.as_str()) {
                *id = new_id.clone();
            }
        }
        for child in node.children.values_mut() {
            fix_hierarchy(child, remap);
        }
    }

    // 1. Rebuild entities map with normalized keys + id fields
    let old_entities = std::mem::take(&mut graph.entities);
    let mut id_remap: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for (old_key, mut entity) in old_entities {
        let new_key = norm(&old_key);
        entity.id = new_key.clone();
        if old_key != new_key {
            id_remap.insert(old_key, new_key.clone());
        }
        graph.entities.insert(new_key, entity);
    }

    // Nothing was remapped — graph already uses forward slashes
    if id_remap.is_empty() {
        return;
    }

    // 2. Update file_index value lists
    for ids in graph.file_index.values_mut() {
        for id in ids.iter_mut() {
            if let Some(new_id) = id_remap.get(id.as_str()) {
                *id = new_id.clone();
            }
        }
    }

    // 3. Update edge source/target
    for edge in &mut graph.edges {
        if let Some(new_id) = id_remap.get(&edge.source) {
            edge.source = new_id.clone();
        }
        if let Some(new_id) = id_remap.get(&edge.target) {
            edge.target = new_id.clone();
        }
    }

    // 4. Update hierarchy entity lists (recursive)
    for node in graph.hierarchy.values_mut() {
        fix_hierarchy(node, &id_remap);
    }
}

/// Serialize an RPGraph to a pretty-printed JSON string.
///
/// Edges are sorted by (source, target, kind) for deterministic output,
/// ensuring minimal git diffs when the graph is re-saved.
pub fn to_json(graph: &RPGraph) -> Result<String> {
    let mut graph = graph.clone();
    graph.edges.sort();
    serde_json::to_string_pretty(&graph).context("failed to serialize RPG to JSON")
}

/// Deserialize an RPGraph from a JSON string.
pub fn from_json(json: &str) -> Result<RPGraph> {
    let mut graph: RPGraph =
        serde_json::from_str(json).context("failed to deserialize RPG from JSON")?;
    validate_version(&graph)?;
    migrate(&mut graph)?;
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_with_version(version: &str) -> RPGraph {
        let json = format!(
            r#"{{
                "version": "{}",
                "created_at": "2025-01-01T00:00:00Z",
                "updated_at": "2025-01-01T00:00:00Z",
                "metadata": {{
                    "language": "rust",
                    "total_files": 0,
                    "total_entities": 0,
                    "functional_areas": 0,
                    "total_edges": 0,
                    "dependency_edges": 0,
                    "containment_edges": 0
                }},
                "entities": {{}},
                "edges": [],
                "hierarchy": {{}},
                "file_index": {{}}
            }}"#,
            version
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_exact_version_match() {
        let graph = graph_with_version("2.0.0");
        assert!(validate_version(&graph).is_ok());
    }

    #[test]
    fn test_compatible_minor_bump() {
        let graph = graph_with_version("2.1.0");
        assert!(validate_version(&graph).is_ok());
    }

    #[test]
    fn test_compatible_patch_bump() {
        let graph = graph_with_version("2.0.3");
        assert!(validate_version(&graph).is_ok());
    }

    #[test]
    fn test_incompatible_major_bump() {
        let graph = graph_with_version("3.0.0");
        assert!(validate_version(&graph).is_err());
    }

    #[test]
    fn test_incompatible_old_major() {
        let graph = graph_with_version("1.0.0");
        assert!(validate_version(&graph).is_err());
    }

    #[test]
    fn test_migrate_updates_version() {
        let mut graph = graph_with_version("2.0.0");
        graph.version = "2.0.0".to_string();
        assert!(migrate(&mut graph).is_ok());
        assert_eq!(graph.version, CURRENT_VERSION);
    }

    #[test]
    fn test_invalid_version_string() {
        let graph = graph_with_version("not-a-version");
        assert!(validate_version(&graph).is_err());
    }

    #[test]
    fn test_migrate_normalizes_backslash_ids() {
        use crate::graph::{
            DependencyEdge, EdgeKind, Entity, EntityDeps, EntityKind, HierarchyNode,
        };
        use std::path::PathBuf;

        let mut graph = graph_with_version("2.1.0");

        // Insert entity with backslash ID (as Windows would produce)
        let old_id = r"src\auth\login.py:validate".to_string();
        graph.entities.insert(
            old_id.clone(),
            Entity {
                id: old_id.clone(),
                kind: EntityKind::Function,
                name: "validate".to_string(),
                file: PathBuf::from("src/auth/login.py"),
                line_start: 1,
                line_end: 10,
                parent_class: None,
                semantic_features: vec!["validates user credentials".to_string()],
                feature_source: Some("llm".to_string()),
                hierarchy_path: String::new(),
                deps: EntityDeps::default(),
                signature: None,
            },
        );

        // file_index references the old ID
        graph
            .file_index
            .insert(PathBuf::from("src/auth/login.py"), vec![old_id.clone()]);

        // edge references the old ID
        graph.edges.push(DependencyEdge {
            source: old_id.clone(),
            target: "other:target".to_string(),
            kind: EdgeKind::Invokes,
        });

        // hierarchy references the old ID
        let mut node = HierarchyNode::new("Auth");
        node.entities.push(old_id.clone());
        graph.hierarchy.insert("h:Auth".to_string(), node);

        // Run migration
        migrate(&mut graph).unwrap();

        let new_id = "src/auth/login.py:validate";

        // Entity moved to new key
        assert!(graph.entities.contains_key(new_id));
        assert!(!graph.entities.contains_key(&old_id));
        assert_eq!(graph.entities[new_id].id, new_id);
        // Semantic features preserved
        assert_eq!(
            graph.entities[new_id].semantic_features,
            vec!["validates user credentials"]
        );

        // file_index updated
        let file_ids = &graph.file_index[&PathBuf::from("src/auth/login.py")];
        assert_eq!(file_ids, &[new_id]);

        // edge updated
        assert_eq!(graph.edges[0].source, new_id);

        // hierarchy updated
        assert_eq!(graph.hierarchy["h:Auth"].entities, vec![new_id]);

        // version bumped
        assert_eq!(graph.version, CURRENT_VERSION);
    }
}
