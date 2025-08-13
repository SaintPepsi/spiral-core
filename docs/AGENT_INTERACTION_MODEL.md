# Agent Interaction Model

## Overview

The Spiral Core agent system uses three distinct interaction patterns, each serving a specific purpose in the orchestration architecture.

## 1. Direct Execution Pattern (Developer Agent)

**Purpose**: Immediate task execution without planning overhead

**Workflow**:

```
User Request → Developer Agent → Direct Execution → Result
```

**Example**:

- User: `@SpiralDev fix the login bug in auth.rs`
- Agent: Directly analyzes code, implements fix, reports completion
- No consultation with other agents
- No planning phase

**When to Use**:

- Simple, well-defined tasks
- Bug fixes with clear scope
- Direct code generation requests

## 2. Orchestration Pattern (Project Manager Agent)

**Purpose**: Strategic planning through multi-agent consultation

**Workflow**:

```
User Request → PM Agent → Consult Multiple Agents → Synthesize Plan → Coordinate Execution
                    ↓
            [Developer Agent: "How would you architect this?"]
            [QA Agent: "What testing concerns exist?"]
            [Security Agent: "What security implications?"]
                    ↓
            Comprehensive Implementation Plan
```

**Example**:

- User: `@SpiralPM implement user authentication system`
- PM Agent:
  1. Asks Developer Agent for architecture approach
  2. Asks QA Agent for testing strategy
  3. Asks Security Agent for security considerations
  4. Synthesizes feedback into comprehensive plan
  5. Returns strategic implementation roadmap

**When to Use**:

- Complex features requiring multiple perspectives
- Architecture decisions needing consensus
- Risk assessment for critical changes

## 3. Autonomous Self-Update Pattern (Spiral Constellation)

**Purpose**: System self-improvement without human intervention

**Workflow**:

```
Discord Mention → Validation → Implementation → Two-Phase Pipeline → Auto-Commit
                      ↓               ↓                  ↓
                [Security Check] [Claude Code] [Phase 1: Quality]
                [Scope Limit]    [Execution]  [Phase 2: Compliance]
```

**Key Characteristics**:

- **NOT a conversational bot** - it's the self-update pipeline
- Triggered by Discord mentions but operates autonomously
- Uses 15+ specialized validation agents
- Automatic rollback on failure
- No human approval needed (within scope limits)

## Current Implementation Status

### ✅ Working

- **Developer Agent**: Fully functional direct execution
- **Self-Update System**: Complete autonomous pipeline

### 🚧 Partially Working

- **Project Manager Agent**: Implemented but doesn't consult other agents yet
  - Current: Generates plans internally
  - Needed: Multi-agent consultation mechanism

### ❌ Not Implemented

- **Multi-Agent Consultation**: PM can't query other agents
- **Agent Communication Protocol**: No inter-agent messaging
- **Consensus Building**: No mechanism for synthesizing agent feedback

## Technical Requirements for Full Implementation

### For Project Manager Orchestration

```rust
// CURRENT (incorrect - doesn't consult)
impl ProjectManagerAgent {
    async fn execute(&mut self, task: Task) -> Result<TaskResult> {
        let phases = self.generate_phases(&task); // Internal only
    }
}

// NEEDED (correct - multi-agent consultation with dynamic discovery)
impl ProjectManagerAgent {
    async fn execute(&mut self, task: Task) -> Result<TaskResult> {
        // Step 1: Discover available agents dynamically
        let available_agents = self.list_available_agents().await?;
        // Example: [Developer, QA, Security, CEO, Performance, etc.]
        
        // Step 2: Query relevant agents based on availability
        let mut consultations = Vec::new();
        
        // Always consult Developer for technical input
        if available_agents.contains(&AgentType::Developer) {
            consultations.push(
                self.consult_agent(AgentType::Developer,
                    "How would you architect this feature?").await?
            );
        }
        
        // Conditionally consult other agents if available
        if available_agents.contains(&AgentType::QA) {
            consultations.push(
                self.consult_agent(AgentType::QA,
                    "What testing strategy and risks exist?").await?
            );
        }
        
        if available_agents.contains(&AgentType::CEO) {
            consultations.push(
                self.consult_agent(AgentType::CEO,
                    "What are the business implications?").await?
            );
        }
        
        if available_agents.contains(&AgentType::Security) {
            consultations.push(
                self.consult_agent(AgentType::Security,
                    "What security considerations apply?").await?
            );
        }
        
        // Step 3: Synthesize all agent responses
        let plan = self.synthesize_plan(consultations);
        
        // Step 4: Return comprehensive strategy
        Ok(TaskResult::Success(plan))
    }
    
    async fn list_available_agents(&self) -> Result<Vec<AgentType>> {
        // Query agent registry for currently active agents
        self.agent_registry.list_active_agents().await
    }
}
```

### Inter-Agent Communication Protocol

Required components:

1. **Agent Registry**: Central registry for agent discovery
2. **Message Bus**: Async communication channel between agents
3. **Consultation API**: Structured query/response format
4. **Response Synthesis**: Logic to combine multiple agent inputs

## Usage Guidelines

### Choose Developer Agent When

- Task is clear and scoped
- Single domain expertise needed
- Speed is priority over planning
- Direct implementation requested

### Choose Project Manager When

- Task spans multiple domains
- Architecture decisions needed
- Risk assessment required
- Multiple implementation approaches exist

### Spiral Constellation Triggers On

- Any Discord mention with self-update intent
- Automatic improvement requests
- System optimization needs
- Bug fixes in core system

## Common Misconceptions

❌ **Misconception**: All agents work the same way
✅ **Reality**: Each agent type has distinct interaction pattern

❌ **Misconception**: PM Agent is just another task executor  
✅ **Reality**: PM orchestrates other agents, doesn't execute directly

❌ **Misconception**: Constellation Bot is a chatbot
✅ **Reality**: It's an autonomous self-update pipeline

## Next Steps

1. **Implement Agent Communication Protocol**
2. **Add consultation mechanism to PM Agent**
3. **Create agent response synthesis logic**
4. **Test multi-agent orchestration patterns**
5. \*\*Document agent consultation API
