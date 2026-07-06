use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum Target {
    Llm,
    Process(i32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum TimeoutPolicy {
    Dynamic {
        heartbeat_timeout_ms: u64,
        max_silent_wait_ms: u64,
    },
    Infinite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload")]
pub enum RasCoreEvent {
    HttpChunkReceived {
        chunk: String,
    },
    ToolCallRequested {
        call_id: String,
        name: String,
        args: serde_json::Value,
    },
    ProcessSpawned {
        pgid: i32,
        pid: i32,
    },
    ProcessStdout {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    ProcessStderr {
        pgid: i32,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    ProcessExited {
        pgid: i32,
        exit_code: Option<i32>,
    },
    FileChanged {
        path: PathBuf,
        change_type: String,
    },
    StreamTimeout {
        target: String,
        duration_ms: u64,
    },
    HumanInputReceived {
        text: String,
    },
    TaskCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "method", content = "params")]
pub enum RasRpcCommand {
    FileRead {
        path: PathBuf,
    },
    FileWrite {
        path: PathBuf,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
    FileEditPatch {
        path: PathBuf,
        diff: String,
    },
    SpawnBashProcess {
        command: String,
    },
    CreateNode {
        parent_id: String,
        node_type: String,
    },
    SetNodeText {
        node_id: String,
        text: String,
    },
    MergeNodes {
        node_ids: Vec<String>,
        summary_text: String,
    },
    DeleteNode {
        node_id: String,
    },
    TakeSnapshot {
        node_id: String,
        target_paths: Vec<PathBuf>,
    },
    CheckoutSnapshot {
        node_id: String,
    },
    OpenHttpStream {
        url: String,
        headers: HashMap<String, String>,
        body: String,
    },
    SetStreamTimeoutPolicy {
        target: Target,
        policy: TimeoutPolicy,
    },
    WriteStdout {
        text: String,
    },
    CompleteTask,
    GetDag,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RasRpcRequest {
    pub id: Option<String>,
    #[serde(flatten)]
    pub command: RasRpcCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RasRpcResponse {
    pub id: Option<String>,
    pub result: Result<serde_json::Value, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagNode {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub node_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dag {
    pub nodes: HashMap<String, DagNode>,
    pub current_node_id: Option<String>,
    pub next_node_index: usize,
}

impl Dag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&mut self, parent_id: &str, node_type: &str) -> Result<String, String> {
        let mut parent_ids = Vec::new();
        if !parent_id.is_empty() {
            if !self.nodes.contains_key(parent_id) {
                return Err(format!("Parent node '{parent_id}' not found"));
            }
            parent_ids.push(parent_id.to_string());
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let node = DagNode {
            id: new_id.clone(),
            parent_ids,
            node_type: node_type.to_string(),
            text: String::new(),
        };

        self.nodes.insert(new_id.clone(), node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    pub fn set_node_text(&mut self, node_id: &str, text: &str) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{node_id}' not found"))?;
        node.text = text.to_string();
        Ok(())
    }

    pub fn merge_nodes(&mut self, node_ids: &[String], summary_text: &str) -> Result<String, String> {
        if node_ids.is_empty() {
            return Err("Cannot merge empty list of nodes".to_string());
        }

        let mut collected_parents = HashSet::new();
        let target_set: HashSet<&String> = node_ids.iter().collect();

        for id in node_ids {
            let node = self.nodes.get(id).ok_or_else(|| format!("Node '{id}' not found"))?;
            for parent in &node.parent_ids {
                if !target_set.contains(parent) {
                    collected_parents.insert(parent.clone());
                }
            }
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let merge_node = DagNode {
            id: new_id.clone(),
            parent_ids: collected_parents.into_iter().collect(),
            node_type: "merge".to_string(),
            text: summary_text.to_string(),
        };

        self.redirect_children(node_ids, &new_id);

        for id in node_ids {
            self.nodes.remove(id);
        }

        self.nodes.insert(new_id.clone(), merge_node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    fn redirect_children(&mut self, merged_ids: &[String], new_parent_id: &str) {
        let target_set: HashSet<&String> = merged_ids.iter().collect();
        for node in self.nodes.values_mut() {
            if target_set.contains(&node.id) {
                continue;
            }
            for parent in &mut node.parent_ids {
                if target_set.contains(parent) {
                    *parent = new_parent_id.to_string();
                }
            }
            node.parent_ids.sort();
            node.parent_ids.dedup();
        }
    }

    pub fn delete_node(&mut self, node_id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found"));
        }

        self.nodes.remove(node_id);

        for node in self.nodes.values_mut() {
            node.parent_ids.retain(|x| x != node_id);
        }

        if self.current_node_id.as_deref() == Some(node_id) {
            self.current_node_id = None;
        }

        Ok(())
    }
}
