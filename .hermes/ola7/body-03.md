# [Ola 7.03] feat-swal-63 — Semantic Memory & Knowledge Graph Client in Rust

> Ola 7 — [Files/Memory/Xavier-PGHEART].
> Labels: `ola7`, `wave-7`

---

## Current State (MEDIBLE)
- Feature: `feat-swal-63` at 0% in `.gitcore/features.json`
- Module `crates/swal-files/src/memory_graph.rs` will be created.
- Existing tests in `crates/swal-files`: 154 passing tests.

## Desired State (DELTA)
- **Specific Addition**: Implement `crates/swal-files/src/memory_graph.rs` providing semantic file tagging, knowledge graph relationship indexing, and Xavier/PGHEART memory client integration.
- **File Target**: `crates/swal-files/src/memory_graph.rs`

## Web Research Required
1. search: "knowledge graph semantic tagging file manager Rust"
2. search: "vector search pgvector memory store client Rust"
3. search: "file relationship graph node edge directed graph Rust"

## Acceptance Criteria (VERIFICABLES POR COMANDO)
- [ ] `cargo check -p swal-files` — 0 errors
- [ ] `cargo test -p swal-files` — all tests pass
- [ ] `grep -rn "FileKnowledgeGraph" crates/swal-files/src/memory_graph.rs` >= 1 match
- [ ] `grep -rn "SemanticFileNode" crates/swal-files/src/memory_graph.rs` >= 1 match
- [ ] `grep -rn "RelationshipKind" crates/swal-files/src/memory_graph.rs` >= 1 match

## Exact Code Blueprint & Signatures

```rust
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    DependsOn,
    ImportedBy,
    ParentDirectory,
    AssociatedArtifact,
    SemanticSimilarity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFileNode {
    pub path: PathBuf,
    pub mime_type: String,
    pub tags: Vec<String>,
    pub embedding_id: Option<String>,
    pub summary: Option<String>,
    pub last_indexed_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: PathBuf,
    pub target: PathBuf,
    pub kind: RelationshipKind,
    pub weight: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileKnowledgeGraph {
    nodes: HashMap<PathBuf, SemanticFileNode>,
    edges: Vec<GraphEdge>,
}

impl FileKnowledgeGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: SemanticFileNode) {
        self.nodes.insert(node.path.clone(), node);
    }

    pub fn get_node(&self, path: &Path) -> Option<&SemanticFileNode> {
        self.nodes.get(path)
    }

    pub fn add_edge(&mut self, source: PathBuf, target: PathBuf, kind: RelationshipKind, weight: f32) {
        self.edges.push(GraphEdge {
            source,
            target,
            kind,
            weight,
        });
    }

    pub fn find_related(&self, path: &Path) -> Vec<PathBuf> {
        let mut related = Vec::new();
        for edge in &self.edges {
            if edge.source == path {
                related.push(edge.target.clone());
            } else if edge.target == path {
                related.push(edge.source.clone());
            }
        }
        related
    }

    pub fn search_by_tag(&self, tag: &str) -> Vec<&SemanticFileNode> {
        self.nodes
            .values()
            .filter(|n| n.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }
}
```

## Unit Tests Requirements
1. `test_semantic_file_node_creation`
2. `test_knowledge_graph_add_and_get_node`
3. `test_knowledge_graph_relationships_and_neighbors`
4. `test_tag_search_and_filtering`
5. `test_knowledge_graph_serde_roundtrip`

## Anti-Hallucination Guard
- Do NOT edit other crates or shared files.
- Place all implementation strictly inside `crates/swal-files/src/memory_graph.rs`.
