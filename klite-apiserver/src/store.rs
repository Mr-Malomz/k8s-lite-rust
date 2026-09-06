use std::{collections::HashMap, sync::RwLock};

pub struct InMemoryStore {
    pods: RwLock<HashMap<String, klite_core::Pod>>,
    nodes: RwLock<HashMap<String, klite_core::Node>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            pods: RwLock::new(HashMap::new()),
            nodes: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl klite_core::Store for InMemoryStore {
     async fn put_pod(&self, pod: klite_core::Pod) -> anyhow::Result<()> {
        let mut pods = self.pods.write().unwrap();
        pods.insert(pod.spec.name.clone(), pod);
        Ok(())
    }

    async fn get_pod(&self, name: &str) -> anyhow::Result<Option<klite_core::Pod>> {
        let pods = self.pods.read().unwrap();
        Ok(pods.get(name).cloned())
    }

    async fn list_pods(&self) -> anyhow::Result<Vec<klite_core::Pod>> {
        let pods = self.pods.read().unwrap();
        Ok(pods.values().cloned().collect())
    }

    async fn put_node(&self, node: klite_core::Node) -> anyhow::Result<()> {
        let mut nodes = self.nodes.write().unwrap();
        nodes.insert(node.name.clone(), node);
        Ok(())
    }

    async fn list_nodes(&self) -> anyhow::Result<Vec<klite_core::Node>> {
        let nodes = self.nodes.read().unwrap();
        Ok(nodes.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use klite_core::Store;

use super::*;

    #[tokio::test]
    async fn test_put_pod() -> anyhow::Result<()> {
        let store = InMemoryStore::new();
        let pod = klite_core::Pod {
            uid: "1".to_string(),
            spec: klite_core::PodSpec {
                name: "test-pod".to_string(),
                image: "test-image".to_string(),
                command: None,
                env: vec![],
                restart_policy: klite_core::RestartPolicy::Always,
                resources: klite_core::ResourceRequest {
                    cpu_millis: 1000,
                    memory_mb: 1000,
                },
            },
            status: klite_core::PodStatus {
                phase: klite_core::PodPhase::Pending,
                node_name: None,
                container_id: None,
                restart_count: 0,
                last_transition: std::time::SystemTime::now(),
            },
            created_at: std::time::SystemTime::now(),
        };

        let expected = pod.clone();
        store.put_pod(pod).await?;
        assert_eq!(store.get_pod("test-pod").await?, Some(expected));
        Ok(())
    }

    #[tokio::test]
    async fn  test_get_pod_not_found() -> anyhow::Result<()> {
        let store = InMemoryStore::new();
        let result = store.get_pod("non-existent-pod").await?;
        assert!(result.is_none());
        Ok(())
    }
}
