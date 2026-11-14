use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use utoipa::ToSchema;

use crate::{
    interfaces::http::{
        extractors::authenticated_user::AuthenticatedUser,
        token_usage::{load_scope_totals, TokenUsageTotalsDto},
        workspaces::AppState,
    },
    shared::error::AppError,
};

pub const HISTOGRAM_BUCKETS: &[f64] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[derive(Debug, Clone, Default)]
struct HistogramData {
    buckets: Vec<u64>,
    sum: f64,
    count: u64,
}

#[derive(Debug)]
struct RequestLogEntry {
    timestamp: Instant,
    is_success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ServiceMetricsDto {
    /// Percentage of successful HTTP requests (status < 500) over the last 5 minutes (0.0 to 100.0)
    pub success_rate_percent: f64,
    /// Average HTTP requests per second over the last 60 seconds
    pub requests_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MetricsSummaryResponseDto {
    /// Token usage totals for the authenticated user
    pub token_totals: TokenUsageTotalsDto,
    /// Platform-wide token usage totals (admin only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_token_totals: Option<TokenUsageTotalsDto>,
    /// Service health and traffic metrics (admin only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_metrics: Option<ServiceMetricsDto>,
}

fn escape_prometheus_label_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub struct MetricsCollector {
    // Key: (method, path, status) -> count
    requests_total: RwLock<HashMap<(String, String, u16), u64>>,
    // Key: (method, path) -> HistogramData
    latencies: RwLock<HashMap<(String, String), HistogramData>>,
    // Sliding window of requests for service metrics calculations
    recent_requests: Mutex<VecDeque<RequestLogEntry>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            requests_total: RwLock::new(HashMap::new()),
            latencies: RwLock::new(HashMap::new()),
            recent_requests: Mutex::new(VecDeque::new()),
        }
    }

    pub fn record(&self, method: &str, path: &str, status: u16, duration_seconds: f64) {
        let is_success = status < 500;
        let now = Instant::now();

        // 1. Update request counter with bounded cardinality (method, template path, status)
        {
            let mut counters = self.requests_total.write();
            let key = (method.to_string(), path.to_string(), status);
            *counters.entry(key).or_insert(0) += 1;
        }

        // 2. Update latency histogram
        {
            let mut latencies = self.latencies.write();
            let key = (method.to_string(), path.to_string());
            let hist = latencies.entry(key).or_insert_with(|| HistogramData {
                buckets: vec![0; HISTOGRAM_BUCKETS.len()],
                sum: 0.0,
                count: 0,
            });
            hist.count += 1;
            hist.sum += duration_seconds;
            for (i, &le) in HISTOGRAM_BUCKETS.iter().enumerate() {
                if duration_seconds <= le {
                    hist.buckets[i] += 1;
                }
            }
        }

        // 3. Update sliding window for service metrics
        {
            let mut queue = self.recent_requests.lock();
            let five_mins = Duration::from_secs(300);
            while let Some(front) = queue.front() {
                if now.duration_since(front.timestamp) > five_mins {
                    queue.pop_front();
                } else {
                    break;
                }
            }
            queue.push_back(RequestLogEntry {
                timestamp: now,
                is_success,
            });
        }
    }

    pub fn get_admin_metrics(&self) -> (f64, f64) {
        let now = Instant::now();
        let five_mins = Duration::from_secs(300);
        let sixty_secs = Duration::from_secs(60);

        let mut queue = self.recent_requests.lock();
        while let Some(front) = queue.front() {
            if now.duration_since(front.timestamp) > five_mins {
                queue.pop_front();
            } else {
                break;
            }
        }

        let total_5m = queue.len();
        let success_5m = queue.iter().filter(|e| e.is_success).count();
        let success_rate_percent = if total_5m == 0 {
            100.0
        } else {
            (success_5m as f64 / total_5m as f64) * 100.0
        };

        let count_60s = queue
            .iter()
            .filter(|e| now.duration_since(e.timestamp) <= sixty_secs)
            .count();
        let requests_per_second = count_60s as f64 / 60.0;

        (success_rate_percent, requests_per_second)
    }

