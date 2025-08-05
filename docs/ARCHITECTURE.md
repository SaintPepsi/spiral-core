# Spiral Core Architecture

## System Overview

Spiral Core is a high-performance, memory-efficient AI agent orchestration system built in Rust. Agents collaborate through Claude Code integration to build tools and manage complex workflows, optimized for deployment on modest hardware (8GB VPS or Apple Silicon).

**Key Simplification**: Agents serve as intelligent orchestrators of Claude Code capabilities rather than managing local LLM inference, dramatically reducing complexity while maintaining sophisticated functionality.

## Core Architecture Components

### Technology Stack

- **Rust Backend**: Agent orchestration system that coordinates with Claude Code
- **Claude Code Integration**: Primary AI engine for code generation and analysis
- **Discord Bot Service**: Node.js/TypeScript service for human interaction
- **GitHub Integration**: Automated repository management and PR creation
- **Redis Message Queues**: Asynchronous communication between agents
- **PostgreSQL Database**: Agent state and task persistence

### System Benefits

| Category        | Benefits                                                         |
| --------------- | ---------------------------------------------------------------- |
| **Performance** | ~2GB memory footprint, 0.3-0.8s boot time, 6+ concurrent agents  |
| **Development** | Compile-time safety, type-safe agent interactions, memory safety |
| **Operational** | 8GB VPS ready, Apple Silicon optimized, no GPU required          |

## Agent System Design

### Agent Types and Responsibilities

1. **Software Developer Agent**

   - Transform ideas into working code through Claude Code orchestration
   - Language detection, architecture planning, code quality, documentation
   - Decision focus: Technical implementation choices

2. **Project Manager Agent**

   - Strategic oversight and coordination of complex multi-phase projects
   - Strategic analysis, resource planning, risk assessment, progress monitoring
   - Decision focus: What to build first, sequencing, pivoting

3. **Quality Assurance Agent**

   - Ensure reliability, security, and robustness of delivered solutions
   - Risk analysis, security review, testing strategy, performance evaluation
   - Decision focus: Risk levels, coverage requirements, security standards

4. **Decision Maker Agent**

   - Resolve conflicts and make final decisions when agents disagree
   - Trade-off analysis, priority scoring, conflict resolution
   - Decision focus: Meta-decisions when specialists disagree

5. **Creative Innovator Agent**

   - Explore alternative approaches and challenge conventional thinking
   - Alternative generation, assumption challenging, innovation catalyst
   - Decision focus: Exploring unconventional approaches

6. **Process Coach Agent**
   - Optimize team performance and improve coordination efficiency
   - Performance analysis, process improvement, communication enhancement
   - Decision focus: How agents coordinate, when to escalate

### Coordination Philosophy

#### Resource Sharing

- Dynamic allocation from shared Claude Code API pool
- Elastic scaling to current needs
- Transparent usage tracking
- Fair access prevention of monopolization

#### Communication Patterns

- Structured conversations with clear topics
- Inclusive participation from relevant agents
- Automatic moderation via message limits
- Human escalation for unresolved conflicts

## Why Rust for AI Agent Orchestration

### Resource Efficiency

- **Principle**: Maximize capability while minimizing resource consumption
- **Approach**: External intelligence via Claude Code API
- **Results**: ~2.1GB footprint vs 8GB+ for local models

### Compile-Time Safety

- **Principle**: Prevent bugs through type system rather than runtime checks
- **Approach**: Model agent states and communication as types
- **Results**: No coordination deadlocks, deterministic behavior

## Human Integration

### Discord-First Interaction

- Primary interface for human-agent communication
- Conversational mentions (@SpiralDev, @SpiralPM)
- Progress updates and approval requests in-channel
- Rich Discord features for complex coordination

### Approval-Based Tool Creation

- Agents identify and request new capabilities
- Comprehensive analysis provided for human review
- Transparent request process via Discord
- Human approval maintains system integrity

## Implementation Phases

### Phase 1: Foundation (Current)

- Discord Bot Integration with conversational mentions
- Developer Agent with language detection
- Claude Code Client integration
- Minimal HTTP API for agent communication

### Phase 2: Enhanced Coordination

- Project Manager Agent implementation
- GitHub Integration for automated PR creation
- Agent Status Monitoring and resource tracking

### Phase 3: Advanced Features

- QA Agent for code review and validation
- Enhanced Discord Commands
- Database Persistence for agent state

### Phase 4: Tool Building System

- Human Approval Workflows
- Custom Tool Creation
- Self-Improvement Mechanisms

## Deployment Architecture

### System Requirements

- **Memory**: ~2.1GB RAM
- **CPU**: 2+ cores recommended
- **Network**: Stable internet for Claude API calls
- **Storage**: Minimal (logs and state only)

### Container Deployment

```yaml
services:
  spiral-core:
    image: spiral-core:latest
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - DISCORD_TOKEN=${DISCORD_TOKEN}
    ports:
      - "3000:3000"
    resources:
      limits:
        memory: 2.5G
```

### Scaling Strategy

- Horizontal scaling via multiple agent instances
- Load balancing across Discord shards
- Redis for inter-agent communication
- PostgreSQL for shared state

## Security Considerations

### API Security

- Environment variable configuration
- API key rotation support
- Rate limiting and throttling
- Request validation and sanitization

### Discord Security

- User authorization via ID whitelist
- Command permission levels
- Audit logging for all actions
- Contextual denial messages

### Code Execution

- No direct code execution
- All code generation via Claude Code
- Human approval for system changes
- Sandboxed test environments

## Monitoring and Observability

### Metrics

- Agent response times
- Claude API usage tracking
- Discord command latency
- System resource utilization

### Logging

- Structured logging with tracing
- Debug, info, warn, error levels
- Correlation IDs for request tracking
- External log aggregation support

### Health Checks

- `/health` endpoint for liveness
- `/ready` endpoint for readiness
- Agent-specific health status
- Dependency health monitoring

## Future Enhancements

### Short Term

- Enhanced agent collaboration patterns
- Improved error recovery mechanisms
- Extended Discord command set
- Performance optimization

### Long Term

- Multi-tenant support
- Plugin architecture for custom agents
- Advanced learning mechanisms
- Cross-platform UI clients

## Related Documentation

For implementation details, see:

- [Development Guide](DEVELOPMENT.md) - Coding standards and practices
- [Setup Guide](SETUP.md) - Installation and configuration
- [API Reference](API.md) - HTTP endpoint documentation
- [Operations Guide](OPERATIONS.md) - Deployment and monitoring
