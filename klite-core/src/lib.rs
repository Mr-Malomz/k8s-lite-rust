use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// What the user submits ("desired state")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSpec {
    pub name: String,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub env: Vec<(String, String)>,
    pub restart_policy: RestartPolicy,
    // scheduler for bin-packing decisions, not enforced by the agent yet
    pub resources: ResourceRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    pub cpu_millis: u32,
    pub memory_mb: u32,
}

/// What the system observes ("actual state")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodStatus {
    pub phase: PodPhase,
    pub node_name: Option<String>,    // set once scheduled
    pub container_id: Option<String>, // set once the agent starts it
    pub restart_count: u32,
    pub last_transition: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PodPhase {
    Pending,   // accepted by API server, not yet scheduled
    Scheduled, // assigned to a node, not yet running
    Running,
    Failed,
    Succeeded,
}

/// The full object as stored — spec (desired) + status (actual) is the
/// exact pattern real k8s uses, and it's the whole trick: reconciliation
/// is just "make status converge toward spec"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub uid: String,
    pub spec: PodSpec,
    pub status: PodStatus,
    pub created_at: SystemTime,
}

/// A worker node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub address: String, // agent's callback/base URL
    pub capacity: ResourceRequest,
    pub last_heartbeat: SystemTime,
}

// ---- The store trait — swap implementations later (HashMap -> sled -> raft) ----

#[async_trait::async_trait]
pub trait Store: Send + Sync {
    async fn put_pod(&self, pod: Pod) -> anyhow::Result<()>;
    async fn get_pod(&self, uid: &str) -> anyhow::Result<Option<Pod>>;
    async fn list_pods(&self) -> anyhow::Result<Vec<Pod>>;
    async fn put_node(&self, node: Node) -> anyhow::Result<()>;
    async fn list_nodes(&self) -> anyhow::Result<Vec<Node>>;
}
