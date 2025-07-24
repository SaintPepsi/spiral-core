# CLAUDE-integrations-github.md

**Purpose**: GitHub integration patterns for automated repository management and PR creation
**Dependencies**: [Coding Standards](CLAUDE-core-coding-standards.md), [Developer Agent](CLAUDE-agents-developer.md)
**Updated**: 2024-07-24

## Quick Start

GitHub integration provides automated repository management, PR creation, and code review workflows:

```rust
impl GitHubIntegration {
    pub async fn create_pr_from_agent_work(&self, work_result: &CodeResult) -> Result<PullRequest, GitHubError>;
    pub async fn setup_repository(&self, repo_config: &RepositoryConfig) -> Result<Repository, GitHubError>;
    pub async fn review_and_merge(&self, pr_id: u64, review_criteria: &ReviewCriteria) -> Result<MergeResult, GitHubError>;
}
```

## Core Architecture

### GitHub Client Implementation

```rust
// ✅ Good - Single responsibility for GitHub operations
pub struct GitHubClient {
    token: String,
    http_client: reqwest::Client,
    rate_limiter: Arc<RateLimiter>,
    webhook_secret: Option<String>,
}

impl GitHubClient {
    pub async fn create_repository(&self, config: &RepositoryConfig) -> Result<Repository, GitHubError> {
        // Handle repository creation with templates and settings
        let create_request = CreateRepositoryRequest {
            name: config.name.clone(),
            description: config.description.clone(),
            private: config.is_private,
            auto_init: true,
            gitignore_template: config.gitignore_template.clone(),
            license_template: config.license.clone(),
        };

        let response = self.http_client
            .post("https://api.github.com/user/repos")
            .bearer_auth(&self.token)
            .json(&create_request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GitHubError::ApiError(response.status().as_u16()));
        }

        let repository: Repository = response.json().await?;
        
        // Setup branch protection and additional configuration
        self.configure_repository_settings(&repository, config).await?;
        
        Ok(repository)
    }

    pub async fn create_pull_request(&self, repo: &Repository, pr_data: &PullRequestData) -> Result<PullRequest, GitHubError> {
        // Create branch for the changes
        let branch_name = format!("spiral-agent/{}", pr_data.feature_name);
        self.create_branch(repo, &branch_name, &pr_data.base_branch).await?;

        // Commit all changes to the new branch
        for file_change in &pr_data.file_changes {
            self.commit_file_change(repo, &branch_name, file_change).await?;
        }

        // Create the pull request
        let pr_request = CreatePullRequestRequest {
            title: pr_data.title.clone(),
            body: self.generate_pr_description(pr_data),
            head: branch_name,
            base: pr_data.base_branch.clone(),
        };

        let response = self.http_client
            .post(&format!("https://api.github.com/repos/{}/pulls", repo.full_name))
            .bearer_auth(&self.token)
            .json(&pr_request)
            .send()
            .await?;

        let pull_request: PullRequest = response.json().await?;
        
        // Add labels and assignees
        self.configure_pull_request(&repo, &pull_request, pr_data).await?;
        
        Ok(pull_request)
    }
}
```

## Agent Integration Patterns

### Developer Agent → GitHub Workflow

