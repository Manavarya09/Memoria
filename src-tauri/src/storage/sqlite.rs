use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(data_dir: &Path) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        std::fs::create_dir_all(data_dir)?;
        let db_path = data_dir.join("memoria.db");

        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        db.init_schema()?;
        info!("Database initialized at {:?}", db_path);

        Ok(db)
    }

    fn init_schema(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();

        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS activities (
                id TEXT PRIMARY KEY,
                activity_type TEXT NOT NULL,
                app_name TEXT,
                app_bundle_id TEXT,
                title TEXT,
                content TEXT,
                file_path TEXT,
                url TEXT,
                timestamp INTEGER NOT NULL,
                session_id TEXT,
                metadata TEXT,
                created_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_activities_timestamp ON activities(timestamp);
            CREATE INDEX IF NOT EXISTS idx_activities_session ON activities(session_id);
            CREATE INDEX IF NOT EXISTS idx_activities_type ON activities(activity_type);
            
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                session_type TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                app_sequence TEXT,
                metadata TEXT,
                created_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_time);
            
            CREATE TABLE IF NOT EXISTS embeddings (
                id TEXT PRIMARY KEY,
                activity_id TEXT NOT NULL,
                embedding BLOB NOT NULL,
                model_name TEXT NOT NULL,
                dimensions INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_embeddings_activity ON embeddings(activity_id);
            
            CREATE TABLE IF NOT EXISTS knowledge_nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                identifier TEXT NOT NULL,
                label TEXT,
                properties TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_knowledge_identifier ON knowledge_nodes(identifier);
            
            CREATE TABLE IF NOT EXISTS knowledge_edges (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                edge_type TEXT NOT NULL,
                weight REAL DEFAULT 1.0,
                properties TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (source_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE,
                FOREIGN KEY (target_id) REFERENCES knowledge_nodes(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_edges_source ON knowledge_edges(source_id);
            CREATE INDEX IF NOT EXISTS idx_edges_target ON knowledge_edges(target_id);
            
            CREATE TABLE IF NOT EXISTS screenshots (
                id TEXT PRIMARY KEY,
                activity_id TEXT,
                image_path TEXT NOT NULL,
                text_content TEXT,
                timestamp INTEGER NOT NULL,
                width INTEGER,
                height INTEGER,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE SET NULL
            );
            
            CREATE INDEX IF NOT EXISTS idx_screenshots_timestamp ON screenshots(timestamp);
            
            CREATE TABLE IF NOT EXISTS pending_embeddings (
                id TEXT PRIMARY KEY,
                activity_id TEXT NOT NULL,
                content TEXT NOT NULL,
                priority INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL,
                retry_count INTEGER DEFAULT 0
            );
            
            CREATE INDEX IF NOT EXISTS idx_pending_priority ON pending_embeddings(priority DESC, created_at ASC);
        "#)?;

        Ok(())
    }

    pub fn insert_activity(
        &self,
        activity: &Activity,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO activities (id, activity_type, app_name, app_bundle_id, title, content, file_path, url, timestamp, session_id, metadata, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
            params![
                activity.id,
                activity.activity_type,
                activity.app_name,
                activity.app_bundle_id,
                activity.title,
                activity.content,
                activity.file_path,
                activity.url,
                activity.timestamp,
                activity.session_id,
                activity.metadata,
                activity.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_activities(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, activity_type, app_name, app_bundle_id, title, content, file_path, url, timestamp, session_id, metadata, created_at 
             FROM activities ORDER BY timestamp DESC LIMIT ?1 OFFSET ?2"
        )?;

        let activities = stmt
            .query_map(params![limit, offset], |row| {
                Ok(Activity {
                    id: row.get(0)?,
                    activity_type: row.get(1)?,
                    app_name: row.get(2)?,
                    app_bundle_id: row.get(3)?,
                    title: row.get(4)?,
                    content: row.get(5)?,
                    file_path: row.get(6)?,
                    url: row.get(7)?,
                    timestamp: row.get(8)?,
                    session_id: row.get(9)?,
                    metadata: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(activities)
    }

    pub fn get_activities_in_range(
        &self,
        start: i64,
        end: i64,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, activity_type, app_name, app_bundle_id, title, content, file_path, url, timestamp, session_id, metadata, created_at 
             FROM activities WHERE timestamp >= ?1 AND timestamp <= ?2 ORDER BY timestamp DESC"
        )?;

        let activities = stmt
            .query_map(params![start, end], |row| {
                Ok(Activity {
                    id: row.get(0)?,
                    activity_type: row.get(1)?,
                    app_name: row.get(2)?,
                    app_bundle_id: row.get(3)?,
                    title: row.get(4)?,
                    content: row.get(5)?,
                    file_path: row.get(6)?,
                    url: row.get(7)?,
                    timestamp: row.get(8)?,
                    session_id: row.get(9)?,
                    metadata: row.get(10)?,
                    created_at: row.get(11)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(activities)
    }

    pub fn insert_session(
        &self,
        session: &Session,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO sessions (id, session_type, start_time, end_time, app_sequence, metadata, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                session.id,
                session.session_type,
                session.start_time,
                session.end_time,
                session.app_sequence,
                session.metadata,
                session.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_sessions(
        &self,
        limit: usize,
    ) -> Result<Vec<Session>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, session_type, start_time, end_time, app_sequence, metadata, created_at 
             FROM sessions ORDER BY start_time DESC LIMIT ?1",
        )?;

        let sessions = stmt
            .query_map(params![limit], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    session_type: row.get(1)?,
                    start_time: row.get(2)?,
                    end_time: row.get(3)?,
                    app_sequence: row.get(4)?,
                    metadata: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sessions)
    }

    pub fn insert_embedding(
        &self,
        embedding: &Embedding,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO embeddings (id, activity_id, embedding, model_name, dimensions, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                embedding.id,
                embedding.activity_id,
                embedding.embedding,
                embedding.model_name,
                embedding.dimensions,
                embedding.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_embedding(
        &self,
        activity_id: &str,
    ) -> Result<Option<Embedding>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, activity_id, embedding, model_name, dimensions, created_at 
             FROM embeddings WHERE activity_id = ?1",
        )?;

        let mut rows = stmt.query(params![activity_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Embedding {
                id: row.get(0)?,
                activity_id: row.get(1)?,
                embedding: row.get(2)?,
                model_name: row.get(3)?,
                dimensions: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_embeddings(
        &self,
    ) -> Result<Vec<Embedding>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, activity_id, embedding, model_name, dimensions, created_at FROM embeddings",
        )?;

        let embeddings = stmt
            .query_map([], |row| {
                Ok(Embedding {
                    id: row.get(0)?,
                    activity_id: row.get(1)?,
                    embedding: row.get(2)?,
                    model_name: row.get(3)?,
                    dimensions: row.get(4)?,
                    created_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(embeddings)
    }

    pub fn insert_pending_embedding(
        &self,
        pending: &PendingEmbedding,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT INTO pending_embeddings (id, activity_id, content, priority, created_at, retry_count)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
            params![
                pending.id,
                pending.activity_id,
                pending.content,
                pending.priority,
                pending.created_at,
                pending.retry_count,
            ],
        )?;
        Ok(())
    }

    pub fn get_pending_embeddings(
        &self,
        limit: usize,
    ) -> Result<Vec<PendingEmbedding>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, activity_id, content, priority, created_at, retry_count 
             FROM pending_embeddings ORDER BY priority DESC, created_at ASC LIMIT ?1",
        )?;

        let pending = stmt
            .query_map(params![limit], |row| {
                Ok(PendingEmbedding {
                    id: row.get(0)?,
                    activity_id: row.get(1)?,
                    content: row.get(2)?,
                    priority: row.get(3)?,
                    created_at: row.get(4)?,
                    retry_count: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(pending)
    }

    pub fn delete_pending_embedding(
        &self,
        id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM pending_embeddings WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn insert_knowledge_node(
        &self,
        node: &KnowledgeNode,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT OR REPLACE INTO knowledge_nodes (id, node_type, identifier, label, properties, created_at, updated_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                node.id,
                node.node_type,
                node.identifier,
                node.label,
                node.properties,
                node.created_at,
                node.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_knowledge_nodes(
        &self,
        node_type: Option<&str>,
    ) -> Result<Vec<KnowledgeNode>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();

        let query = if let Some(nt) = node_type {
            format!(
                "SELECT id, node_type, identifier, label, properties, created_at, updated_at 
                 FROM knowledge_nodes WHERE node_type = '{}'",
                nt
            )
        } else {
            "SELECT id, node_type, identifier, label, properties, created_at, updated_at FROM knowledge_nodes".to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let nodes: Vec<KnowledgeNode> = stmt
            .query_map([], |row| {
                Ok(KnowledgeNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    identifier: row.get(2)?,
                    label: row.get(3)?,
                    properties: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(nodes)
    }

    pub fn insert_knowledge_edge(
        &self,
        edge: &KnowledgeEdge,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute(
            r#"INSERT OR REPLACE INTO knowledge_edges (id, source_id, target_id, edge_type, weight, properties, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                edge.id,
                edge.source_id,
                edge.target_id,
                edge.edge_type,
                edge.weight,
                edge.properties,
                edge.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_knowledge_edges(
        &self,
        node_id: Option<&str>,
    ) -> Result<Vec<KnowledgeEdge>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();

        let query = if let Some(nid) = node_id {
            format!(
                "SELECT id, source_id, target_id, edge_type, weight, properties, created_at 
                 FROM knowledge_edges WHERE source_id = '{}' OR target_id = '{}'",
                nid, nid
            )
        } else {
            "SELECT id, source_id, target_id, edge_type, weight, properties, created_at FROM knowledge_edges".to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let edges: Vec<KnowledgeEdge> = stmt
            .query_map([], |row| {
                Ok(KnowledgeEdge {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    edge_type: row.get(3)?,
                    weight: row.get(4)?,
                    properties: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(edges)
    }

    pub fn delete_activity(
        &self,
        id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM activities WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_all_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        conn.execute_batch(
            r#"
            DELETE FROM activities;
            DELETE FROM sessions;
            DELETE FROM embeddings;
            DELETE FROM knowledge_nodes;
            DELETE FROM knowledge_edges;
            DELETE FROM screenshots;
            DELETE FROM pending_embeddings;
        "#,
        )?;
        Ok(())
    }

    pub fn get_activity_count(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM activities", [], |row| row.get(0))?;
        Ok(count)
    }

    pub fn get_screenshot_count(&self) -> Result<i64, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.conn.lock();
        let count: i64 =
            conn.query_row("SELECT COUNT(*) FROM screenshots", [], |row| row.get(0))?;
        Ok(count)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Activity {
    pub id: String,
    pub activity_type: String,
    pub app_name: Option<String>,
    pub app_bundle_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub file_path: Option<String>,
    pub url: Option<String>,
    pub timestamp: i64,
    pub session_id: Option<String>,
    pub metadata: Option<String>,
    pub created_at: i64,
}

impl Activity {
    pub fn new(activity_type: &str) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            activity_type: activity_type.to_string(),
            app_name: None,
            app_bundle_id: None,
            title: None,
            content: None,
            file_path: None,
            url: None,
            timestamp: now,
            session_id: None,
            metadata: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub session_type: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub app_sequence: Option<String>,
    pub metadata: Option<String>,
    pub created_at: i64,
}

impl Session {
    pub fn new(session_type: &str) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            session_type: session_type.to_string(),
            start_time: now,
            end_time: None,
            app_sequence: None,
            metadata: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Embedding {
    pub id: String,
    pub activity_id: String,
    pub embedding: Vec<u8>,
    pub model_name: String,
    pub dimensions: i32,
    pub created_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingEmbedding {
    pub id: String,
    pub activity_id: String,
    pub content: String,
    pub priority: i32,
    pub created_at: i64,
    pub retry_count: i32,
}

impl PendingEmbedding {
    pub fn new(activity_id: &str, content: &str) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            activity_id: activity_id.to_string(),
            content: content.to_string(),
            priority: 0,
            created_at: now,
            retry_count: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeNode {
    pub id: String,
    pub node_type: String,
    pub identifier: String,
    pub label: Option<String>,
    pub properties: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl KnowledgeNode {
    pub fn new(node_type: &str, identifier: &str) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            node_type: node_type.to_string(),
            identifier: identifier.to_string(),
            label: None,
            properties: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEdge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f64,
    pub properties: Option<String>,
    pub created_at: i64,
}

impl KnowledgeEdge {
    pub fn new(source_id: &str, target_id: &str, edge_type: &str) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            edge_type: edge_type.to_string(),
            weight: 1.0,
            properties: None,
            created_at: now,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Screenshot {
    pub id: String,
    pub activity_id: Option<String>,
    pub image_path: String,
    pub text_content: Option<String>,
    pub timestamp: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: i64,
}
