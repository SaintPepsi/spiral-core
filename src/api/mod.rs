use crate::{
    agents::AgentOrchestrator,
    auth::{auth_middleware, create_auth_state},
    config::{ApiConfig, Config},
    models::{AgentType, Priority, Task, TaskStatus},
    monitoring::SystemMonitor,
    rate_limit::rate_limit_middleware, // RateLimitConfig},
    validation::TaskContentValidator,
    Result,
    SpiralError,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{error, info, warn};

// 🏗️ ARCHITECTURE DECISION: Service metadata constants
// Why: Centralized version and service info for consistency
// Alternative: Build-time env vars (rejected: harder to update)
// Trade-off: Manual version updates vs automated versioning
const SERVICE_NAME: &str = "spiral-core";
const SERVICE_VERSION: &str = "0.1.0";

// 🏗️ ARCHITECTURE DECISION: API route constants for maintainability
// Why: Single source of truth for route definitions
// Alternative: Inline strings (rejected: error-prone, hard to refactor)
// Audit: All routes must be defined here
const ROUTE_HEALTH: &str = "/health";
const ROUTE_TASKS: &str = "/tasks";
const ROUTE_TASK_BY_ID: &str = "/tasks/{task_id}";
const ROUTE_TASK_ANALYZE: &str = "/tasks/{task_id}/analyze";
const ROUTE_AGENTS: &str = "/agents";
const ROUTE_AGENT_BY_TYPE: &str = "/agents/{agent_type}";
const ROUTE_SYSTEM_STATUS: &str = "/system/status";
const ROUTE_SYSTEM_METRICS: &str = "/system/metrics";
const ROUTE_SYSTEM_METRICS_HISTORY: &str = "/system/metrics/history";
const ROUTE_SYSTEM_HEALTH: &str = "/system/health";
const ROUTE_CIRCUIT_BREAKERS: &str = "/circuit-breakers";
const ROUTE_WORKSPACES: &str = "/workspaces";

// 🏗️ ARCHITECTURE DECISION: Error message constants
// Why: Consistent error messages across API responses
// Alternative: Inline strings (rejected: inconsistent user experience)
const ERROR_INTERNAL_SERVER: &str = "Internal server error";
const ERROR_AGENT_NOT_FOUND: &str = "Agent not found";
const ERROR_INVALID_CONTENT: &str = "Invalid task content";
const ERROR_INVALID_CONTEXT_KEY: &str = "Invalid context key";
const ERROR_INVALID_CONTEXT_VALUE: &str = "Invalid context value";

// ⚡ PERFORMANCE DECISION: Workspace status thresholds
// Why: Time-based categorization for workspace activity
// Alternative: Event-based tracking (rejected: more complex)
// Trade-off: Simple but less precise than actual activity tracking
const WORKSPACE_ACTIVE_THRESHOLD_SECS: u64 = 300; // 5 minutes
const WORKSPACE_RECENT_THRESHOLD_SECS: u64 = 3600; // 1 hour
const WORKSPACE_IDLE_THRESHOLD_SECS: u64 = 86400; // 24 hours

// 🏗️ ARCHITECTURE DECISION: Workspace defaults
// Why: Consistent fallback values for missing data
const UNKNOWN_VALUE: &str = "unknown";
const WORKSPACES_DIR: &str = "claude-workspaces";
const SESSION_PREFIX: &str = "session-";

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    orchestrator: Arc<AgentOrchestrator>,
    validator: TaskContentValidator,
    system_monitor: Option<Arc<SystemMonitor>>,
    // rate_limiter: RateLimitConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub agent_type: AgentType,
    pub content: String,
    pub priority: Option<Priority>,
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusResponse {
    pub task_id: String,
    pub agent_type: AgentType,
    pub status: TaskStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatusResponse {
    pub agent_type: AgentType,
    pub is_busy: bool,
    pub current_task_id: Option<String>,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub average_execution_time: f64,
}

/// 🔧 DRY PRINCIPLE: Single conversion logic for AgentStatus -> AgentStatusResponse
/// DECISION: Centralize status mapping to eliminate code duplication
/// Why: Reduces maintenance burden and ensures consistency across endpoints
/// Alternative: Inline mapping (rejected: violates DRY principle)
impl From<crate::agents::AgentStatus> for AgentStatusResponse {
    fn from(status: crate::agents::AgentStatus) -> Self {
        Self {
            agent_type: status.agent_type,
            is_busy: status.is_busy,
            current_task_id: status.current_task_id,
            tasks_completed: status.tasks_completed,
            tasks_failed: status.tasks_failed,
            average_execution_time: status.average_execution_time,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub agents: HashMap<AgentType, AgentStatusResponse>,
    pub queue_length: usize,
    pub system_uptime: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskAnalysisResponse {
    pub complexity: String,
    pub estimated_minutes: u32,
    pub required_skills: Vec<String>,
    pub challenges: Vec<String>,
    pub approach: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceStatusResponse {
    pub workspace_id: String,
    pub session_id: Option<String>,
    pub created_at: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub file_count: usize,
    pub last_modified: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllWorkspacesStatusResponse {
    pub workspaces: Vec<WorkspaceStatusResponse>,
    pub total_count: usize,
    pub total_size_bytes: u64,
    pub total_size_human: String,
}

#[derive(Debug, Deserialize)]
pub struct TaskQueryParams {
    pub limit: Option<usize>,
    pub agent_type: Option<AgentType>,
    pub status: Option<TaskStatus>,
}

impl ApiServer {
    pub fn new(config: Config, orchestrator: Arc<AgentOrchestrator>) -> Result<Self> {
        let validator = TaskContentValidator::new()?;
        // let rate_limiter = RateLimitConfig::new();
        Ok(Self {
            config: config.api,
            orchestrator,
            validator,
            system_monitor: None,
            // rate_limiter,
        })
    }

    /// Set the system monitor for monitoring endpoints
    pub fn with_system_monitor(mut self, monitor: Arc<SystemMonitor>) -> Self {
        self.system_monitor = Some(monitor);
        self
    }

    pub async fn run(&self) -> Result<()> {
        let app = self.build_router();

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", self.config.host, self.config.port))
                .await
                .map_err(|e| SpiralError::Internal(e.into()))?;

        info!(
            "API server listening on {}:{}",
            self.config.host, self.config.port
        );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .map_err(|e| SpiralError::Internal(e.into()))?;

        Ok(())
    }

    /// 🏗️ ARCHITECTURE DECISION: Layered middleware approach
    /// Why: Clear separation of concerns for security and observability
    /// Alternative: Monolithic handler (rejected: poor separation)
    /// Order matters: Rate limit -> Auth -> Trace -> CORS -> Routes
    pub fn build_router(&self) -> Router {
        // 🛡️ SECURITY CHECKPOINT: Auth state initialization
        // Critical: API keys and auth config loaded here
        // Audit: Verify auth_state contains valid configuration
        let auth_state = create_auth_state(self.config.clone());

        // 🛡️ SECURITY DECISION: Restrictive CORS policy
        // Why: Prevent unauthorized cross-origin requests
        // Alternative: Permissive CORS (rejected: security risk)
        // Trade-off: May block legitimate clients if not configured
        let cors_layer = CorsLayer::new()
            .allow_origin(
                self.config
                    .allowed_origins
                    .iter()
                    .filter_map(|origin| origin.parse().ok())
                    .collect::<Vec<_>>(),
            )
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
                axum::http::HeaderName::from_static("x-api-key"),
            ])
            .max_age(std::time::Duration::from_secs(3600)); // 1 hour cache

        Router::new()
            .route(ROUTE_HEALTH, get(health_check))
            .route(ROUTE_TASKS, post(create_task))
            .route(ROUTE_TASK_BY_ID, get(get_task_status))
            .route(ROUTE_TASK_ANALYZE, post(analyze_task))
            .route(ROUTE_AGENTS, get(get_all_agent_statuses))
            .route(ROUTE_AGENT_BY_TYPE, get(get_agent_status))
            .route(ROUTE_SYSTEM_STATUS, get(get_system_status))
            .route(ROUTE_SYSTEM_METRICS, get(get_system_metrics))
            .route(ROUTE_SYSTEM_METRICS_HISTORY, get(get_metrics_history))
            .route(ROUTE_SYSTEM_HEALTH, get(get_system_health))
            .route(ROUTE_CIRCUIT_BREAKERS, get(get_circuit_breaker_status))
            .route(ROUTE_WORKSPACES, get(get_all_workspaces_status))
            .layer(
                ServiceBuilder::new()
                    .layer(middleware::from_fn(rate_limit_middleware)) // SECURITY: Rate limiting
                    .layer(middleware::from_fn_with_state(auth_state, auth_middleware))
                    .layer(TraceLayer::new_for_http())
                    .layer(cors_layer), // SECURITY: Restrictive CORS policy
            )
            .with_state(self.clone())
    }
}

/// 🏗️ ARCHITECTURE DECISION: Static health response
/// Why: Simple health check for load balancers and monitoring
/// Alternative: Include system metrics (rejected: separate /metrics endpoint)
/// Trade-off: Less info but faster response and lower overhead
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": SERVICE_NAME,
        "version": SERVICE_VERSION
    }))
}

/// 📝 CREATE TASK ENDPOINT: Primary user request entry point
/// AUDIT CHECKPOINT: Critical security and validation path
/// Verify: Authentication, rate limiting, content validation, orchestrator submission
async fn create_task(
    State(api_server): State<ApiServer>,
    Json(request): Json<CreateTaskRequest>,
) -> std::result::Result<(StatusCode, Json<CreateTaskResponse>), (StatusCode, Json<ErrorResponse>)>
{
    // 🛡️ SECURITY AUDIT CHECKPOINT: Content validation and sanitization
    // CRITICAL: This is the primary defense against malicious task content
    // Verify: XSS prevention, injection attack mitigation, content length limits
    let sanitized_content = match api_server
        .validator
        .validate_and_sanitize_task_content(&request.content)
    {
        Ok(content) => content,
        Err(_e) => {
            // 🚨 SECURITY INCIDENT: Invalid content detected
            // AUDIT: Check if this indicates an attack attempt or accidental malformed input
            warn!(
                "Task content validation failed for content: {}",
                &request.content[..std::cmp::min(100, request.content.len())]
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ERROR_INVALID_CONTENT.to_string(),
                    details: None, // SECURITY: Don't expose validation details
                }),
            ));
        }
    };

    // 📊 PRIORITY ASSIGNMENT: Default to medium priority for balanced processing
    // AUDIT: Verify priority escalation policies and user privilege alignment
    let priority = request.priority.unwrap_or(Priority::Medium);
    let mut task = Task::new(request.agent_type, sanitized_content, priority);

    // 🔍 CONTEXT VALIDATION AUDIT CHECKPOINT: Secondary security validation
    // CRITICAL: Context can contain sensitive data or injection vectors
    // Verify: Key format validation, value sanitization, size limits
    if let Some(context) = request.context {
        for (key, value) in context {
            // 🔑 KEY VALIDATION: Prevent malicious context keys
            if api_server.validator.validate_context_key(&key).is_err() {
                warn!("Invalid context key detected: {}", key);
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: ERROR_INVALID_CONTEXT_KEY.to_string(),
                        details: None, // SECURITY: Don't expose key validation rules
                    }),
                ));
            }

            // 🛡️ VALUE SANITIZATION: Clean potentially malicious context values
            let sanitized_value = match api_server
                .validator
                .validate_and_sanitize_context_value(&value)
            {
                Ok(sanitized_value) => sanitized_value,
                Err(_) => {
                    warn!(
                        "Invalid context value for key '{}': {}",
                        key,
                        &value[..std::cmp::min(50, value.len())]
                    );
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: ERROR_INVALID_CONTEXT_VALUE.to_string(),
                            details: None, // SECURITY: Don't expose value validation details
                        }),
                    ));
                }
            };

            task = task.with_context(key, sanitized_value);
        }
    }

    // 🎯 ORCHESTRATOR SUBMISSION AUDIT CHECKPOINT: Hand-off to agent system
    // CRITICAL: Last point of API control before agent processing
    // Verify: Task queue health, agent availability, resource limits
    match api_server.orchestrator.submit_task(task).await {
        Ok(task_id) => {
            // ✅ SUCCESSFUL SUBMISSION: Task accepted by orchestrator
            // AUDIT: Verify task ID generation security and uniqueness
            info!("Task {} successfully submitted to orchestrator", task_id);
            Ok((
                StatusCode::CREATED,
                Json(CreateTaskResponse {
                    task_id,
                    status: "submitted".to_string(),
                }),
            ))
        }
        Err(e) => {
            // 🚨 SUBMISSION FAILURE AUDIT CHECKPOINT: System capacity or validation issue
            // CRITICAL: Could indicate system overload, agent unavailability, or attack
            warn!("Failed to submit task to orchestrator: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ERROR_INTERNAL_SERVER.to_string(),
                    details: None, // SECURITY: Never expose internal orchestrator errors
                }),
            ))
        }
    }
}

