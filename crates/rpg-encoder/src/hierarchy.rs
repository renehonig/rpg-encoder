//! Structure Reorganization — apply hierarchy assignments to the RPG graph.

use rpg_core::graph::RPGraph;
use std::collections::HashMap;

/// Apply hierarchy assignments to the RPG graph.
///
/// Keys in `assignments` can be entity IDs (`file:name`) or bare names.
/// Prefers direct ID lookup; falls back to name-based matching only when
/// the name is unambiguous (exactly one entity has that name).
///
/// Paper §9.1.2: When a Module entity receives a hierarchy path, all entities
/// in the same file inherit that path (file-level granularity assignment).
pub fn apply_hierarchy(graph: &mut RPGraph, assignments: &HashMap<String, String>) {
    // Build name → IDs index for fallback matching
    let name_to_ids: HashMap<String, Vec<String>> = {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for (id, entity) in &graph.entities {
            map.entry(entity.name.clone()).or_default().push(id.clone());
        }
        map
    };

    for (key, path) in assignments {
        // 1. Try direct entity ID lookup (preferred — unambiguous)
        let entity_id = if graph.entities.contains_key(key) {
            Some(key.clone())
        } else if let Some(ids) = name_to_ids.get(key) {
            // 2. Bare name fallback — only if unambiguous (exactly one match)
            if ids.len() == 1 {
                Some(ids[0].clone())
            } else {
                // Ambiguous name (e.g., "new", "default") — skip to avoid wrong assignment
                None
            }
        } else {
            None
        };

        if let Some(id) = entity_id {
            // Check if this is a Module entity — if so, propagate path to all file siblings
            let is_module = graph
                .entities
                .get(&id)
                .is_some_and(|e| e.kind == rpg_core::graph::EntityKind::Module);

            if is_module {
                // Paper §9.1.2: file-level assignment — all entities in this file
                // inherit the Module's hierarchy path.
                let file = graph.entities.get(&id).map(|e| e.file.clone());
                if let Some(file) = file {
                    let sibling_ids: Vec<String> =
                        graph.file_index.get(&file).cloned().unwrap_or_default();

                    for sibling_id in &sibling_ids {
                        if let Some(entity) = graph.entities.get_mut(sibling_id) {
                            entity.hierarchy_path = path.clone();
                        }
                        graph.insert_into_hierarchy(path, sibling_id);
                    }
                }
            } else {
                // Individual entity assignment (backward compat for evolution incremental updates)
                if let Some(entity) = graph.entities.get_mut(&id) {
                    entity.hierarchy_path = path.clone();
                }
                graph.insert_into_hierarchy(path, &id);
            }
        }
    }
}
