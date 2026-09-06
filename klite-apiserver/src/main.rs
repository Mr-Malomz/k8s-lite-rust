use std::sync::Arc;

use axum::{Router, routing::get};
use klite_core::Store;

use crate::handlers::{apply, get_pod, list_pods};

mod handlers;
mod store;

#[tokio::main]
async fn main() {
    let store: Arc<dyn Store> = Arc::new(store::InMemoryStore::new());

    let app = Router::new()
        .route("/pods/:name", get(get_pod).put(apply))
        .route("/pods", get(list_pods))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