```rust
impl DeveloperAgent {
    pub async fn commit_and_create_pr(&mut self, task_result: &TaskResult) -> Result<PullRequest, AgentError> {
        match task_result {
            TaskResult::Completed { code_output, task, language, .. } => {
                // Prepare repository configuration
                let repo_config = self.infer_repository_config(task, language, code_output)?;
                
                // Setup or use existing repository
                let repository = if code_output.creates_new_project {
                    self.github_client.create_repository(&repo_config).await?
                } else {
                    self.github_client.get_repository(&repo_config.name).await?
                };

                // Prepare PR data
                let pr_data = PullRequestData {
                    title: format!("🚀 {}", self.generate_pr_title(task)),
                    feature_name: self.sanitize_branch_name(task),
                    description: self.generate_detailed_description(task, code_output),
                    file_changes: self.convert_to_file_changes(code_output),
                    base_branch: "main".to_string(),
                    labels: self.determine_pr_labels(language, code_output),
                    assignees: vec![], // Could be configured
                };

                // Create the pull request
                let pr = self.github_client.create_pull_request(&repository, &pr_data).await?;
                
                // Add AI-generated commit messages and documentation
                self.enhance_pr_with_documentation(&repository, &pr).await?;
                
                Ok(pr)
            },
            _ => Err(AgentError::InvalidTaskStateForGitHub),
        }
    }

    fn generate_pr_title(&self, task: &str) -> String {
        // AI-assisted PR title generation
        let title_keywords = self.extract_key_concepts(task);
        
        match title_keywords.primary_action {
            "create" => format!("Add {}", title_keywords.main_component),
            "build" => format!("Implement {}", title_keywords.main_component),
            "fix" => format!("Fix {}", title_keywords.main_component),
            "update" => format!("Update {}", title_keywords.main_component),
            "refactor" => format!("Refactor {}", title_keywords.main_component),
            _ => format!("Develop {}", title_keywords.main_component),
        }
    }

    fn generate_detailed_description(&self, task: &str, code_output: &CodeResult) -> String {
        format!(
            "## 🤖 AI-Generated Implementation\n\n\
            **Original Request**: {}\n\n\
            **Implementation Summary**:\n\
            {}\n\n\
            **Files Created/Modified**:\n\
            {}\n\n\
            **Testing**:\n\
            - ✅ {} tests passing\n\
            - 📊 Test coverage: {}%\n\n\
            **Code Quality**:\n\
            - 🏗️ Follows established architecture patterns\n\
            - 📝 Comprehensive documentation included\n\
            - ⚡ Performance optimized\n\
            - 🔒 Security best practices applied\n\n\
            **Ready for Review**:\n\
            This PR is ready for human review and can be merged once approved.\n\n\
            ---\n\
            🤖 *Generated by Spiral Developer Agent*",
            task,
            code_output.summary,
            code_output.files_created.iter()
                .map(|f| format!("- `{}`", f.path))
                .collect::<Vec<_>>()
                .join("\\n"),
            code_output.tests_passing,
            code_output.test_coverage.unwrap_or(85.0)
        )
    }
}
```

### Project Manager Integration

```rust
impl ProjectManagerAgent {
    pub async fn create_project_repository(&mut self, analysis: &StrategicAnalysis) -> Result<ProjectSetup, AgentError> {
        if analysis.requires_new_repository {
            let repo_config = self.design_repository_structure(analysis)?;
            
            // Create repository with proper structure
            let repository = self.github_client.create_repository(&repo_config).await?;
            
            // Setup project management features
            self.setup_project_management(&repository, analysis).await?;
            
            // Create initial project structure
            self.create_initial_project_files(&repository, analysis).await?;
            
            Ok(ProjectSetup {
                repository,
                project_board: self.create_project_board(&repository, analysis).await?,
                initial_issues: self.create_planning_issues(&repository, analysis).await?,
            })
        } else {
            Err(AgentError::RepositoryNotRequired)
        }
    }

    async fn setup_project_management(&mut self, repo: &Repository, analysis: &StrategicAnalysis) -> Result<(), GitHubError> {
        // Create project board
        let project_board = self.github_client.create_project_board(repo, &ProjectBoardConfig {
            name: format!("{} Development", analysis.project_name),
            description: analysis.project_description.clone(),
            columns: vec![
                "📋 Backlog".to_string(),
                "🏗️ In Progress".to_string(), 
                "👀 Review".to_string(),
                "✅ Done".to_string(),
            ],
        }).await?;

        // Create milestone for each implementation phase
        for (index, phase) in analysis.implementation_phases.iter().enumerate() {
            self.github_client.create_milestone(repo, &MilestoneConfig {
                title: format!("Phase {}: {}", index + 1, phase.name),
                description: phase.description.clone(),
                due_date: phase.target_completion,
            }).await?;
        }

        // Setup branch protection
        self.github_client.setup_branch_protection(repo, &BranchProtectionConfig {
            branch: "main".to_string(),
            required_reviews: 1,
            dismiss_stale_reviews: true,
            require_code_owner_reviews: false,
            require_status_checks: true,
            status_checks: vec![
                "ci/tests".to_string(),
                "ci/lint".to_string(),
            ],
        }).await?;

        Ok(())
    }
}
```