    pub fn render_prometheus(&self) -> String {
        let mut buffer = String::new();

        // 1. Counter: http_requests_total
        buffer.push_str("# HELP http_requests_total Total number of HTTP requests processed\n");
        buffer.push_str("# TYPE http_requests_total counter\n");
        {
            let counters = self.requests_total.read();
            let mut items: Vec<_> = counters.iter().collect();
            items.sort_by(|a, b| a.0.cmp(b.0));
            for ((method, path, status), count) in items {
                let method = escape_prometheus_label_value(method);
                let path = escape_prometheus_label_value(path);
                buffer.push_str(&format!(
                    "http_requests_total{{method=\"{}\",path=\"{}\",status=\"{}\"}} {}\n",
                    method, path, status, count
                ));
            }
        }

        // 2. Histogram: http_request_duration_seconds
        buffer.push_str("# HELP http_request_duration_seconds HTTP request latency in seconds\n");
        buffer.push_str("# TYPE http_request_duration_seconds histogram\n");
        {
            let latencies = self.latencies.read();
            let mut items: Vec<_> = latencies.iter().collect();
            items.sort_by(|a, b| a.0.cmp(b.0));
            for ((method, path), hist) in items {
                let method = escape_prometheus_label_value(method);
                let path = escape_prometheus_label_value(path);
                for (i, &le) in HISTOGRAM_BUCKETS.iter().enumerate() {
                    let bucket_count = hist.buckets[i];
                    buffer.push_str(&format!(
                        "http_request_duration_seconds_bucket{{method=\"{}\",path=\"{}\",le=\"{}\"}} {}\n",
                        method, path, le, bucket_count
                    ));
                }
                buffer.push_str(&format!(
                    "http_request_duration_seconds_bucket{{method=\"{}\",path=\"{}\",le=\"+Inf\"}} {}\n",
                    method, path, hist.count
                ));
                buffer.push_str(&format!(
                    "http_request_duration_seconds_sum{{method=\"{}\",path=\"{}\"}} {:.6}\n",
                    method, path, hist.sum
                ));
                buffer.push_str(&format!(
                    "http_request_duration_seconds_count{{method=\"{}\",path=\"{}\"}} {}\n",
                    method, path, hist.count
                ));
            }
        }

        // 3. Service admin metrics gauges
        let (success_rate, rps) = self.get_admin_metrics();
        buffer.push_str("# HELP mensetsu_service_success_rate_percent Service success rate percentage over the last 5 minutes\n");
        buffer.push_str("# TYPE mensetsu_service_success_rate_percent gauge\n");
        buffer.push_str(&format!(
            "mensetsu_service_success_rate_percent {:.2}\n",
            success_rate
        ));

        buffer.push_str("# HELP mensetsu_service_requests_per_second Average requests per second over the last 60 seconds\n");
        buffer.push_str("# TYPE mensetsu_service_requests_per_second gauge\n");
        buffer.push_str(&format!(
            "mensetsu_service_requests_per_second {:.4}\n",
            rps
        ));

        buffer
    }
}

pub fn normalize_path(path: &str) -> String {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return "/".to_string();
    }
    let segments: Vec<&str> = trimmed.split('/').collect();
    let normalized: Vec<&str> = segments
        .into_iter()
        .map(|seg| {
            if seg.is_empty() {
                return seg;
            }
            if seg.starts_with("usr_")
                || seg.starts_with("ws_")
                || seg.starts_with("run_")
                || seg.starts_with("exec_")
                || seg.starts_with("task_")
                || seg.starts_with("event_")
                || seg.starts_with("upload_")
                || seg.starts_with("job_")
                || seg.starts_with("app_")
            {
                return "{id}";
            }
            if uuid::Uuid::parse_str(seg).is_ok() {
                return "{id}";
            }
            if seg.chars().all(|c| c.is_ascii_digit()) {
                return "{id}";
            }
            if seg.len() >= 20 && seg.chars().all(|c| c.is_ascii_hexdigit()) {
                return "{id}";
            }
            seg
        })
        .collect();
    format!("/{}", normalized.join("/"))
}

