# Spiral Core Architecture Overview

This document provides a high-level overview of the Spiral Core AI agent orchestration system architecture.

## System Overview

A high-performance, memory-efficient agent system built in Rust where agents collaborate through Claude Code integration to build tools and manage complex workflows. The architecture emphasizes simplicity, compile-time safety, and efficient resource management for deployment on 8GB VPS or Apple Silicon hardware.

## Why Rust for AI Agent Orchestration

### Resource Efficiency Philosophy

**Principle**: Maximize capability while minimizing resource consumption for deployment on modest hardware (8GB VPS or Apple Silicon).

**Approach**: Agents serve as intelligent orchestrators of Claude Code capabilities rather than managing local LLM inference, dramatically reducing complexity while maintaining sophisticated functionality.

**Benefits**:

- **Predictable Memory Usage**: ~2.1GB total system footprint vs 8GB+ with local model architectures
- **Fast Startup**: Sub-second initialization vs minutes for model loading
- **Horizontal Scaling**: Multiple lightweight agents vs single heavyweight processes
- **External Intelligence**: Leverage Claude Code API for sophisticated reasoning

### Compile-Time Safety Philosophy

**Principle**: Prevent coordination bugs and race conditions through Rust's type system rather than runtime checks.

**Approach**: Model agent states, resource allocation, and inter-agent communication as types that the compiler can verify.

**Benefits**:

- **Impossible Invalid States**: Type system prevents agents from entering undefined coordination states
- **Memory Safety**: No coordination deadlocks or resource leaks
- **Predictable Behavior**: Agent interactions are deterministic and verifiable
- **Development Velocity**: Catch coordination bugs at compile time, not in production

## Key Simplification

**Agents serve as intelligent orchestrators of Claude Code capabilities** rather than managing local LLM inference, dramatically reducing complexity while maintaining sophisticated functionality.

## Architecture Components

### Core Components

- **Rust Backend**: Agent orchestration system that coordinates with Claude Code
- **Claude Code Integration**: Primary AI engine for code generation and analysis
- **Discord Bot Service**: Node.js/TypeScript service for human interaction
- **GitHub Integration**: Automated repository management and PR creation
- **Redis Message Queues**: Asynchronous communication between agents
- **PostgreSQL Database**: Agent state and task persistence

### Agent Types

The system implements specialized orchestrators for different concerns:

1. **Software Developer Agent** - Code generation and implementation
2. **Project Manager Agent** - Strategic analysis and coordination
3. **Quality Assurance Agent** - Risk assessment and testing
4. **Decision Maker Agent** - Priority scoring and conflict resolution
5. **Creative Innovator Agent** - Alternative approaches and innovation
6. **Process Coach Agent** - Performance optimization and facilitation

## System Benefits

### Performance Advantages

- **Low Memory Footprint**: ~2GB total system usage
- **Fast Startup**: 0.3-0.8 second boot time
- **High Concurrency**: 6+ agents without resource contention
- **Efficient Scaling**: Linear resource growth with agent count

### Development Advantages

- **Compile-Time Safety**: Rust prevents coordination bugs and race conditions
- **Type Safety**: Agent interactions are validated at compile time
- **Memory Safety**: No garbage collection overhead or memory leaks
- **Concurrency Safety**: Built-in protection against data races

### Operational Advantages

- **Resource Efficient**: Optimized for 8GB VPS deployment
- **Apple Silicon Optimized**: Native M-series processor support
- **No GPU Requirements**: External Claude Code API eliminates local model needs
- **Simplified Deployment**: Single binary with minimal dependencies

## Related Documentation

- **[Agent System Details](ARCHITECTURE_AGENTS.md)** - Detailed agent implementation
- **[Tool Building Workflows](ARCHITECTURE_WORKFLOWS.md)** - Agent collaboration patterns
- **[Deployment Guide](ARCHITECTURE_DEPLOYMENT.md)** - Infrastructure setup