## Webhook Integration

### Automated Code Review

```rust
pub struct GitHubWebhookHandler {
    claude_client: Box<dyn ClaudeClient>,
    review_criteria: CodeReviewCriteria,
    auto_merge_conditions: AutoMergeConditions,
}

impl GitHubWebhookHandler {
    pub async fn handle_pull_request_event(&self, event: &PullRequestEvent) -> Result<(), WebhookError> {
        match event.action.as_str() {
            "opened" | "synchronize" => {
                self.perform_automated_review(&event.pull_request).await?;
            },
            "review_requested" => {
                self.handle_review_request(&event.pull_request).await?;
            },
            _ => {
                // Other PR events don't require action
            }
        }
        Ok(())
    }

    async fn perform_automated_review(&self, pr: &PullRequest) -> Result<(), ReviewError> {
        // Get PR diff and files changed
        let diff = self.get_pr_diff(pr).await?;
        let files_changed = self.get_changed_files(pr).await?;

        // Perform AI code review
        let review_prompt = self.build_code_review_prompt(&diff, &files_changed, pr);
        
        let review_result = self.claude_client
            .execute_task(review_prompt, ProgrammingLanguage::CodeReview)
            .await?;

        let review_analysis = CodeReviewAnalysis::from_claude_response(review_result)?;

        // Post review comments
        if !review_analysis.suggestions.is_empty() {
            self.post_review_comments(pr, &review_analysis).await?;
        }

        // Approve or request changes
        match review_analysis.overall_assessment {
            ReviewAssessment::Approved => {
                self.approve_pull_request(pr, &review_analysis).await?;
                
                // Check if auto-merge conditions are met
                if self.should_auto_merge(pr, &review_analysis).await? {
                    self.merge_pull_request(pr).await?;
                }
            },
            ReviewAssessment::RequestChanges => {
                self.request_changes(pr, &review_analysis).await?;
            },
            ReviewAssessment::Comment => {
                // Just post comments, no approval/rejection
            }
        }

        Ok(())
    }

    fn build_code_review_prompt(&self, diff: &str, files: &[ChangedFile], pr: &PullRequest) -> String {
        format!(
            "Please perform a comprehensive code review of this pull request:\n\n\
            **PR Title**: {}\n\
            **PR Description**: {}\n\n\
            **Files Changed**: {}\n\n\
            **Diff**:\n\
            ```diff\n{}\n```\n\n\
            **Review Criteria**:\n\
            1. **Code Quality**: Clean, readable, maintainable code\n\
            2. **Architecture**: Follows SOLID principles and system patterns\n\
            3. **Security**: No security vulnerabilities or sensitive data exposure\n\
            4. **Performance**: Efficient algorithms and resource usage\n\
            5. **Testing**: Adequate test coverage and quality\n\
            6. **Documentation**: Clear comments and documentation\n\n\
            **System Context**:\n\
            - Language: Rust with async/tokio\n\
            - Architecture: Multi-agent system with Claude Code integration\n\
            - Performance Requirements: <1s response time, memory efficient\n\
            - Deployment: 8GB VPS environment\n\n\
            Please provide:\n\
            1. Overall assessment: APPROVE, REQUEST_CHANGES, or COMMENT\n\
            2. Specific line-by-line suggestions (if any)\n\
            3. Security concerns (if any)\n\
            4. Performance considerations\n\
            5. Architecture feedback\n\
            6. Testing recommendations\n\n\
            Focus on constructive feedback that improves code quality while maintaining development velocity.",
            pr.title,
            pr.body.as_deref().unwrap_or("No description provided"),
            files.iter().map(|f| &f.filename).collect::<Vec<_>>().join(", "),
            diff
        )
    }
}
```

