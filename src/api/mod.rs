use crate::{
    agents::AgentOrchestrator,
    auth::{auth_middleware, create_auth_state},
    config::{ApiConfig, Config},
    models::{AgentType, Priority, Task, TaskStatus},
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
use tracing::{info, warn};

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    orchestrator: Arc<AgentOrchestrator>,
    validator: TaskContentValidator,
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
            // rate_limiter,
        })
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

    pub fn build_router(&self) -> Router {
        let auth_state = create_auth_state(self.config.clone());

        // SECURITY: Configure restrictive CORS policy
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
            .route("/health", get(health_check))
            .route("/tasks", post(create_task))
            .route("/tasks/:task_id", get(get_task_status))
            .route("/tasks/:task_id/analyze", post(analyze_task))
            .route("/agents", get(get_all_agent_statuses))
            .route("/agents/:agent_type", get(get_agent_status))
            .route("/system/status", get(get_system_status))
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

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "spiral-core",
        "version": "0.1.0"
    }))
}

/// 📝 CREATE TASK ENDPOINT: Primary user request entry point
/// AUDIT CHECKPOINT: Critical security and validation path
/// Verify: Authentication, rate limiting, content validation, orchestrator submission
async fn create_task(
    State(api_server): State<ApiServer>,
    Json(request): Json<CreateTaskRequest>,
) -> std::result::Result<Json<CreateTaskResponse>, (StatusCode, Json<ErrorResponse>)> {
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
                    error: "Invalid task content".to_string(),
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
                        error: "Invalid context key".to_string(),
                        details: None, // SECURITY: Don't expose key validation rules
                    }),
                ));
            }

            // 🛡️ VALUE SANITIZATION: Clean potentially malicious context values
            let sanitized_value = match api_server
                .validator
                .validate_and_sanitize_context_value(&value)
            {
                Ok(val) => val,
                Err(_) => {
                    warn!(
                        "Invalid context value for key '{}': {}",
                        key,
                        &value[..std::cmp::min(50, value.len())]
                    );
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Invalid context value".to_string(),
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
            Ok(Json(CreateTaskResponse {
                task_id,
                status: "submitted".to_string(),
            }))
        }
        Err(e) => {
            // 🚨 SUBMISSION FAILURE AUDIT CHECKPOINT: System capacity or validation issue
            // CRITICAL: Could indicate system overload, agent unavailability, or attack
            warn!("Failed to submit task to orchestrator: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
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
                    error: "Internal server error".to_string(),
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
                    error: "Agent not found".to_string(),
                    details: Some(format!("Agent type: {agent_type_str}")),
                }),
            ));
        }
    };
    match api_server.orchestrator.get_agent_status(&agent_type).await {
        Some(status) => Ok(Json(AgentStatusResponse {
            agent_type: status.agent_type,
            is_busy: status.is_busy,
            current_task_id: status.current_task_id,
            tasks_completed: status.tasks_completed,
            tasks_failed: status.tasks_failed,
            average_execution_time: status.average_execution_time,
        })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Agent not found".to_string(),
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
        .map(|(agent_type, status)| {
            (
                agent_type,
                AgentStatusResponse {
                    agent_type: status.agent_type,
                    is_busy: status.is_busy,
                    current_task_id: status.current_task_id,
                    tasks_completed: status.tasks_completed,
                    tasks_failed: status.tasks_failed,
                    average_execution_time: status.average_execution_time,
                },
            )
        })
        .collect();

    Json(response)
}

async fn get_system_status(State(api_server): State<ApiServer>) -> Json<SystemStatusResponse> {
    let agent_statuses = api_server.orchestrator.get_all_agent_statuses().await;
    let queue_length = api_server.orchestrator.get_queue_length().await;
    let system_uptime = api_server.orchestrator.get_system_uptime().await;

    let agents: HashMap<AgentType, AgentStatusResponse> = agent_statuses
        .into_iter()
        .map(|(agent_type, status)| {
            (
                agent_type,
                AgentStatusResponse {
                    agent_type: status.agent_type,
                    is_busy: status.is_busy,
                    current_task_id: status.current_task_id,
                    tasks_completed: status.tasks_completed,
                    tasks_failed: status.tasks_failed,
                    average_execution_time: status.average_execution_time,
                },
            )
        })
        .collect();

    Json(SystemStatusResponse {
        agents,
        queue_length,
        system_uptime,
    })
}
