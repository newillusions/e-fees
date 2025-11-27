# Sub-Agent Routing Guide

> **Purpose**: Comprehensive decision framework for delegating tasks to the appropriate specialist agent in the e-fees project. Use this guide to determine which sub-agent to invoke based on task characteristics, symptoms, and context.

## Quick Reference Decision Tree

```
START HERE: What is the primary nature of the request?

├─ MCP/Socket/Testing Framework Issues
│  └─→ MCP SPECIALIST
│
├─ Desktop App/Window/IPC/Tauri Core
│  └─→ TAURI DEVELOPER
│
├─ Test Scenarios/E2E Workflows/Test Data
│  └─→ TESTING SPECIALIST
│
├─ Database/Queries/Schema/Performance
│  └─→ DATABASE SPECIALIST
│
├─ Code Quality/Architecture/Review
│  └─→ CODE REVIEWER
│
└─ UI/UX/Components/Styling/Scaling
   └─→ FRONTEND SPECIALIST
```

---

## Detailed Routing Matrix

### By Problem Domain

| Domain | Primary Specialist | Secondary/Handoff |
|--------|-------------------|-------------------|
| Socket communication failing | MCP Specialist | Tauri Developer (if IPC related) |
| Application won't launch | Tauri Developer | MCP Specialist (if MCP-related) |
| Tests flaking/failing | Testing Specialist | MCP Specialist (framework), Database Specialist (data) |
| Slow queries | Database Specialist | Code Reviewer (optimization patterns) |
| Code smells | Code Reviewer | Domain specialist for specific fixes |
| UI not scaling properly | Frontend Specialist | Tauri Developer (if window-related) |
| Data corruption | Database Specialist | Testing Specialist (test data safety) |
| Build errors | Tauri Developer | Code Reviewer (if architecture issue) |
| CSS not applying | Frontend Specialist | Tauri Developer (if CSP/config) |
| Invalid Thing references | Database Specialist | Testing Specialist (if in tests) |

---

## By Symptom Recognition

### "The app won't start/crashes on launch"
**Primary**: Tauri Developer  
**Reasoning**: Core application lifecycle, window management, system integration  
**When to escalate**: If crash is MCP-socket related → MCP Specialist

### "Tests are failing intermittently"
**Primary**: Testing Specialist  
**Reasoning**: Test stability, wait strategies, assertion patterns  
**When to escalate**: If MCP commands timing out → MCP Specialist  
**When to escalate**: If data state issues → Database Specialist

### "Socket not found" or "Connection refused"
**Primary**: MCP Specialist  
**Reasoning**: Socket path issues, server lifecycle, protocol debugging  
**When to escalate**: If Tauri plugin not loading → Tauri Developer

### "Text is cut off at 125%/150% scale"
**Primary**: Frontend Specialist  
**Reasoning**: DPI scaling, Tailwind fixed pixels, responsive design  
**When to escalate**: Never - this is purely frontend domain

### "Query returns null/unexpected data"
**Primary**: Database Specialist  
**Reasoning**: SurrealQL syntax, Thing ID resolution, schema relationships  
**When to escalate**: If test data contamination → Testing Specialist

### "Code works but feels messy"
**Primary**: Code Reviewer  
**Reasoning**: Architecture patterns, DRY violations, maintainability  
**When to escalate**: To domain specialist for implementing recommended changes

---

## By File/Directory Context

When working in specific project areas, prefer these specialists:

### `/src-tauri/src/`
**Primary**: Tauri Developer  
**Exceptions**:
- `mcp/` directory → MCP Specialist
- Database logic → Database Specialist
- General code quality → Code Reviewer

### `/src-tauri/src/mcp/`
**Primary**: MCP Specialist  
**Always**: This is MCP Specialist's exclusive domain

### `/src/lib/components/`
**Primary**: Frontend Specialist  
**Exceptions**:
- Testing utilities → Testing Specialist
- Data models → Database Specialist

### `/src/lib/db/`
**Primary**: Database Specialist  
**Always**: Schema, queries, migrations