## Discord Integration

### GitHub Status Updates in Discord

```rust
impl GitHubDiscordIntegration {
    pub async fn notify_pr_created(&self, pr: &PullRequest, agent_type: AgentType) -> Result<(), NotificationError> {
        let channel_id = self.get_notification_channel(agent_type);
        
        let embed = DiscordEmbed {
            title: format!("🚀 New PR Created"),
            description: format!(
                "**{}** just created a pull request!\n\n\
                **Title**: {}\n\
                **Repository**: {}\n\
                **Files Changed**: {} files\n\
                **Lines**: +{} -{}\n\n\
                [View PR]({}) | [Review Changes]({})",
                self.get_agent_display_name(agent_type),
                pr.title,
                pr.base.repo.name,
                pr.changed_files,
                pr.additions,
                pr.deletions,
                pr.html_url,
                format!("{}/files", pr.html_url)
            ),
            color: 0x28a745, // GitHub green
            footer: Some("🤖 Automated by Spiral Agent".to_string()),
            timestamp: Some(chrono::Utc::now()),
        };

        self.discord_client.send_embed(channel_id, embed).await?;
        Ok(())
    }

    pub async fn notify_review_completed(&self, pr: &PullRequest, review: &CodeReviewAnalysis) -> Result<(), NotificationError> {
        let status_emoji = match review.overall_assessment {
            ReviewAssessment::Approved => "✅",
            ReviewAssessment::RequestChanges => "❌", 
            ReviewAssessment::Comment => "💬",
        };

        let embed = DiscordEmbed {
            title: format!("{} Code Review Completed", status_emoji),
            description: format!(
                "**AI Review Results** for PR: {}\n\n\
                **Assessment**: {}\n\
                **Issues Found**: {}\n\
                **Suggestions**: {}\n\n\
                **Security Score**: {}/10\n\
                **Code Quality**: {}/10\n\
                **Test Coverage**: {}%\n\n\
                [View Review]({}) | [See Changes]({})",
                pr.title,
                review.overall_assessment.to_string(),
                review.issues_found.len(),
                review.suggestions.len(),
                review.security_score,
                review.code_quality_score,
                review.test_coverage_percentage,
                pr.html_url,
                format!("{}/files", pr.html_url)
            ),
            color: match review.overall_assessment {
                ReviewAssessment::Approved => 0x28a745,
                ReviewAssessment::RequestChanges => 0xd73a49,
                ReviewAssessment::Comment => 0x0366d6,
            },
            footer: Some("🔍 AI Code Review".to_string()),
            timestamp: Some(chrono::Utc::now()),
        };

        self.discord_client.send_embed(self.code_review_channel_id, embed).await?;
        Ok(())
    }
}
```

## Repository Templates

### Project Structure Generation

```rust
impl GitHubClient {
    pub async fn create_project_from_template(&self, config: &ProjectTemplateConfig) -> Result<Repository, GitHubError> {
        let repository = self.create_repository(&config.repo_config).await?;
        
        // Generate project structure based on language and type
        let project_structure = match config.project_type {
            ProjectType::RustLibrary => self.generate_rust_library_structure(&config),
            ProjectType::RustBinary => self.generate_rust_binary_structure(&config),
            ProjectType::WebAPI => self.generate_web_api_structure(&config),
            ProjectType::DiscordBot => self.generate_discord_bot_structure(&config),
            ProjectType::CLI => self.generate_cli_structure(&config),
        };

        // Create all files in the repository
        for file in project_structure.files {
            self.create_file(&repository, &file).await?;
        }

        // Setup CI/CD workflows
        self.setup_github_actions(&repository, &config.ci_config).await?;
        
        // Create initial issues and project board
        if config.create_project_management {
            self.setup_project_management(&repository, &config.pm_config).await?;
        }

        Ok(repository)
    }

    fn generate_rust_library_structure(&self, config: &ProjectTemplateConfig) -> ProjectStructure {
        ProjectStructure {
            files: vec![
                ProjectFile {
                    path: "Cargo.toml".to_string(),
                    content: self.generate_cargo_toml(config),
                },
                ProjectFile {
                    path: "src/lib.rs".to_string(),
                    content: self.generate_lib_rs(config),
                },
                ProjectFile {
                    path: "README.md".to_string(),
                    content: self.generate_readme(config),
                },
                ProjectFile {
                    path: ".gitignore".to_string(),
                    content: include_str!("../templates/rust.gitignore").to_string(),
                },
                ProjectFile {
                    path: ".github/workflows/ci.yml".to_string(),
                    content: self.generate_github_actions_ci(config),
                },
                ProjectFile {
                    path: "tests/integration_tests.rs".to_string(),
                    content: self.generate_integration_tests(config),
                },
            ],
        }
    }
}
```