async fn get_task_status(
    State(api_server): State<ApiServer>,
    Path(task_id): Path<String>,
) -> std::result::Result<Json<TaskStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    match api_server.orchestrator.get_task_status(&task_id).await {
        Some(task) => Ok(Json(TaskStatusResponse {
            task_id: task.id,
            agent_type: task.agent_type,
            status: task.status,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Task not found".to_string(),
                details: Some(format!("Task ID: {task_id}")),
            }),
        )),
    }
}

async fn analyze_task(
    State(api_server): State<ApiServer>,
    Path(task_id): Path<String>,
    Json(request): Json<CreateTaskRequest>,
) -> std::result::Result<Json<TaskAnalysisResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut task = Task::new(request.agent_type, request.content, Priority::Medium);
    task.id = task_id;

    if let Some(context) = request.context {
        for (key, value) in context {
            task = task.with_context(key, value);
        }
    }

    match api_server.orchestrator.analyze_task(&task).await {
        Ok(analysis) => Ok(Json(TaskAnalysisResponse {
            complexity: analysis.complexity,
            estimated_minutes: analysis.estimated_minutes,
            required_skills: analysis.required_skills,
            challenges: analysis.challenges,
            approach: analysis.approach,
        })),
        Err(e) => {
            // SECURITY: Log detailed error server-side only
            warn!("Failed to analyze task: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: ERROR_INTERNAL_SERVER.to_string(),
                    details: None, // SECURITY: Never expose internal errors
                }),
            ))
        }
    }
}

