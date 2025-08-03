# Spiral Core Monorepo Structure Proposal

## 🎯 Vision: Unified AI Agent Ecosystem

Transform Spiral Core into a comprehensive monorepo supporting AI agents that can:

- Fetch and analyze any public repository
- Generate pull requests with improvements
- Create new repositories from templates
- Perform cross-codebase pattern analysis
- Execute tasks in isolated, clean workspaces

## 📁 Proposed Structure

```
spiral-core/                           # Main monorepo root
├── Cargo.toml                        # Workspace Cargo.toml
├── README.md                         # Main project documentation
├── LICENSE                           # MIT license
├── .gitignore                        # Comprehensive gitignore
├── .github/
│   ├── workflows/                    # CI/CD for entire monorepo
│   │   ├── test-agents.yml          # Test all agent components
│   │   ├── test-tools.yml           # Test all tools
│   │   └── security-audit.yml       # Security scanning
│   └── ISSUE_TEMPLATE/              # Issue templates for agents/tools
├── docs/                            # Comprehensive documentation
│   ├── ARCHITECTURE.md              # Overall system architecture
│   ├── MONOREPO_GUIDE.md           # How to work with monorepo
│   ├── AGENT_DEVELOPMENT.md        # Guide for creating new agents
│   ├── TOOL_DEVELOPMENT.md         # Guide for creating new tools
│   └── proximity-patterns/         # Our proximity pattern documentation
├── crates/                          # Core system crates
│   ├── spiral-core/                # Main orchestration system
│   │   ├── Cargo.toml
│   │   └── src/                    # Current spiral-core code
│   ├── spiral-agents/              # Agent trait definitions and base agents
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs           # Agent trait definitions
│   │       ├── developer/          # Software developer agent
│   │       ├── project_manager/    # Project manager agent
│   │       ├── qa/                 # Quality assurance agent
│   │       └── spiral_king/        # Lordgenome spiral king agent
│   ├── spiral-tools/               # Tool trait definitions and utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── traits.rs           # Tool trait definitions
│   │       └── workspace.rs        # Workspace management
│   └── spiral-discord/             # Discord integration
│       ├── Cargo.toml
│       └── src/                    # Current Discord bot code
├── tools/                          # Standalone tool implementations
│   ├── repo-fetcher/               # Git repository management
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── main.rs             # CLI interface
│   │       ├── lib.rs              # Library interface for agents
│   │       ├── git_ops.rs          # Git operations
│   │       ├── github_api.rs       # GitHub API integration
│   │       └── workspace_prep.rs   # Prepare workspaces for analysis
│   ├── codebase-analyzer/          # Advanced codebase analysis
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── languages/          # Language-specific analyzers
│   │       │   ├── rust.rs
│   │       │   ├── typescript.rs
│   │       │   ├── python.rs
│   │       │   └── go.rs
│   │       ├── patterns/           # Code pattern detection
│   │       │   ├── security.rs     # Security pattern analysis
│   │       │   └── performance.rs  # Performance pattern analysis
│   │       └── metrics/            # Code quality metrics
│   ├── proximity-analyzer/         # Aggressive proximity pattern analysis
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── main.rs             # CLI interface
│   │       ├── lib.rs              # Library interface for agents
│   │       ├── analyzers/          # Core analysis modules
│   │       │   ├── decision_comments.rs  # Decision archaeology analysis
│   │       │   ├── test_colocation.rs    # Test colocation analysis
│   │       │   ├── abstraction_tracker.rs # 3-strikes rule tracking
│   │       │   └── security_audit.rs     # Security audit checkpoint analysis
│   │       ├── reporters/          # Output formatters
│   │       │   ├── markdown.rs     # Markdown audit reports
│   │       │   ├── json.rs         # JSON output for agents
│   │       │   └── html.rs         # Interactive HTML reports
│   │       ├── fixers/             # Automatic proximity fixes
│   │       │   ├── comment_generator.rs  # Generate decision archaeology
│   │       │   ├── test_colocation.rs    # Suggest test improvements
│   │       │   └── abstraction_extractor.rs # Extract 3-strikes utilities
│   │       ├── templates/          # Code templates for patterns
│   │       │   ├── decision_archaeology.rs
│   │       │   ├── startup_shutdown.rs
│   │       │   ├── three_strikes.rs
│   │       │   └── architecture_decision.rs
│   │       └── examples/           # Pattern examples
│   │           ├── basic_proximity/
│   │           ├── agent_orchestration/
│   │           ├── api_design/
│   │           └── security_patterns/
│   ├── pr-generator/               # Pull request automation
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── templates/          # PR templates for different improvements
│   │       ├── github_integration.rs
│   │       └── diff_generator.rs
│   ├── repo-creator/               # New repository creation
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── templates/          # Repository templates
│   │       └── github_setup.rs
│   └── security-auditor/           # Security-focused analysis
│       ├── Cargo.toml
│       ├── README.md
│       └── src/
│           ├── main.rs
│           ├── lib.rs
│           ├── vulnerability_scan.rs
│           └── audit_report.rs
├── workspaces/                     # Claude Code execution environments
│   ├── README.md                   # Workspace management guide
│   ├── templates/                  # Workspace templates
│   │   ├── rust-library/           # Template for analyzing Rust libraries
│   │   │   ├── Cargo.toml
│   │   │   ├── .gitignore
│   │   │   └── src/lib.rs
│   │   ├── web-api/                # Template for web API analysis
│   │   │   ├── Cargo.toml
│   │   │   ├── package.json
│   │   │   └── src/
│   │   ├── ai-agent/               # Template for AI system analysis
│   │   └── security-audit/         # Template for security reviews
│   ├── active/                     # Active workspace sessions
│   │   ├── session-001/            # Isolated workspace for task 001
│   │   ├── session-002/            # Isolated workspace for task 002
│   │   └── .gitignore              # Ignore active sessions
│   └── cleanup.rs                  # Workspace cleanup utilities
├── examples/                       # Example projects and demonstrations
│   ├── public-repo-analysis/       # Examples of analyzing public repos
│   ├── pr-improvements/            # Examples of generated improvements
│   ├── security-audits/            # Examples of security analysis
│   └── proximity-patterns/         # Our proximity pattern examples
├── tests/                          # Integration tests
│   ├── agent_integration.rs        # Test agent coordination
│   ├── tool_integration.rs         # Test tool interoperability
│   ├── workspace_isolation.rs      # Test workspace isolation
│   └── end_to_end.rs               # Full system tests
└── scripts/                        # Development and deployment scripts
    ├── setup.sh                    # Initial monorepo setup
    ├── test-all.sh                 # Run all tests
    ├── clean-workspaces.sh         # Clean up workspace sessions
    └── deploy.sh                   # Deployment script
```

