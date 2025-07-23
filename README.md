# VS Code Agent

Ultra-simple VS Code chat agent automation that leverages the `code chat --mode='agent'` command to generate complete Rust projects.

## Features

- **Ultimate Simplicity**: Uses VS Code's built-in chat agent directly
- **Zero Infrastructure**: No containers, LSP servers, or API endpoints needed
- **Real Copilot AI**: Same quality as VS Code's integrated experience
- **Complete Projects**: Generates Cargo.toml, src/, tests/ with proper structure
- **Validated Output**: Automatically builds and tests generated code

## Prerequisites

1. **VS Code** with the `code` CLI command available in PATH
2. **GitHub Copilot** enabled in VS Code
3. **Rust toolchain** (cargo) for building generated projects

## Installation

```bash
git clone <repo>
cd vscode-agent
cargo build --release
```

## Usage

### Check Setup

```bash
cargo run -- check
```

### Generate a Project

```bash
# Simple function
cargo run -- dev "Simple: Create a function that adds two numbers"

# Complete project
cargo run -- dev "Create a REST API for user management"
```

### List Generated Workspaces

```bash
cargo run -- list
```

### Clean Old Workspaces

```bash
cargo run -- clean
```

### Run Integration Tests

```bash
cargo run -- test
```

## How It Works

1. **Create Workspace**: Creates a timestamped directory for the project
2. **Generate Prompt**: Creates appropriate prompts based on task complexity
3. **Ask Chat Agent**: Calls `code chat --mode='agent'` with the prompt
4. **Parse Response**: Extracts Cargo.toml and Rust files from the response
5. **Build & Test**: Validates the generated code with `cargo check` and `cargo test`

## Example Output

```
🤖 VS Code Chat Agent: Create a REST API for user management
⚡ Using `code chat --mode='agent'` - the simplest possible approach!
📁 Workspace: ./workspaces/20241125_143022_create-a-rest-api
🤖 Asking VS Code Chat Agent: Create a REST API for user management
✅ Chat agent responded with 2847 characters
🧠 Received 2847 characters of generated code
📝 Created 3 files:
   Cargo.toml
   src/lib.rs
   tests/integration_tests.rs
🔧 Building project...
📊 Results:
   Build: ✅ PASSED
   Tests: 4 passed, 0 failed

🎉 Task completed!
📂 Code location: ./workspaces/20241125_143022_create-a-rest-api
```

## Environment Variables

- `WORKSPACE_DIR`: Directory for generated workspaces (default: `./workspaces`)

## Architecture

This tool demonstrates the power of simplicity - instead of building complex infrastructure, it leverages VS Code's existing chat agent functionality with minimal wrapper code. The entire implementation is ~300 lines of Rust that handles:

- CLI interface with clap
- VS Code chat agent communication
- Code parsing and file generation
- Project building and testing
- Workspace management

The result is a robust, production-ready tool that generates high-quality Rust projects using the same AI that powers VS Code's integrated experience.