async fn get_agent_status(
    State(api_server): State<ApiServer>,
    Path(agent_type_str): Path<String>,
) -> std::result::Result<Json<AgentStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Try to parse the agent type string into AgentType enum
    let agent_type = match agent_type_str.parse::<AgentType>() {
        Ok(agent_type) => agent_type,
        Err(_) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: ERROR_AGENT_NOT_FOUND.to_string(),
                    details: Some(format!("Agent type: {agent_type_str}")),
                }),
            ));
        }
    };
    match api_server.orchestrator.get_agent_status(&agent_type).await {
        Some(status) => Ok(Json(status.into())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ERROR_AGENT_NOT_FOUND.to_string(),
                details: Some(format!("Agent type: {agent_type:?}")),
            }),
        )),
    }
}

async fn get_all_agent_statuses(
    State(api_server): State<ApiServer>,
) -> Json<HashMap<AgentType, AgentStatusResponse>> {
    let statuses = api_server.orchestrator.get_all_agent_statuses().await;
    let response: HashMap<AgentType, AgentStatusResponse> = statuses
        .into_iter()
        .map(|(agent_type, status)| (agent_type, status.into()))
        .collect();

    Json(response)
}

async fn get_system_status(State(api_server): State<ApiServer>) -> Json<SystemStatusResponse> {
    let agent_statuses = api_server.orchestrator.get_all_agent_statuses().await;
    let queue_length = api_server.orchestrator.get_queue_length().await;
    let system_uptime = api_server.orchestrator.get_system_uptime().await;

    let agents: HashMap<AgentType, AgentStatusResponse> = agent_statuses
        .into_iter()
        .map(|(agent_type, status)| (agent_type, status.into()))
        .collect();

    Json(SystemStatusResponse {
        agents,
        queue_length,
        system_uptime,
    })
}