## 🚀 Benefits of This Structure

### 1. 🤖 Agent Tool Discovery

```rust
// Agents can discover and use all available tools
use spiral_tools::ToolRegistry;

let tools = ToolRegistry::discover_all()?;
let repo_fetcher = tools.get_tool::<RepoFetcher>()?;
let analyzer = tools.get_tool::<CodebaseAnalyzer>()?;
let pr_generator = tools.get_tool::<PrGenerator>()?;
```

### 2. 🔄 Workspace Management

```rust
// Clean, isolated workspaces for each task
use spiral_tools::workspace::WorkspaceManager;

let workspace = WorkspaceManager::create_from_template("rust-library")?;
let repo = repo_fetcher.clone_to_workspace(&workspace, "https://github.com/example/repo")?;
let analysis = analyzer.analyze_codebase(&workspace)?;
```

### 3. 📊 Cross-Project Learning

```rust
// Agents learn from previous analyses
use spiral_tools::patterns::PatternDatabase;

let patterns = PatternDatabase::load_learned_patterns()?;
let context = build_analysis_context(&repo, &patterns);
let improvements = generate_improvements_with_context(&context)?;
```

### 4. 🔧 Unified Configuration

```rust
// Single configuration system for all components
use spiral_core::config::Config;

let config = Config::load()?;
let github_token = config.github.token;  // Used by multiple tools
let claude_config = config.claude_code;  // Shared Claude Code settings
```

