# Development Workflow

*How features go from idea to shipped code in intrada.*

This document covers the end-to-end workflow, tooling setup, and AI agent
configuration used for development. It assumes you're using
[Claude Code](https://claude.ai/code) as your development environment.

---

## Overview

intrada uses a **spec-driven development** workflow powered by
[SpecKit](https://pypi.org/project/specify-cli/). Every feature follows a
structured pipeline with formal artifacts at each stage and human review gates
between them. Three custom AI subagents provide quality checks at key points.

```
Idea
 │
 ▼
/speckit-specify ──→ spec.md, requirements.md
 │
 ▼
spec-reviewer agent ──→ gap analysis, ambiguity check
 │
 ▼
(human review gate)
 │
 ▼
Design in Pencil ──→ frames in design/intrada.pen (if UI work)
 │
 ▼
/speckit-plan ──→ research.md, data-model.md, contracts/, quickstart.md
 │
 ▼
(human review gate)
 │
 ▼
/speckit-tasks ──→ tasks.md (dependency-ordered, phase-grouped)
 │
 ▼
/speckit-analyze ──→ cross-artifact consistency report (read-only)
/speckit-checklist ──→ domain checklists (UX, API, security, etc.)
 │
 ▼
(human review gate)
 │
 ▼
/speckit-implement ──→ executes tasks in order, marks complete
 │
 ▼
qa-validator agent ──→ tests, acceptance criteria verification
ux-auditor agent ──→ design system compliance (if UI work)
 │
 ▼
PR → review → merge
```

---

## Prerequisites

### Tools

| Tool | Install | Purpose |
|------|---------|---------|
| Rust stable | [rustup.rs](https://rustup.rs) | Core language |
| [just](https://github.com/casey/just) | `brew install just` | Task runner |
| [Trunk](https://trunkrs.dev) | `cargo install trunk` | WASM dev server |
| [uv](https://github.com/astral-sh/uv) | `brew install uv` | Python tool manager |
| [SpecKit](https://pypi.org/project/specify-cli/) | `uv tool install specify-cli` | Spec-driven workflow |
| Xcode 16+ | Mac App Store | iOS builds |

### Claude Code setup

The project expects these MCP servers (install via `claude mcp add`):

```bash
# Real-time library documentation
claude mcp add context7 -- npx -y @upstash/context7-mcp@latest

# Browser automation for E2E verification
claude mcp add playwright -- npx -y @playwright/mcp@latest

# GitHub integration (needs GITHUB_PERSONAL_ACCESS_TOKEN env var)
claude mcp add github -- npx -y @modelcontextprotocol/server-github
```

The project also uses the Pencil MCP (design tool) and several Claude Code
plugins configured globally: Atlassian, GitHub, Slack, Figma.

---

## SpecKit Stages

### 1. Specify (`/speckit-specify`)

Converts a natural-language feature description into a formal business spec.

- Creates a feature branch (auto-numbered)
- Produces `specs/{number}-{slug}/spec.md` (user-focused, no technical details)
- Produces `requirements.md` (quality checklist for the requirements themselves)
- Asks up to 3 clarification questions

**Human gate**: Review the spec. Run `/speckit-clarify` if requirements are
underspecified.

**Quality check**: The `spec-reviewer` agent automatically reviews for gaps,
ambiguity, and missing acceptance criteria.

### 2. Design in Pencil (if UI work)

After specifying and before planning, mock new views or significant UI changes
in `design/intrada.pen`.

- Desktop (1440px) and mobile (375px) frames required
- Reuse existing design system components
- Reference Pencil variables for colours (not raw hex)

See [CLAUDE.md](../CLAUDE.md) for full Pencil design rules.

### 3. Plan (`/speckit-plan`)

Transforms the spec into technical architecture decisions.

- Produces `research.md` (technical unknowns resolved via research)
- Produces `data-model.md` (entities, relationships, validation rules)
- Produces `contracts/` directory (API specs, interface definitions)
- Produces `quickstart.md` (integration test scenarios)

**Human gate**: Review architecture decisions, API contracts, data model.

### 4. Tasks (`/speckit-tasks`)

Breaks the plan into executable, dependency-ordered tasks.

- Produces `tasks.md` with strict checklist format
- Organised by phase: Setup, Foundational, User Stories (by priority), Polish
- Marks parallelisable tasks with `[P]`
- Typically produces 30-100 granular tasks

### 5. Quality gates

Two quality gates between tasks and implementation:

**`/speckit-analyze`** — Non-destructive cross-artifact consistency check.
Reads spec.md, plan.md, and tasks.md. Detects duplications, ambiguities, gaps,
and unmapped requirements. Produces a report but does NOT modify files.

**`/speckit-checklist`** — Domain-specific requirement quality validation.
Generates checklists for UX, API design, security, performance, etc. Tests
requirement *quality* (clarity, measurability), not implementation.

**Human gate**: Review findings, decide if issues are blocking.

### 6. Implement (`/speckit-implement`)

Executes all tasks in dependency order.

- Verifies all checklists are complete (blocks if incomplete)
- Runs tasks sequentially, parallelisable tasks together
- Marks each task `[X]` as complete
- Halts on failures in sequential tasks

### 7. Validation

After implementation, two agents verify the work:

**`qa-validator`** — Runs the full test suite (`cargo test`, `cargo clippy`,
`just ios-swift-check`, Playwright E2E), then verifies each acceptance criterion
from the spec against the actual implementation. Produces a PASS/FAIL verdict.

**`ux-auditor`** (if UI work) — Checks design token compliance, component reuse,
cross-platform parity (web vs iOS), and iOS UX rules. Uses the Pencil MCP to
compare against the design file.

---

## AI Subagents

Three custom subagents are defined in `.claude/agents/`. Claude auto-delegates
to them when it recognises a matching task, or you can invoke them explicitly.

### spec-reviewer

| Property | Value |
|----------|-------|
| Model | Sonnet |
| Tools | Read-only + web search |
| Trigger | After `/speckit-specify`, or "review this spec" |
| Purpose | Find gaps, ambiguity, missing acceptance criteria |

Example: `Use the spec-reviewer to review the practice intentions spec`

### qa-validator

| Property | Value |
|----------|-------|
| Model | Sonnet |
| Tools | Read-only + Bash (for running tests) |
| Trigger | After implementation, or "validate/QA this feature" |
| Purpose | Run tests, verify acceptance criteria, check regressions |

Example: `QA the session builder against its spec`

### ux-auditor

| Property | Value |
|----------|-------|
| Model | Haiku (fast, cheap) |
| Tools | Read-only + Pencil MCP |
| Trigger | After UI work, or "audit the UI" |
| Purpose | Design token compliance, component reuse, cross-platform parity |

Example: `Audit the analytics dashboard for design system compliance`

### Managing agents

- List all agents: `/agents` in Claude Code
- Create new agents: `/agents` then "Create new agent"
- Edit agent files directly: `.claude/agents/{name}.md`
- Agents are project-scoped and checked into version control

---

## Artifacts Reference

A completed feature produces this directory structure:

```
specs/{number}-{slug}/
  spec.md              # Business requirements (user-focused)
  requirements.md      # Requirement quality checklist
  research.md          # Technical decisions and research
  data-model.md        # Entity definitions and relationships
  contracts/           # API specs, interface definitions
  quickstart.md        # Integration test scenarios
  tasks.md             # Dependency-ordered task checklist
  checklists/          # Domain-specific quality checklists
```

---

## Quick Reference

| I want to... | Command |
|--------------|---------|
| Start a new feature | `/speckit-specify` |
| Get spec reviewed | "Use the spec-reviewer agent" (auto or explicit) |
| Ask clarifying questions | `/speckit-clarify` |
| Create technical plan | `/speckit-plan` |
| Generate tasks | `/speckit-tasks` |
| Check consistency | `/speckit-analyze` |
| Generate checklists | `/speckit-checklist` |
| Implement the feature | `/speckit-implement` |
| QA the implementation | "Use the qa-validator agent" (auto or explicit) |
| Audit UI consistency | "Use the ux-auditor agent" (auto or explicit) |
| Run all tests | `just check` |
| Build for iOS | `just ios` |
| Quick Swift check | `just ios-swift-check` |
| Start dev servers | `just dev` |
| Seed sample data | `just seed` |
