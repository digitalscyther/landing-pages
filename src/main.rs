use axum::{extract::{Path, State}, http::{header, StatusCode}, response::IntoResponse, routing::{get, post}, Router, Json};
use redis::{AsyncCommands, RedisResult};
use std::{env, sync::Arc};
use http::HeaderMap;
use tracing::{info, error, Level};
use serde::Serialize;

pub struct AppState {
    pub redis_client: redis::Client,
}

#[tokio::main]
async fn main() {
     tracing_subscriber::fmt()
        .with_max_level(Level::ERROR)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())// Log only errors and above
        .init();

    let redis_url = env::var("REDIS_URL").unwrap_or("redis://127.0.0.1/".to_string());
    let redis_client = redis::Client::open(redis_url).unwrap();
    let app_state = AppState { redis_client };

    let router = Router::new()
        .route("/p/:path", get(get_page))
        .route("/create_page/:path", post(create_page))
        .with_state(Arc::new(app_state));

    let host = env::var("HOST").unwrap_or("127.0.0.1".to_string());
    let port = env::var("PORT").unwrap_or("3000".to_string());
    let bind_address = format!("{}:{}", host, port);
    info!("Listening on {}", bind_address);
    let listener = tokio::net::TcpListener::bind(bind_address)
        .await
        .unwrap();

    axum::serve(listener, router.into_make_service()).await.unwrap();
}

#[derive(Serialize)]
struct CreatePageResponse {
    success: bool
}

async fn create_page(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<CreatePageResponse>, StatusCode> {
    let auth_token = env::var("AUTH_TOKEN").map_err(|_| StatusCode::UNAUTHORIZED)?;
    let req_auth_token = headers.get("Authorization").and_then(|v| v.to_str().ok()).ok_or(StatusCode::UNAUTHORIZED)?;
    if auth_token != req_auth_token {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut redis_conn = redis_connection(&state.redis_client).await.map_err(|e| log_err("redis_connection", e))?;

    let page_key =  format!("page:{}", path);
    redis_conn.hset(&page_key, "html", body.to_vec()).await.map_err(|e| log_err("0", e))?;
    redis_conn.expire(&page_key, 60 * 60 * 24 * 30).await.map_err(|e| log_err("1", e))?;

    Ok(Json(CreatePageResponse { success: true }))
}

async fn redis_connection(redis_client: &redis::Client) -> RedisResult<redis::aio::MultiplexedConnection> {
    redis_client.get_multiplexed_async_connection().await
}

fn log_err<T: std::fmt::Display>(tag: &str, err: T) -> StatusCode {
    error!("{} - error - {}", tag, err);
    StatusCode::INTERNAL_SERVER_ERROR
}

async fn get_page(
    Path(path): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut redis_conn = match redis_connection(&state.redis_client).await {
        Ok(conn) => conn,
        Err(err) => {
            return (log_err("redis_connection", err),
                    [(header::CONTENT_TYPE, "text/html")],
                    "Internal error".to_string());
        }
    };

    let page_key = format!("page:{}", path);

    let page_content = match redis_conn.hget(&page_key, "html").await {
        Ok(Some(content)) => content,
        _ => return (StatusCode::NOT_FOUND,
                     [(header::CONTENT_TYPE, "text/html")],
                     "".to_string()),
    };

    (StatusCode::OK, [(header::CONTENT_TYPE, "text/html")], page_content)
}