## 🎯 Implementation Strategy

### Phase 1: Restructure Existing Code

1. Move current `src/` to `crates/spiral-core/src/`
2. Extract agent logic to `crates/spiral-agents/`
3. Extract Discord bot to `crates/spiral-discord/`
4. Create workspace Cargo.toml

### Phase 2: Add Tool Infrastructure

1. Create `tools/repo-fetcher/`
2. Create `tools/codebase-analyzer/`
3. Create `workspaces/templates/`
4. Implement tool discovery system

### Phase 3: Advanced Capabilities

1. Add `tools/pr-generator/`
2. Add `tools/repo-creator/`
3. Add cross-project learning
4. Implement pattern database

### Phase 4: Polish and Documentation

1. Comprehensive integration tests
2. Documentation for each component
3. Example workflows
4. CI/CD for monorepo

## 🔍 Workspace Isolation Strategy

**Each task gets clean environment:**

```rust
// Task execution with isolation
async fn execute_task_with_isolation(task: Task) -> Result<TaskResult> {
    let workspace = WorkspaceManager::create_isolated(&task.id)?;

    // Clone repository to isolated workspace
    let repo = fetch_repo_to_workspace(&task.repo_url, &workspace).await?;

    // Analyze in isolation
    let analysis = analyze_codebase_in_workspace(&workspace).await?;

    // Generate improvements
    let improvements = generate_improvements(&analysis).await?;

    // Create PR (if requested)
    if task.create_pr {
        let pr = create_pull_request(&repo, &improvements).await?;
    }

    // Cleanup workspace
    workspace.cleanup().await?;

    Ok(TaskResult::success(improvements))
}
```

## 🛡️ Security Considerations

**Workspace Security:**

- Each workspace is completely isolated
- No access to parent filesystem
- Network access controlled per task
- Git credentials scoped appropriately
- Automatic cleanup prevents data leaks

**Tool Security:**

- GitHub tokens with minimal required permissions
- API rate limiting and abuse prevention
- Audit trails for all repository operations
- Secure handling of private repository access

## 📊 Advantages vs Traditional Multi-Repo

| Aspect                     | Monorepo       | Multi-Repo             |
| -------------------------- | -------------- | ---------------------- |
| **Tool Discovery**         | ✅ Automatic   | ❌ Manual coordination |
| **Workspace Templates**    | ✅ Shared      | ❌ Duplicated          |
| **Cross-Project Learning** | ✅ Built-in    | ❌ Complex             |
| **Dependency Management**  | ✅ Unified     | ❌ Version conflicts   |
| **CI/CD**                  | ✅ Coordinated | ❌ Separate            |
| **Agent Development**      | ✅ Integrated  | ❌ Fragmented          |

## 🎭 Use Cases Enabled

1. **Public Repository Analysis**:
   - Fetch any GitHub repository
   - Analyze for proximity patterns, security issues, performance
   - Generate comprehensive audit reports

2. **Automated Improvements**:
   - Detect improvement opportunities
   - Generate pull requests with fixes
   - Apply proximity patterns automatically

3. **Template Creation**:
   - Analyze multiple similar projects
   - Extract common patterns into templates
   - Create new repositories from learned patterns

4. **Security Auditing**:
   - Scan codebases for security vulnerabilities
   - Generate security improvement recommendations
   - Track security metrics across projects

5. **Learning and Adaptation**:
   - Learn from each analysis
   - Improve recommendations over time
   - Build domain-specific expertise

This monorepo structure would transform Spiral Core from a single-purpose agent system into a comprehensive AI-powered development platform capable of understanding, improving, and creating codebases at scale.