### `/e2e-tests/`
**Primary**: Testing Specialist  
**Exceptions**:
- MCP framework issues → MCP Specialist
- Test data patterns → Database Specialist

### `/src/lib/styles/` or `.css` files
**Primary**: Frontend Specialist  
**Always**: Styling, Tailwind, responsive design

---

## By Task Type Categorization

### Development Tasks

#### "Add a new feature"
1. **Code Reviewer** - Assess design approach
2. **Domain Specialist** - Implement based on area:
   - UI feature → Frontend Specialist
   - Backend logic → Tauri Developer
   - Database schema → Database Specialist
3. **Testing Specialist** - Create E2E tests
4. **Code Reviewer** - Final review

#### "Fix a bug"
1. **Identify symptom** (use symptom recognition above)
2. **Primary specialist** - Diagnose and fix
3. **Testing Specialist** - Add regression test
4. **Code Reviewer** - Verify fix quality

#### "Optimize performance"
1. **Code Reviewer** - Profile and identify bottleneck
2. **Domain Specialist**:
   - Query performance → Database Specialist
   - Render performance → Frontend Specialist
   - IPC overhead → Tauri Developer
3. **Testing Specialist** - Verify no regression

#### "Refactor code"
1. **Code Reviewer** - Design refactoring strategy
2. **Domain Specialist** - Execute refactoring
3. **Testing Specialist** - Ensure tests still pass
4. **Code Reviewer** - Final review

### Debugging Tasks

#### "Something is broken, not sure what"
**Sequential diagnostic approach**:
1. **Start with symptom**: Use symptom recognition table
2. **If unclear**: Code Reviewer triages
3. **Delegate to specialist** based on triage findings
4. **Handoff chain** as needed

#### "Feature works locally but not in tests"
**Primary**: Testing Specialist  
**Handoff to**: MCP Specialist if socket issues, Database Specialist if data state issues

#### "Tests pass but UI is broken"
**Sequential approach**:
1. **Frontend Specialist** - Check UI implementation
2. **Testing Specialist** - Review test coverage gaps
3. **Code Reviewer** - Assess overall quality

---

## Multi-Specialist Scenarios

Some tasks naturally require coordination between specialists. Here's how to orchestrate:

### Scenario: New Entity Type (e.g., "TimeEntry")

**Phase 1 - Schema Design**
- **Database Specialist**: Define schema, relationships, indices
- **Code Reviewer**: Review schema design for normalization

**Phase 2 - Backend Implementation**
- **Tauri Developer**: Create IPC commands
- **Database Specialist**: Write queries and mutations

**Phase 3 - Frontend Implementation**
- **Frontend Specialist**: Build UI components
- **Code Reviewer**: Review component architecture

**Phase 4 - Testing**
- **Testing Specialist**: Write E2E test scenarios
- **Database Specialist**: Verify test data cleanup

**Phase 5 - Quality Assurance**
- **Code Reviewer**: Final comprehensive review
- **Testing Specialist**: Run full test suite

### Scenario: DPI Scaling Bug Across App

**Phase 1 - Assessment**
- **Frontend Specialist**: Audit all fixed pixel usages
- **Code Reviewer**: Identify architectural patterns causing issues

**Phase 2 - Fix Strategy**
- **Frontend Specialist**: Lead conversion to relative units
- **Tauri Developer**: Verify window sizing configs don't conflict

**Phase 3 - Validation**
- **Testing Specialist**: Create scaling test checklist
- **Frontend Specialist**: Manual testing at each scale

### Scenario: MCP Integration Enhancement

**Phase 1 - Protocol Design**
- **MCP Specialist**: Design new MCP capabilities
- **Tauri Developer**: Assess Tauri plugin requirements

**Phase 2 - Implementation**
- **MCP Specialist**: Implement server-side protocol
- **Tauri Developer**: Update Rust plugin integration

**Phase 3 - Testing Integration**
- **Testing Specialist**: Adopt new MCP capabilities in tests
- **MCP Specialist**: Debug any protocol issues

