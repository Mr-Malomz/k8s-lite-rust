use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use klite_core::{Pod, PodSpec, Store};

pub struct AppError {
    error: anyhow::Error,
    status: StatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (self.status, self.error.to_string()).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self {
            error: err.into(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// ---- Handlers (REST handlers backed by the store) ----

pub async fn get_pod(
    State(store): State<Arc<dyn Store>>,
    Path(name): Path<String>,
) -> Result<Json<Pod>, AppError> {
    let pod = store.get_pod(&name).await?;
    match pod {
        Some(pod) => Ok(Json(pod)),
        None => Err(AppError {
            error: anyhow::anyhow!("Pod not found"),
            status: StatusCode::NOT_FOUND,
        }),
    }
}

pub async fn apply(
    State(store): State<Arc<dyn Store>>,
    Path(name): Path<String>,
    Json(spec): Json<PodSpec>,
) -> Result<Json<Pod>, AppError> {
    // Optional: validate name in path matches spec.name and return 400 if not,
    if name != spec.name {
        return Err(AppError {
            error: anyhow::anyhow!("name in path must match spec.name"),
            status: StatusCode::BAD_REQUEST,
        });
    }
    let existing = store.get_pod(&spec.name).await?;

    let pod = match existing {
        Some(existing_pod) => Pod {
            uid: existing_pod.uid,
            spec,
            status: existing_pod.status,
            created_at: existing_pod.created_at,
        },
        None => Pod {
            uid: uuid::Uuid::new_v4().to_string(),
            spec,
            status: klite_core::PodStatus {
                phase: klite_core::PodPhase::Pending,
                node_name: None,
                container_id: None,
                restart_count: 0,
                last_transition: std::time::SystemTime::now(),
            },
            created_at: std::time::SystemTime::now(),
        },
    };

    store.put_pod(pod.clone()).await?;
    Ok(Json(pod))
}

pub async fn list_pods(State(store): State<Arc<dyn Store>>) -> Result<Json<Vec<Pod>>, AppError> {
    let pods = store.list_pods().await?;
    Ok(Json(pods))
}
