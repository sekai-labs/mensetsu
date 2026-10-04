use axum::{
    body::Body,
    http::{Request, Response},
    middleware::Next,
};
use tracing::info_span;
use uuid::Uuid;

pub async fn correlation_id_middleware(
    req: Request<Body>,
    next: Next,
) -> Response<Body> {
    let correlation_id = req
        .headers()
        .get("x-correlation-id")
        .and_then(|h| h.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let span = info_span!("request", correlation_id = %correlation_id);
    let _guard = span.enter();

    let mut res = next.run(req).await;
    if let Ok(val) = correlation_id.parse() {
        res.headers_mut().insert("x-correlation-id", val);
    }
    res
}
