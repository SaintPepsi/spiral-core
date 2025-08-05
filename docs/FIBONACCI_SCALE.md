# Complexity/Risk Fibonacci Scale

This document defines the standard Fibonacci scale used throughout the Spiral Core system for estimating complexity and assessing risk levels.

## Complexity Rating Scale

| Scale | Level | Description | Characteristics |
|-------|-------|-------------|-----------------|
| **?** | **Unknown** | Requires investigation and research before estimation | • Unclear requirements<br>• Unknown technologies<br>• Missing dependencies<br>• Need spike/research phase |
| **1** | **Trivial** | Simple, well-understood tasks | • Clear requirements<br>• Familiar technology<br>• No dependencies<br>• Single person can complete |
| **2** | **Simple** | Straightforward with minor complications | • Well-defined scope<br>• Some minor unknowns<br>• Minimal dependencies<br>• Standard implementation |
| **3** | **Moderate** | Some complexity but manageable | • Multiple components involved<br>• Some research required<br>• Moderate dependencies<br>• May need collaboration |
| **5** | **Complex** | Significant effort with multiple moving parts | • Cross-system integration<br>• New technology/patterns<br>• Multiple dependencies<br>• Requires team coordination |
| **8** | **Very Complex** | High complexity requiring careful planning | • Major architectural changes<br>• Multiple teams involved<br>• Significant unknowns<br>• External dependencies |
| **13** | **Extremely Complex** | Massive undertaking requiring decomposition | • System-wide impact<br>• Multiple phases required<br>• High coordination needs<br>• Should be broken down |
| **∞** | **Epic/Initiative** | Too large to estimate - must be decomposed | • Requires breaking into smaller stories<br>• Strategic initiative level<br>• Multiple teams and quarters |

## Risk Level Scale

| Scale | Level | Description | Impact | Mitigation Strategy |
|-------|-------|-------------|---------|-------------------|
| **?** | **Unknown** | Risk level unclear - needs investigation | • Potential for any level of impact<br>• Requires risk assessment | • Conduct risk analysis<br>• Research similar projects<br>• Consult experts |
| **1** | **Low** | Minimal risk with negligible impact | • Minor inconvenience if issues arise<br>• Easy to recover from<br>• No business impact | • Standard monitoring<br>• Basic rollback plan |
| **2** | **Potential** | Some risk but manageable | • Possible minor delays<br>• Temporary workarounds available<br>• Limited scope of impact | • Contingency planning<br>• Regular check-ins |
| **3** | **Medium** | Moderate risk requiring attention | • Could cause project delays<br>• May impact related systems<br>• Requires active management | • Risk mitigation plan<br>• Alternative approaches<br>• Stakeholder communication |
| **5** | **Certain** | High probability of issues | • Will likely encounter problems<br>• Significant impact potential<br>• Needs proactive measures | • Detailed mitigation strategy<br>• Fallback options<br>• Regular review cycles |
| **8** | **High** | Serious risk with major implications | • Major project impact likely<br>• Could affect business operations<br>• May require external help | • Comprehensive risk plan<br>• Executive awareness<br>• Multiple backup strategies |
| **13** | **Nuclear** | Critical risk that could be catastrophic | • Project failure possible<br>• Significant business impact<br>• Potential system-wide issues | • Executive approval required<br>• Extensive planning<br>• Risk register management<br>• Consider project viability |
| **∞** | **Do Not Implement** | Unacceptable risk level | • **Reasoning for ∞ Rating:**<br>• Potential for irreversible damage<br>• Regulatory/legal violations<br>• Existential threat to business<br>• Technology not mature enough<br>• Cost exceeds any possible benefit<br>• Ethical concerns<br>• Security vulnerabilities too severe | • **Actions:**<br>• Document decision rationale<br>• Explore alternative solutions<br>• Wait for technology maturity<br>• Reassess in future cycles<br>• Consider third-party solutions |

## Usage Guidelines

### When to Use Each Scale

- **Complexity**: Estimate effort, time, and technical difficulty
- **Risk**: Assess probability and impact of potential issues

### Best Practices

1. **Start with ?** if you're unsure - investigate first
2. **Break down ∞ items** into smaller, estimable pieces
3. **Review regularly** - complexity/risk can change
4. **Use team consensus** for estimation accuracy
5. **Document assumptions** behind ratings

### Escalation Matrix

| Complexity | Risk | Action Required |
|------------|------|-----------------|
| Any | 13 or ∞ | Executive approval required |
| 13 or ∞ | Any | Must decompose before proceeding |
| 8+ | 8+ | Senior leadership review |
| ? | ? | Investigation phase mandatory |

## Example Usage Scenarios

### Scenario 1: New Feature Implementation
- Complexity: 5 (requires new API integration)
- Risk: 3 (might affect existing users)
- Action: Proceed with careful planning and testing

### Scenario 2: Database Migration
- Complexity: 8 (involves multiple systems)
- Risk: 8 (potential data loss)
- Action: Extensive planning, multiple environments, rollback strategy

### Scenario 3: Experimental AI Feature
- Complexity: ? (unclear technical requirements)
- Risk: ? (unknown user impact)
- Action: Research phase required before estimation

### Scenario 4: Critical Security Fix
- Complexity: 2 (straightforward patch)
- Risk: 13 (if not fixed, system vulnerable)
- Action: Emergency deployment with executive approval

### Scenario 5: Complete System Rewrite
- Complexity: ∞ (too large to estimate)
- Risk: ∞ (business continuity at risk)
- Action: Decompose into phases, consider alternatives

## Integration with Planning

When creating implementation plans:
1. Assign complexity rating to each task
2. Assess overall risk level for the update
3. Use escalation matrix to determine approval requirements
4. Adjust approach based on risk/complexity combination

## Relationship to Time

**Important**: These scales measure complexity and risk, NOT time.
- A complexity 1 task might take days if waiting for approvals
- A complexity 8 task might be done quickly with the right expertise
- Focus on the intrinsic difficulty and risk, not duration