pub async fn metrics_middleware(
    State(metrics): State<Arc<MetricsCollector>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().to_string();
    let uri_path = req.uri().path().to_string();
    let path = req
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(|mp| mp.as_str().to_string())
        .unwrap_or_else(|| normalize_path(&uri_path));

    let start = Instant::now();
    let res = next.run(req).await;
    let duration_sec = start.elapsed().as_secs_f64();
    let status = res.status().as_u16();

    metrics.record(&method, &path, status, duration_sec);
    res
}

pub fn metrics_routes(state: AppState) -> Router {
    Router::new()
        .route("/metrics", get(prometheus_metrics))
        .route("/api/v1/metrics/summary", get(metrics_summary))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus exposition metrics", content_type = "text/plain"),
        (status = 401, description = "Unauthorized or Prometheus metrics disabled")
    ),
    security(("bearerAuth" = []))
)]
pub async fn prometheus_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let configured_token = match &state.config.prometheus_metrics_token {
        Some(token) if !token.trim().is_empty() => token.trim(),
        _ => {
            return Err(AppError::Unauthorized(
                "Prometheus metrics endpoint is disabled or unconfigured".into(),
            ))
        }
    };

    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());
    let provided_token = match auth_header {
        Some(val) if val.starts_with("Bearer ") => val.strip_prefix("Bearer ").unwrap().trim(),
        _ => {
            return Err(AppError::Unauthorized(
                "Missing or invalid Authorization header".into(),
            ))
        }
    };

    let expected = configured_token.as_bytes();
    let provided = provided_token.as_bytes();
    let is_valid: bool = (expected.len() == provided.len()) && bool::from(expected.ct_eq(provided));
    if !is_valid {
        return Err(AppError::Unauthorized(
            "Invalid Prometheus metrics token".into(),
        ));
    }

    let output = state.metrics.render_prometheus();
    Ok((
        [(
            header::CONTENT_TYPE,
            "text/plain; version=0.0.4; charset=utf-8",
        )],
        output,
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/metrics/summary",
    responses(
        (status = 200, description = "Aggregated token and service metrics summary", body = MetricsSummaryResponseDto),
        (status = 401, description = "Unauthenticated")
    ),
    security(("bearerAuth" = []))
)]
pub async fn metrics_summary(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<Json<MetricsSummaryResponseDto>, AppError> {
    let db = &state.db.client;
    let user_id = &auth_user.user_id.0;

    let mut role_result = db
        .query("SELECT role FROM type::record('user', $user_id);")
        .bind(("user_id", user_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    let role_row: Option<serde_json::Value> = role_result.take(0).unwrap_or_default();
    let is_admin = role_row
        .as_ref()
        .and_then(|r| r.get("role"))
        .and_then(|v| v.as_str())
        == Some("admin");

    let user_totals = load_scope_totals(&state.db, Some(user_id)).await?;

    if is_admin {
        let platform_totals = load_scope_totals(&state.db, None).await?;
        let (success_rate_percent, requests_per_second) = state.metrics.get_admin_metrics();
        Ok(Json(MetricsSummaryResponseDto {
            token_totals: user_totals,
            platform_token_totals: Some(platform_totals),
            service_metrics: Some(ServiceMetricsDto {
                success_rate_percent,
                requests_per_second,
            }),
        }))
    } else {
        Ok(Json(MetricsSummaryResponseDto {
            token_totals: user_totals,
            platform_token_totals: None,
            service_metrics: None,
        }))
    }
}
#[cfg(test)]
mod prometheus_label_tests {
    use super::MetricsCollector;

    #[test]
    fn escapes_untrusted_labels_in_counter_and_histogram_output() {
        let metrics = MetricsCollector::new();
        metrics.record("GET\"\\\n", "/bad\"\\\n", 200, 0.01);

        let output = metrics.render_prometheus();
        assert!(output.contains(r#"method="GET\"\\\n",path="/bad\"\\\n",status="200""#));
        assert!(output.contains(
            r#"http_request_duration_seconds_count{method="GET\"\\\n",path="/bad\"\\\n"} 1"#
        ));
    }
}
