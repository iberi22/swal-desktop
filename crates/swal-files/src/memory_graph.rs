//! memory_graph.rs
//! Semantic Memory & Knowledge Graph Client in Rust

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_file_node_creation() {
        let node = SemanticFileNode {
            path: PathBuf::from("/home/user/docs/report.pdf"),
            mime_type: "application/pdf".to_string(),
            tags: vec!["finance".to_string(), "quarterly".to_string()],
            embedding_id: Some("vec-123".to_string()),
            summary: Some("Q2 Financial Report".to_string()),
            last_indexed_timestamp: 1700000000,
        };

        assert_eq!(node.mime_type, "application/pdf");
        assert_eq!(node.tags.len(), 2);
        assert_eq!(node.embedding_id.as_deref(), Some("vec-123"));
    }

    #[test]
    fn test_knowledge_graph_add_and_get_node() {
        let mut graph = FileKnowledgeGraph::new();
        let node = SemanticFileNode {
            path: PathBuf::from("/code/main.rs"),
            mime_type: "text/x-rust".to_string(),
            tags: vec!["rust".to_string(), "entrypoint".to_string()],
            embedding_id: None,
            summary: None,
            last_indexed_timestamp: 100,
        };

        graph.add_node(node);
        assert_eq!(graph.node_count(), 1);
        let retrieved = graph.get_node(Path::new("/code/main.rs"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().mime_type, "text/x-rust");
    }

    #[test]
    fn test_knowledge_graph_relationships_and_neighbors() {
        let mut graph = FileKnowledgeGraph::new();
        let src = PathBuf::from("/src/lib.rs");
        let tgt = PathBuf::from("/src/main.rs");

        graph.add_edge(src.clone(), tgt.clone(), RelationshipKind::DependsOn, 1.0);
        assert_eq!(graph.edge_count(), 1);

        let related = graph.find_related(&src);
        assert_eq!(related, vec![tgt.clone()]);

        let related_back = graph.find_related(&tgt);
        assert_eq!(related_back, vec![src.clone()]);
    }

    #[test]
    fn test_tag_search_and_filtering() {
        let mut graph = FileKnowledgeGraph::new();
        graph.add_node(SemanticFileNode {
            path: PathBuf::from("/data/sales.csv"),
            mime_type: "text/csv".to_string(),
            tags: vec!["sales".to_string(), "analytics".to_string()],
            embedding_id: None,
            summary: None,
            last_indexed_timestamp: 1,
        });
        graph.add_node(SemanticFileNode {
            path: PathBuf::from("/data/users.json"),
            mime_type: "application/json".to_string(),
            tags: vec!["users".to_string(), "auth".to_string()],
            embedding_id: None,
            summary: None,
            last_indexed_timestamp: 2,
        });

        let results = graph.search_by_tag("sales");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, PathBuf::from("/data/sales.csv"));
    }

    #[test]
    fn test_knowledge_graph_serde_roundtrip() {
        let mut graph = FileKnowledgeGraph::new();
        graph.add_node(SemanticFileNode {
            path: PathBuf::from("/test.txt"),
            mime_type: "text/plain".to_string(),
            tags: vec!["test".to_string()],
            embedding_id: None,
            summary: None,
            last_indexed_timestamp: 0,
        });

        let json = serde_json::to_string(&graph).expect("serialize");
        let deserialized: FileKnowledgeGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.node_count(), 1);
    }
}