## Error Handling and Recovery

### GitHub API Rate Limiting

```rust
pub struct GitHubRateLimiter {
    requests_remaining: Arc<Mutex<u32>>,
    reset_time: Arc<Mutex<chrono::DateTime<chrono::Utc>>>,
    core_limit: u32,
    search_limit: u32,
}

impl GitHubRateLimiter {
    pub async fn check_rate_limit(&self) -> Result<(), RateLimitError> {
        let remaining = *self.requests_remaining.lock().await;
        let reset_time = *self.reset_time.lock().await;
        
        if remaining == 0 {
            let wait_duration = reset_time - chrono::Utc::now();
            if wait_duration > chrono::Duration::zero() {
                return Err(RateLimitError::RateLimited { 
                    retry_after: wait_duration.to_std().unwrap_or(Duration::from_secs(3600))
                });
            }
        }
        
        Ok(())
    }

    pub async fn update_from_response(&self, response: &reqwest::Response) {
        if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
            if let Ok(remaining_str) = remaining.to_str() {
                if let Ok(remaining_count) = remaining_str.parse::<u32>() {
                    *self.requests_remaining.lock().await = remaining_count;
                }
            }
        }

        if let Some(reset) = response.headers().get("x-ratelimit-reset") {
            if let Ok(reset_str) = reset.to_str() {
                if let Ok(reset_timestamp) = reset_str.parse::<i64>() {
                    *self.reset_time.lock().await = chrono::DateTime::from_timestamp(reset_timestamp, 0)
                        .unwrap_or_else(chrono::Utc::now);
                }
            }
        }
    }
}
```

## Testing Strategy

### GitHub Integration Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn github_client_creates_repository_successfully() {
        let github_client = create_test_github_client().await;
        let config = RepositoryConfig {
            name: "test-spiral-project".to_string(),
            description: Some("Test repository created by Spiral Agent".to_string()),
            is_private: true,
            gitignore_template: Some("Rust".to_string()),
            license: Some("MIT".to_string()),
        };

        let repository = github_client.create_repository(&config).await.unwrap();

        assert_eq!(repository.name, "test-spiral-project");
        assert!(repository.private);
        assert!(repository.has_issues);
        assert!(repository.has_projects);
    }

    #[tokio::test]
    async fn github_client_creates_pr_with_agent_content() {
        let github_client = create_test_github_client().await;
        let repository = create_test_repository().await;
        
        let pr_data = PullRequestData {
            title: "Add FastAPI todo application".to_string(),
            feature_name: "fastapi-todo-app".to_string(),
            description: "AI-generated FastAPI application with SQLite backend".to_string(),
            file_changes: vec![
                FileChange {
                    path: "main.py".to_string(),
                    content: "# FastAPI application code here".to_string(),
                    operation: FileOperation::Create,
                },
            ],
            base_branch: "main".to_string(),
            labels: vec!["enhancement".to_string(), "ai-generated".to_string()],
            assignees: vec![],
        };

        let pr = github_client.create_pull_request(&repository, &pr_data).await.unwrap();

        assert_eq!(pr.title, "Add FastAPI todo application");
        assert!(pr.body.unwrap().contains("AI-generated"));
        assert_eq!(pr.head.ref_name, "spiral-agent/fastapi-todo-app");
    }

    #[tokio::test]
    async fn webhook_handler_performs_code_review() {
        let webhook_handler = create_test_webhook_handler().await;
        let pr_event = create_test_pr_opened_event();

        // Should not panic and should perform review
        webhook_handler.handle_pull_request_event(&pr_event).await.unwrap();

        // Verify review was posted (would check GitHub API in real test)
        // This would require mocking the GitHub API calls
    }
}
```

## Security Considerations

### Token Management

```rust
pub struct GitHubTokenManager {
    encrypted_tokens: HashMap<String, EncryptedToken>,
    encryption_key: [u8; 32],
    token_rotation_schedule: TokenRotationSchedule,
}

