//! Integration test for the incremental update pipeline.
//!
//! Tests the full cycle: build graph → modify fixture files → detect changes →
//! apply incremental updates → verify graph integrity.
//! Uses the Python fixture project from tests/fixtures/python_project.

use rpg_core::graph::{EntityKind, RPGraph};
use rpg_encoder::evolution::{apply_deletions, apply_renames};
use rpg_parser::entities::{RawEntity, extract_entities};
use rpg_parser::languages::Language;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/python_project")
}

fn collect_fixture_files(root: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    collect_recursive(root, root, &mut files);
    files
}

fn collect_recursive(base: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_recursive(base, &path, out);
        } else if path.extension().is_some_and(|e| e == "py") {
            let rel = path.strip_prefix(base).unwrap().to_path_buf();
            let source = std::fs::read_to_string(&path).unwrap();
            out.push((rel, source));
        }
    }
}

fn build_fixture_graph() -> RPGraph {
    let root = fixture_root();
    let files = collect_fixture_files(&root);

    let mut all_entities: Vec<RawEntity> = Vec::new();
    for (rel_path, source) in &files {
        let raw = extract_entities(rel_path, source, Language::Python);
        all_entities.extend(raw);
    }

    let mut graph = RPGraph::new("python");
    for raw in all_entities {
        graph.insert_entity(raw.into_entity());
    }
    graph.create_module_entities();
    graph.build_file_path_hierarchy();
    rpg_encoder::grounding::resolve_dependencies(&mut graph);
    graph.assign_hierarchy_ids();
    graph.aggregate_hierarchy_features();
    graph.materialize_containment_edges();
    rpg_encoder::grounding::ground_hierarchy(&mut graph);
    graph.refresh_metadata();
    graph
}

/// Verify graph invariants that should hold after any update.
fn verify_graph_integrity(graph: &RPGraph) {
    // Every entity in entities map should be in file_index
    for (id, entity) in &graph.entities {
        let ids = graph.file_index.get(&entity.file);
        assert!(
            ids.is_some_and(|ids| ids.contains(id)),
            "entity {} not in file_index for {}",
            id,
            entity.file.display()
        );
    }

    // Every entity in file_index should be in entities map
    for (file, ids) in &graph.file_index {
        for id in ids {
            assert!(
                graph.entities.contains_key(id),
                "file_index entry {} for {} has no entity",
                id,
                file.display()
            );
        }
    }

    // Non-containment edges should reference existing entities
    for edge in &graph.edges {
        if edge.kind != rpg_core::graph::EdgeKind::Contains {
            assert!(
                graph.entities.contains_key(&edge.source),
                "dangling edge source: {}",
                edge.source
            );
            assert!(
                graph.entities.contains_key(&edge.target),
                "dangling edge target: {}",
                edge.target
            );
        }
    }
}

// --- Incremental update tests ---

#[test]
fn test_incremental_delete_file() {
    let mut graph = build_fixture_graph();
    let initial_count = graph.entities.len();

    // Count entities in src/models.py
    let models_count = graph
        .entities
        .values()
        .filter(|e| e.file == Path::new("src/models.py"))
        .count();
    assert!(
        models_count > 0,
        "fixture should have entities in models.py"
    );

    // Delete models.py entities
    let removed = apply_deletions(&mut graph, &[PathBuf::from("src/models.py")]);
    assert_eq!(removed, models_count);

    graph.refresh_metadata();
    assert_eq!(graph.entities.len(), initial_count - models_count);

    // No entities should reference the deleted file
    assert!(
        !graph
            .entities
            .values()
            .any(|e| e.file == Path::new("src/models.py")),
        "deleted file entities should be gone"
    );

    verify_graph_integrity(&graph);
}

#[test]
fn test_incremental_rename_file() {
    let mut graph = build_fixture_graph();

    // Find entities in auth/login.py
    let login_entities_before: Vec<String> = graph
        .entities
        .values()
        .filter(|e| e.file == Path::new("src/auth/login.py"))
        .map(|e| e.id.clone())
        .collect();
    assert!(!login_entities_before.is_empty());

    // Rename auth/login.py → auth/authentication.py
    let (migrated, renamed) = apply_renames(
        &mut graph,
        &[(
            PathBuf::from("src/auth/login.py"),
            PathBuf::from("src/auth/authentication.py"),
        )],
    );

    assert_eq!(migrated, 1);
    assert_eq!(renamed, login_entities_before.len());

    // Entities should now have the new file path
    for id in &login_entities_before {
        if let Some(entity) = graph.entities.get(id) {
            assert_eq!(
                entity.file,
                PathBuf::from("src/auth/authentication.py"),
                "entity {} should have updated file path",
                id
            );
        }
    }

    // Old file should not be in file_index
    assert!(
        !graph
            .file_index
            .contains_key(&PathBuf::from("src/auth/login.py"))
    );
    // New file should be in file_index
    assert!(
        graph
            .file_index
            .contains_key(&PathBuf::from("src/auth/authentication.py"))
    );

    verify_graph_integrity(&graph);
}