async fn get_all_workspaces_status(
    State(api_server): State<ApiServer>,
) -> std::result::Result<Json<AllWorkspacesStatusResponse>, (StatusCode, Json<ErrorResponse>)> {
    match scan_workspaces_directory(&api_server).await {
        Ok(workspaces) => {
            let total_count = workspaces.len();
            let total_size_bytes = workspaces.iter().map(|w| w.size_bytes).sum();

            Ok(Json(AllWorkspacesStatusResponse {
                workspaces,
                total_count,
                total_size_bytes,
                total_size_human: format_bytes_human_readable(total_size_bytes),
            }))
        }
        Err(e) => {
            warn!("Failed to scan workspaces: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to scan workspaces".to_string(),
                    details: None, // SECURITY: Don't expose internal error details
                }),
            ))
        }
    }
}

async fn scan_workspaces_directory(
    _api_server: &ApiServer,
) -> Result<Vec<WorkspaceStatusResponse>> {
    use std::fs;

    // Get the current working directory and construct the workspaces path
    let current_dir = std::env::current_dir().map_err(|e| SpiralError::Agent {
        message: format!("Failed to get current directory: {e}"),
    })?;

    // ⚡ PERFORMANCE DECISION: Workspace directory scanning
    // Why: Simple filesystem-based workspace discovery
    // Alternative: Database tracking (rejected: added complexity)
    // Risk: Unbounded directory traversal for large workspace counts
    // Mitigation: Consider adding depth/count limits
    let workspace_base_dir = current_dir.join(WORKSPACES_DIR);

    if !workspace_base_dir.exists() {
        return Ok(Vec::new());
    }

    let mut workspaces = Vec::new();

    let entries = fs::read_dir(&workspace_base_dir).map_err(|e| SpiralError::Agent {
        message: format!("Failed to read workspaces directory: {e}"),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| SpiralError::Agent {
            message: format!("Failed to read workspace entry: {e}"),
        })?;

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let workspace_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(UNKNOWN_VALUE)
            .to_string();

        // Extract session ID if this is a session workspace
        let session_id = if workspace_name.starts_with(SESSION_PREFIX) {
            Some(
                workspace_name
                    .strip_prefix(SESSION_PREFIX)
                    .unwrap_or(UNKNOWN_VALUE)
                    .to_string(),
            )
        } else {
            None
        };

        // Get directory metadata
        let metadata = entry.metadata().map_err(|e| SpiralError::Agent {
            message: format!("Failed to get workspace metadata: {e}"),
        })?;

        let created_at = metadata
            .created()
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Utc> = time.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_else(|_| UNKNOWN_VALUE.to_string());

        let last_modified = metadata
            .modified()
            .map(|time| {
                let datetime: chrono::DateTime<chrono::Utc> = time.into();
                datetime.to_rfc3339()
            })
            .unwrap_or_else(|_| UNKNOWN_VALUE.to_string());

        // Calculate directory size and file count
        let (size_bytes, file_count) = calculate_directory_size(&path)?;

        // Determine status based on age and activity
        let status = determine_workspace_status(&metadata, &path)?;

        workspaces.push(WorkspaceStatusResponse {
            workspace_id: workspace_name,
            session_id,
            created_at,
            size_bytes,
            size_human: format_bytes_human_readable(size_bytes),
            file_count,
            last_modified,
            status,
        });
    }

    // Sort by creation time (newest first)
    workspaces.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    Ok(workspaces)
}