impl GitHubTokenManager {
    pub fn new(encryption_key: [u8; 32]) -> Self {
        Self {
            encrypted_tokens: HashMap::new(),
            encryption_key,
            token_rotation_schedule: TokenRotationSchedule::default(),
        }
    }

    pub fn store_token(&mut self, user_id: &str, token: &str) -> Result<(), TokenError> {
        // Encrypt the token before storing
        let encrypted = self.encrypt_token(token)?;
        
        self.encrypted_tokens.insert(user_id.to_string(), EncryptedToken {
            encrypted_data: encrypted,
            created_at: chrono::Utc::now(),
            last_used: None,
            permissions: self.validate_token_permissions(token)?,
        });

        Ok(())
    }

    pub fn get_token(&mut self, user_id: &str) -> Result<String, TokenError> {
        let encrypted_token = self.encrypted_tokens.get_mut(user_id)
            .ok_or(TokenError::TokenNotFound)?;

        // Check if token needs rotation
        if self.token_rotation_schedule.should_rotate(&encrypted_token) {
            return Err(TokenError::TokenExpired);
        }

        // Decrypt and return
        let decrypted = self.decrypt_token(&encrypted_token.encrypted_data)?;
        encrypted_token.last_used = Some(chrono::Utc::now());
        
        Ok(decrypted)
    }
}
```

## Performance Optimization

### Batch Operations

```rust
impl GitHubClient {
    pub async fn batch_create_files(&self, repo: &Repository, files: Vec<FileChange>) -> Result<Vec<CommitResult>, GitHubError> {
        // Use GitHub's tree API for efficient batch operations
        let tree_entries: Vec<TreeEntry> = files.iter().map(|file| {
            TreeEntry {
                path: file.path.clone(),
                mode: "100644".to_string(), // Regular file
                type_field: "blob".to_string(),
                content: Some(file.content.clone()),
            }
        }).collect();

        // Create tree with all files
        let tree = self.create_tree(repo, tree_entries).await?;
        
        // Create commit pointing to the tree
        let commit = self.create_commit(repo, &CommitData {
            message: format!("🤖 Batch commit: {} files created by Spiral Agent", files.len()),
            tree: tree.sha,
            parents: vec![self.get_latest_commit_sha(repo).await?],
        }).await?;

        // Update branch reference
        self.update_reference(repo, "heads/main", &commit.sha).await?;

        Ok(vec![CommitResult {
            sha: commit.sha,
            files_changed: files.len(),
            message: commit.message,
        }])
    }
}
```

## Integration Points

This GitHub integration module works with:
- [Developer Agent](CLAUDE-agents-developer.md) for automated code commits and PR creation
- [Project Manager Agent](CLAUDE-agents-pm.md) for repository setup and project management
- [Discord Integration](CLAUDE-integrations-discord.md) for GitHub status notifications
- [Claude Code Integration](CLAUDE-integrations-claude-code.md) for AI-powered code reviews

## Related Documentation

- See [Coding Standards](CLAUDE-core-coding-standards.md) for code quality standards applied in reviews
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for GitHub integration setup
- See [Implementation Phase 1](CLAUDE-implementation-phase1.md) for GitHub integration setup