#[test]
fn test_incremental_add_new_file() {
    let mut graph = build_fixture_graph();
    let initial_count = graph.entities.len();

    // Simulate adding a new file: parse it and insert entities
    let new_source = r#"
def send_email(to: str, subject: str, body: str):
    """Send an email message."""
    pass

def format_template(template: str, **kwargs) -> str:
    """Format an email template with variables."""
    return template.format(**kwargs)
"#;

    let new_path = PathBuf::from("src/email.py");
    let new_entities = extract_entities(&new_path, new_source, Language::Python);
    assert!(
        new_entities.len() >= 2,
        "should extract at least 2 functions"
    );

    for raw in new_entities {
        graph.insert_entity(raw.into_entity());
    }

    graph.refresh_metadata();
    assert!(graph.entities.len() > initial_count);

    // New entities should be searchable
    let entity_names: Vec<&str> = graph.entities.values().map(|e| e.name.as_str()).collect();
    assert!(entity_names.contains(&"send_email"));
    assert!(entity_names.contains(&"format_template"));

    verify_graph_integrity(&graph);
}

#[test]
fn test_incremental_modify_file() {
    let mut graph = build_fixture_graph();

    // Simulate modifying config.py: delete old entities, add new ones
    let config_path = PathBuf::from("src/utils/config.py");

    // Remove old config entities
    let removed = apply_deletions(&mut graph, std::slice::from_ref(&config_path));
    assert!(removed > 0);

    // Parse new version with an extra function
    let modified_source = r#"
"""Configuration utilities (v2)."""
import os
from typing import Dict, Any

def load_config(path: str) -> Dict[str, Any]:
    """Load configuration from a TOML file."""
    with open(path) as f:
        return parse_toml(f.read())

def parse_toml(content: str) -> Dict[str, Any]:
    """Parse TOML content into a dictionary."""
    result = {}
    for line in content.strip().split("\n"):
        if "=" in line:
            key, val = line.split("=", 1)
            result[key.strip()] = val.strip().strip('"')
    return result

def get_env_var(name: str, default: str = "") -> str:
    """Get an environment variable with a default."""
    return os.environ.get(name, default)

def validate_config(config: Dict[str, Any]) -> bool:
    """Validate that required config keys are present."""
    required = ["database_url", "secret_key"]
    return all(k in config for k in required)
"#;

    let new_entities = extract_entities(&config_path, modified_source, Language::Python);
    assert!(
        new_entities.len() >= 4,
        "modified config should have >= 4 functions"
    );

    for raw in new_entities {
        graph.insert_entity(raw.into_entity());
    }

    graph.refresh_metadata();

    // Should have the new function
    let entity_names: Vec<&str> = graph.entities.values().map(|e| e.name.as_str()).collect();
    assert!(
        entity_names.contains(&"validate_config"),
        "new function should be present"
    );
    // Original functions should still be there
    assert!(entity_names.contains(&"load_config"));
    assert!(entity_names.contains(&"parse_toml"));
    assert!(entity_names.contains(&"get_env_var"));

    verify_graph_integrity(&graph);
}

#[test]
fn test_incremental_delete_then_rebuild_hierarchy() {
    let mut graph = build_fixture_graph();

    // Delete a file
    apply_deletions(&mut graph, &[PathBuf::from("src/models.py")]);

    // Rebuild hierarchy and verify it's still consistent
    graph.hierarchy.clear();
    graph.build_file_path_hierarchy();
    graph.assign_hierarchy_ids();
    graph.aggregate_hierarchy_features();
    graph.materialize_containment_edges();
    rpg_encoder::grounding::ground_hierarchy(&mut graph);
    graph.refresh_metadata();

    // Hierarchy should still be valid
    assert!(!graph.hierarchy.is_empty());

    // Every entity should have a hierarchy path
    for (id, entity) in &graph.entities {
        if entity.kind != EntityKind::Module {
            assert!(
                !entity.hierarchy_path.is_empty(),
                "entity {} missing hierarchy path after rebuild",
                id
            );
        }
    }

    verify_graph_integrity(&graph);
}

#[test]
fn test_incremental_multiple_operations() {
    let mut graph = build_fixture_graph();

    // 1. Delete models.py
    apply_deletions(&mut graph, &[PathBuf::from("src/models.py")]);

    // 2. Rename login.py → auth_service.py
    apply_renames(
        &mut graph,
        &[(
            PathBuf::from("src/auth/login.py"),
            PathBuf::from("src/auth/auth_service.py"),
        )],
    );

    // 3. Add new file
    let new_source = "def health_check():\n    return True\n";
    let new_path = PathBuf::from("src/health.py");
    let new_entities = extract_entities(&new_path, new_source, Language::Python);
    for raw in new_entities {
        graph.insert_entity(raw.into_entity());
    }

    graph.refresh_metadata();

    // Verify all operations took effect
    assert!(
        !graph
            .entities
            .values()
            .any(|e| e.file == Path::new("src/models.py")),
        "deleted file should be gone"
    );
    assert!(
        graph
            .file_index
            .contains_key(&PathBuf::from("src/auth/auth_service.py")),
        "renamed file should exist"
    );
    assert!(
        graph.entities.values().any(|e| e.name == "health_check"),
        "new function should exist"
    );

    verify_graph_integrity(&graph);
}