**Phase 4 - Documentation**
- **Code Reviewer**: Ensure documentation complete
- **MCP Specialist**: Update MCP architecture docs

---

## Escalation Patterns

### When a Specialist Should Escalate

Each specialist has clear boundaries. Here's when they should hand off:

#### MCP Specialist escalates when:
- Issue is in Tauri core, not MCP layer → Tauri Developer
- Test logic problem, not MCP framework → Testing Specialist
- Database query issues in test → Database Specialist

#### Tauri Developer escalates when:
- MCP socket/protocol issues → MCP Specialist
- UI implementation details → Frontend Specialist
- Database design questions → Database Specialist
- Architecture concerns → Code Reviewer

#### Testing Specialist escalates when:
- MCP commands not working → MCP Specialist
- Test data schema questions → Database Specialist
- UI interaction patterns → Frontend Specialist
- Test architecture concerns → Code Reviewer

#### Database Specialist escalates when:
- Schema needs fundamental redesign → Code Reviewer
- Test data safety patterns → Testing Specialist
- UI display of data → Frontend Specialist

#### Code Reviewer escalates when:
- Needs implementation expertise → Domain specialist
- Can advise but not execute → Appropriate specialist

#### Frontend Specialist escalates when:
- Window/display config at OS level → Tauri Developer
- Data fetching strategy → Database Specialist
- Test coverage for UI → Testing Specialist
- Architecture concerns → Code Reviewer

### Escalation Protocol

```
1. Specialist identifies issue outside their domain
2. Clearly state: "This requires [Specialist Name] expertise because [reason]"
3. Summarize findings so far
4. Provide context for next specialist
5. Hand off explicitly: "Delegating to [Specialist]..."
```

---

## Anti-Patterns: Wrong Specialist for the Job

### ❌ Don't Route MCP Issues to Frontend Specialist
**Why**: Frontend Specialist doesn't have MCP protocol knowledge  
**Symptom**: "Socket connection" errors being treated as UI bugs  
**Correct**: MCP Specialist handles all socket/protocol issues

### ❌ Don't Route UI Scaling to Tauri Developer  
**Why**: This is CSS/component-level, not window management  
**Symptom**: Asking Tauri Developer about Tailwind classes  
**Correct**: Frontend Specialist owns all styling and DPI scaling

### ❌ Don't Route Test Flakiness to Code Reviewer
**Why**: Code Reviewer advises, doesn't debug test infrastructure  
**Symptom**: Asking Code Reviewer to fix timing issues  
**Correct**: Testing Specialist for test stability, MCP Specialist if framework-related

### ❌ Don't Route Schema Changes to Testing Specialist
**Why**: Testing Specialist uses schema, doesn't design it  
**Symptom**: Asking Testing Specialist about foreign key relationships  
**Correct**: Database Specialist owns schema design

### ❌ Don't Route Architecture Decisions to Implementation Specialists
**Why**: Implementation specialists execute, Code Reviewer designs  
**Symptom**: Asking Frontend Specialist about overall app structure  
**Correct**: Code Reviewer for architecture, then specialists implement

---

## Special Situations

### "DELETE ME" Pattern Violations
**Primary**: Testing Specialist  
**Why**: Owns test data safety protocol  
**Action**: Immediate cleanup, audit test creation patterns

### Build/CI Pipeline Issues
**Primary**: Tauri Developer  
**Why**: Owns build configuration and deployment  
**Handoff**: Code Reviewer if architectural problem

### Performance Regression
**Phase 1**: Code Reviewer profiles and identifies bottleneck  
**Phase 2**: Domain specialist optimizes:
- Database → Database Specialist
- Rendering → Frontend Specialist  
- IPC → Tauri Developer
- Test execution → Testing Specialist + MCP Specialist

### Security Concerns
**Primary**: Code Reviewer  
**Why**: Cross-cutting concern requiring architectural view  
**Delegates**: To specialists for implementing fixes

### Accessibility Issues
**Primary**: Frontend Specialist  
**Why**: ARIA, keyboard nav, screen reader support  
**Consult**: Tauri Developer if platform integration needed

---

