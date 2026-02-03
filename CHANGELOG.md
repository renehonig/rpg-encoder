# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.1.0] - 2026-02-06

Initial public release. Independent Rust implementation of the RPG-Encoder framework
described in [arXiv:2602.02084](https://arxiv.org/abs/2602.02084).

### Core Pipeline

- **Semantic Lifting** (Phase 1) — Parse code with tree-sitter, enrich entities with
  verb-object features via the connected coding agent's MCP interactive protocol
  (`get_entities_for_lifting` → `submit_lift_results`)
- **Structure Reorganization** (Phase 2) — Agent discovers functional domains and builds
  a 3-level semantic hierarchy (`build_semantic_hierarchy` → `submit_hierarchy`)
- **Artifact Grounding** (Phase 3) — Anchor hierarchy nodes to directories via LCA algorithm,
  resolve cross-file dependency edges (imports, invocations, inheritance)

### Language Support

- 8 languages via tree-sitter: Python, Rust, TypeScript, JavaScript, Go, Java, C, C++
- Per-language entity extraction (functions, classes, methods, structs, traits, interfaces)
- Per-language dependency resolution (imports, calls, inheritance, trait impls)

### Incremental Evolution

- Git-diff-based incremental updates (Algorithms 2-4 from the paper)
- Deletion pruning with hierarchy cleanup
- Modification with semantic drift detection (Jaccard distance)
- Structural entity insertion with dependency re-resolution
- Modified entities tracked for agent re-lifting

### Navigation & Search

- **search_node** — Intent-based search across 3 modes: features, snippets, auto
- **fetch_node** — Entity details with source code, dependencies, hierarchy context; V_H hierarchy
  node fetch support
- **explore_rpg** — Dependency graph traversal (upstream/downstream/both) with configurable depth
  and edge filtering by kind (imports, invokes, inherits, contains)
- **rpg_info** — Graph statistics, hierarchy overview, per-area lifting coverage
- Cross-view traversal between V_L (code entities) and V_H (hierarchy nodes)
- TOON (Token-Oriented Object Notation) serialization for token-efficient LLM output

### MCP Server

- 15 tools: `build_rpg`, `search_node`, `fetch_node`, `explore_rpg`, `rpg_info`, `update_rpg`,
  `lifting_status`, `get_entities_for_lifting`, `submit_lift_results`, `finalize_lifting`,
  `get_files_for_synthesis`, `submit_file_syntheses`, `build_semantic_hierarchy`,
  `submit_hierarchy`, `reload_rpg`
- Semantic lifting via connected coding agent — no API keys, no local LLMs, no setup
- Staleness detection on read-only tools (prepends `[stale]` notice when graph is behind HEAD)
- Auto-update on server startup when graph is stale (structural-only, sub-second)

### CLI

- Commands: `build`, `update`, `search`, `fetch`, `explore`, `info`, `diff`, `validate`,
  `export`, `hook`
- `--include` / `--exclude` glob filtering for builds
- `--since` commit override for incremental updates
- Pre-commit hook: `rpg-encoder hook install` (auto-updates and stages graph on commit)
- Graph export as DOT (Graphviz) or Mermaid flowchart
- Graph integrity validation

### Storage

- RPG graph committed to repos (`.rpg/graph.json`) — collaborators get instant semantic
  search without rebuilding
- Self-contained `.rpg/.gitignore` (ignores `config.toml`)
- Optional zstd compression for large graphs

### Configuration

- `.rpg/config.toml` with sections: `[encoding]`, `[navigation]`, `[storage]`
- Environment variable overrides (`RPG_BATCH_SIZE`, `RPG_SEARCH_LIMIT`, etc.)
- Feature normalization: trim, lowercase, sort+dedup per paper spec

### Code Quality

- Modular crate architecture: rpg-core, rpg-parser, rpg-encoder, rpg-nav, rpg-cli, rpg-mcp
- Clean `cargo clippy --workspace --all-targets -- -D warnings`
- Clean `cargo fmt --all -- --check`