fn calculate_directory_size(dir_path: &std::path::Path) -> Result<(u64, usize)> {
    use std::fs;

    let mut total_size = 0u64;
    let mut file_count = 0usize;

    fn visit_dir(
        dir: &std::path::Path,
        total_size: &mut u64,
        file_count: &mut usize,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| SpiralError::Agent {
            message: format!("Failed to read directory: {e}"),
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| SpiralError::Agent {
                message: format!("Failed to read entry: {e}"),
            })?;

            let path = entry.path();
            let metadata = entry.metadata().map_err(|e| SpiralError::Agent {
                message: format!("Failed to get metadata: {e}"),
            })?;

            if metadata.is_file() {
                *total_size += metadata.len();
                *file_count += 1;
            } else if metadata.is_dir() {
                visit_dir(&path, total_size, file_count)?;
            }
        }

        Ok(())
    }

    visit_dir(dir_path, &mut total_size, &mut file_count)?;
    Ok((total_size, file_count))
}

fn determine_workspace_status(
    metadata: &std::fs::Metadata,
    _path: &std::path::Path,
) -> Result<String> {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let created = metadata.created().unwrap_or(now);
    let modified = metadata.modified().unwrap_or(now);

    let age = now.duration_since(created).unwrap_or_default();
    let last_activity = now.duration_since(modified).unwrap_or_default();

    // ⚡ PERFORMANCE DECISION: Simple time-based workspace categorization
    // Why: Quick status determination without complex activity tracking
    // Alternative: Event-based tracking (rejected: requires persistent state)
    // Trade-off: Less accurate but much simpler and stateless
    let status = if last_activity.as_secs() < WORKSPACE_ACTIVE_THRESHOLD_SECS {
        "active"
    } else if last_activity.as_secs() < WORKSPACE_RECENT_THRESHOLD_SECS {
        "recent"
    } else if age.as_secs() < WORKSPACE_IDLE_THRESHOLD_SECS {
        "idle"
    } else {
        "old"
    };

    Ok(status.to_string())
}

