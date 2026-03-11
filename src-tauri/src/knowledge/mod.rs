use crate::config::Settings;
use crate::storage::{Activity, Database, KnowledgeEdge, KnowledgeNode};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::{debug, info};

pub struct KnowledgeGraph {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
}

impl KnowledgeGraph {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>) -> Self {
        Self { db, settings }
    }

    pub fn process_activity(
        &self,
        activity: &Activity,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let node_id = match activity.activity_type.as_str() {
            "app_focus" => {
                if let Some(bundle_id) = &activity.app_bundle_id {
                    self.get_or_create_node("app", bundle_id, activity.app_name.as_deref())?
                } else {
                    return Ok(());
                }
            }
            "screenshot" => self.get_or_create_node("screenshot", &activity.id, None)?,
            "clipboard" => self.get_or_create_node("clipboard", &activity.id, None)?,
            _ => {
                if activity.url.is_some() {
                    self.get_or_create_node("url", activity.url.as_ref().unwrap(), None)?
                } else if activity.file_path.is_some() {
                    self.get_or_create_node("file", activity.file_path.as_ref().unwrap(), None)?
                } else {
                    self.get_or_create_node("activity", &activity.id, activity.title.as_deref())?
                }
            }
        };

        if let Some(session_id) = &activity.session_id {
            let session_node = self.get_or_create_node("session", session_id, None)?;
            self.create_edge(&session_node, &node_id, "contains")?;
        }

        Ok(())
    }

    fn get_or_create_node(
        &self,
        node_type: &str,
        identifier: &str,
        label: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let nodes = self.db.get_knowledge_nodes(Some(node_type))?;

        for node in nodes {
            if node.identifier == identifier {
                return Ok(node.id);
            }
        }

        let node = KnowledgeNode {
            id: uuid::Uuid::new_v4().to_string(),
            node_type: node_type.to_string(),
            identifier: identifier.to_string(),
            label: label.map(String::from),
            properties: None,
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
        };

        self.db.insert_knowledge_node(&node)?;
        debug!("Created knowledge node: {} - {}", node_type, identifier);

        Ok(node.id)
    }

    fn create_edge(
        &self,
        source_id: &str,
        target_id: &str,
        edge_type: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let edge = KnowledgeEdge::new(source_id, target_id, edge_type);
        self.db.insert_knowledge_edge(&edge)?;
        debug!(
            "Created knowledge edge: {} -> {} ({})",
            source_id, target_id, edge_type
        );
        Ok(())
    }

    pub fn get_related_nodes(
        &self,
        node_id: &str,
    ) -> Result<Vec<RelatedNode>, Box<dyn std::error::Error + Send + Sync>> {
        let edges = self.db.get_knowledge_edges(Some(node_id))?;

        let mut related = Vec::new();

        for edge in edges {
            let other_id = if edge.source_id == node_id {
                edge.target_id.clone()
            } else {
                edge.source_id.clone()
            };

            let nodes = self.db.get_knowledge_nodes(None)?;
            if let Some(node) = nodes.iter().find(|n| n.id == other_id) {
                related.push(RelatedNode {
                    id: node.id.clone(),
                    node_type: node.node_type.clone(),
                    label: node
                        .label
                        .clone()
                        .unwrap_or_else(|| node.identifier.clone()),
                    identifier: node.identifier.clone(),
                    edge_type: edge.edge_type.clone(),
                    weight: edge.weight,
                });
            }
        }

        Ok(related)
    }

    pub fn get_graph_data(&self) -> Result<GraphData, Box<dyn std::error::Error + Send + Sync>> {
        let nodes = self.db.get_knowledge_nodes(None)?;
        let edges = self.db.get_knowledge_edges(None)?;

        let graph_nodes: Vec<GraphNode> = nodes
            .iter()
            .map(|n| GraphNode {
                id: n.id.clone(),
                node_type: n.node_type.clone(),
                label: n.label.clone().unwrap_or_else(|| n.identifier.clone()),
            })
            .collect();

        let graph_edges: Vec<GraphEdge> = edges
            .iter()
            .map(|e| GraphEdge {
                id: e.id.clone(),
                source: e.source_id.clone(),
                target: e.target_id.clone(),
                edge_type: e.edge_type.clone(),
            })
            .collect();

        Ok(GraphData {
            nodes: graph_nodes,
            edges: graph_edges,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelatedNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub identifier: String,
    pub edge_type: String,
    pub weight: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: String,
}