## Routing Decision Checklist

Before routing to a specialist, verify:

- [ ] I've identified the primary symptom/problem domain
- [ ] I've checked the symptom recognition table
- [ ] I've considered which files/directories are affected
- [ ] I understand if this requires multi-specialist coordination
- [ ] I know the escalation path if this specialist can't resolve it
- [ ] I'm not falling into an anti-pattern

---

## Context Handoff Template

When delegating to a specialist, provide:

```
**Routing to: [Specialist Name]**

**Context**: [Brief description of the task/problem]

**Symptoms/Evidence**:
- [Key symptoms or error messages]
- [Files/components affected]

**What's Been Tried**: [Any prior attempts or diagnostics]

**Constraints**: [Any known limitations or requirements]

**Expected Outcome**: [What success looks like]

**Handoff Notes**: [Any domain-specific context the specialist needs]
```

---

## Routing Examples

### Example 1: "Users report text being cut off in invoices"

**Analysis**:
- Symptom: Display/rendering issue
- Affected: UI components
- Platform context: Likely DPI scaling

**Route to**: Frontend Specialist

**Reasoning**: This is a CSS/component layout issue, specifically related to the known Tailwind scaling problems at 125%/150%/175% display scales that Frontend Specialist specializes in.

---

### Example 2: "E2E tests timing out when creating contacts"

**Analysis**:
- Symptom: Test execution issue
- Affected: Test suite
- Error type: Timeout

**Route to**: Testing Specialist

**Reasoning**: Test stability and timing issues are Testing Specialist domain. They'll determine if it's a wait strategy issue or needs escalation to MCP Specialist for framework problems.

**Potential Escalation**: If Testing Specialist finds MCP commands are actually timing out (not just test waits), escalate to MCP Specialist.

---

### Example 3: "Socket file not found error on startup"

**Analysis**:
- Symptom: Socket connection
- Affected: MCP integration
- Error type: File system/socket

**Route to**: MCP Specialist

**Reasoning**: Anything involving socket paths, MCP server lifecycle, or protocol communication is MCP Specialist's exclusive domain.

---

### Example 4: "Need to add 'archived' field to projects"

**Analysis**:
- Request: Schema modification
- Affected: Database structure
- Scope: Data model

**Multi-specialist approach**:
1. **Database Specialist**: Design schema change, migration strategy
2. **Tauri Developer**: Update IPC commands for new field
3. **Frontend Specialist**: Add UI controls for archiving
4. **Testing Specialist**: Create E2E test scenarios
5. **Code Reviewer**: Final review of implementation

---

### Example 5: "Code works but is hard to maintain"

**Analysis**:
- Concern: Code quality
- Type: Architecture/maintainability
- Not broken: Functional issue

**Route to**: Code Reviewer

**Reasoning**: Code quality, architecture, and maintainability concerns are Code Reviewer's domain. They'll assess and recommend refactoring strategies, then delegate implementation to appropriate specialists.

---

## Quick Reference: Keyword to Specialist Mapping

| Keywords in Request | Primary Specialist |
|---------------------|-------------------|
| socket, MCP, protocol, server, unix | MCP Specialist |
| window, launch, IPC, command, Tauri plugin | Tauri Developer |
| test, E2E, flaky, "DELETE ME", scenario | Testing Specialist |
| query, schema, Thing, database, RELATE | Database Specialist |
| refactor, architecture, design, pattern | Code Reviewer |
| UI, CSS, Tailwind, component, styling, DPI, scaling | Frontend Specialist |

---

## Conclusion

Effective routing requires:

1. **Clear symptom identification**: Use the symptom recognition table
2. **Domain understanding**: Know which specialist owns what
3. **Escalation awareness**: Recognize when to hand off
4. **Multi-specialist coordination**: Orchestrate complex tasks
5. **Anti-pattern avoidance**: Don't route to wrong specialist

**When in doubt**: Start with Code Reviewer for triage, then route to appropriate specialist based on their assessment.

---

*This routing guide is a living document. Update as new patterns emerge or specialist boundaries evolve.*