fn format_bytes_human_readable(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: u64 = 1024;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD as f64 && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD as f64;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{bytes} {}", UNITS[unit_index])
    } else {
        format!("{size:.1} {}", UNITS[unit_index])
    }
}

/// 📊 SYSTEM METRICS ENDPOINT: Current system performance and health metrics
/// DECISION: Real-time metrics for operational visibility
/// Why: Enables proactive monitoring and troubleshooting
/// Alternative: Log-only metrics (rejected: not accessible for monitoring tools)
async fn get_system_metrics(
    State(server): State<ApiServer>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    if let Some(monitor) = &server.system_monitor {
        let metrics = monitor.get_current_metrics().await;
        match serde_json::to_value(metrics) {
            Ok(value) => Ok(Json(value)),
            Err(e) => {
                error!("Failed to serialize metrics: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// 📈 METRICS HISTORY ENDPOINT: Historical performance data
/// DECISION: Provide metrics history for trend analysis
/// Why: Enables identification of performance patterns and degradation
/// Alternative: Current metrics only (rejected: no trend visibility)
async fn get_metrics_history(
    State(server): State<ApiServer>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    if let Some(monitor) = &server.system_monitor {
        let history = monitor.get_metrics_history().await;
        Ok(Json(serde_json::json!({
            "metrics_count": history.len(),
            "metrics": history
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// 🏥 SYSTEM HEALTH ENDPOINT: Overall health assessment
/// DECISION: Simple health check with detailed status
/// Why: Standard health check endpoint for load balalncers and monitoring
/// Alternative: Binary healthy/unhealthy (rejected: insufficient detail)
async fn get_system_health(
    State(server): State<ApiServer>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    if let Some(monitor) = &server.system_monitor {
        let health_status = monitor.get_health_status().await;

        Ok(Json(serde_json::json!({
            "status": health_status,
            "service": "spiral-core",
            "version": "0.1.0",
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "unknown",
            "service": "spiral-core",
            "version": "0.1.0",
            "error": "monitoring not available"
        })))
    }
}

/// ⚡ CIRCUIT BREAKER STATUS ENDPOINT: Circuit breaker states and metrics
/// DECISION: Dedicated endpoint for circuit breaker monitoring
/// Why: Circuit breaker health is critical for system reliability
/// Alternative: Include in general metrics (rejected: needs specific visibility)
async fn get_circuit_breaker_status(
    State(server): State<ApiServer>,
) -> std::result::Result<Json<serde_json::Value>, StatusCode> {
    if let Some(monitor) = &server.system_monitor {
        let metrics = monitor.get_current_metrics().await;
        Ok(Json(serde_json::json!({
            "circuit_breakers": metrics.circuit_breakers,
            "timestamp": metrics.timestamp
        })))
    } else {
        // Fallback: get directly from orchestrator's Claude client if available
        if let Ok(client) = server.orchestrator.get_claude_client() {
            let cb_metrics = client.get_circuit_breaker_metrics().await;
            Ok(Json(serde_json::json!({
                "circuit_breakers": {
                    "claude_code": cb_metrics
                },
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            })))
        } else {
            Err(StatusCode::SERVICE_UNAVAILABLE)
        }
    }
}
