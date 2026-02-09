//! Utility functions shared across tool handlers.

/// Truncate source code to `max_lines`, preserving the signature and start of the body.
/// Appends a `(truncated)` note if the source exceeds the limit.
pub(crate) fn truncate_source(source: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() <= max_lines {
        return source.to_string();
    }
    let mut out: String = lines[..max_lines].join("\n");
    out.push_str(&format!(
        "\n    // ... ({} more lines, truncated for context)",
        lines.len() - max_lines
    ));
    out
}

/// Parse a comma-separated entity type filter string into EntityKind values.
///
/// Accepts entity names: function, class, method, page, layout, component,
/// hook, store, file, module.
/// "file" is an alias for Module (file-level entity nodes, V_L).
///
/// Note: "directory" is not a V_L entity kind — hierarchy nodes (V_H) are
/// traversed via Contains edges but are not subject to entity_type_filter.
pub(crate) fn parse_entity_type_filter(filter: &str) -> Vec<rpg_core::graph::EntityKind> {
    filter
        .split(',')
        .filter_map(|s| match s.trim().to_lowercase().as_str() {
            "function" => Some(rpg_core::graph::EntityKind::Function),
            "class" => Some(rpg_core::graph::EntityKind::Class),
            "method" => Some(rpg_core::graph::EntityKind::Method),
            "page" => Some(rpg_core::graph::EntityKind::Page),
            "layout" => Some(rpg_core::graph::EntityKind::Layout),
            "component" => Some(rpg_core::graph::EntityKind::Component),
            "hook" => Some(rpg_core::graph::EntityKind::Hook),
            "store" => Some(rpg_core::graph::EntityKind::Store),
            "module" | "file" => Some(rpg_core::graph::EntityKind::Module),
            "controller" => Some(rpg_core::graph::EntityKind::Controller),
            "model" => Some(rpg_core::graph::EntityKind::Model),
            "service" => Some(rpg_core::graph::EntityKind::Service),
            "middleware" => Some(rpg_core::graph::EntityKind::Middleware),
            "route" => Some(rpg_core::graph::EntityKind::Route),
            "test" => Some(rpg_core::graph::EntityKind::Test),
            _ => None,
        })
        .collect()
}
