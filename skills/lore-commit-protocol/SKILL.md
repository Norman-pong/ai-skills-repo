---
name: lore-commit-protocol
description: Write git commit messages following the Lore Commit Protocol. Use whenever the user asks to commit code, write a commit message, create a git commit, or mentions committing changes. Also trigger when the user says "commit this", "make a commit", "commit message", "准备提交", or any commit-related workflow.
---

# Lore Commit Protocol

Write git commit messages using the Lore protocol format.

## Core principle

The diffs already show *what* changed. The commit message exists to capture *why* the change was made and what trade-offs shaped it. This is critical for future readers who have no context about the decision — that includes your future self, code reviewers, and agents working on the codebase six months from now.

## Format

```
<intent line: why the change was made, not what changed>

<body: narrative context — constraints, approach rationale>

Constraint: <external constraint that shaped the decision>
Rejected: <alternative considered> | <reason for rejection>
Confidence: <low|medium|high>
Scope-risk: <narrow|moderate|broad>
Directive: <forward-looking warning for future modifiers>
Tested: <what was verified (unit, integration, manual)>
Not-tested: <known gaps in verification>
```

## Rules

### 1. First line describes intent, not content

The subject line answers "why did we do this?" — not "what does this diff contain?" The diff already answers the latter. A good intent line lets someone scanning `git log --oneline` understand the motivation without opening the commit.

Good:
- "Prevent silent session drops during long-running operations"
- "Fix race condition in cache invalidation on profile update"

Avoid:
- "Add error handling to auth.ts"
- "Refactor user controllers"

### 2. Body explains context and trade-offs

After a blank line, write the narrative. Include:
- What constraints forced the decision (external limits, policy, API behavior, deadlines)
- Why this approach was chosen over alternatives
- Any non-obvious side effects or assumptions

Use Chinese or English for the body — the repository convention dictates this. When the repo uses Chinese, write the body in Chinese. When in doubt, match the language used in the project's CLAUDE.md or existing commit history.

### 3. Trailers capture structured metadata

After the body, add a blank line, then append trailers. Trailers use git-native `Key: Value` format — this means `git interpret-trailers` can parse them without custom tooling. Only include trailers that add value for this specific commit. Never add trailers that would be boilerplate.

### 4. Trailer usage guide

**Constraint:** — Use when an external force shaped the decision. This is the most important trailer because it captures information invisible in the code. Examples: an API limitation, a hard deadline, a policy requirement, a dependency version lock, a team convention you had to work around.

**Rejected:** — Use when you genuinely considered and discarded an alternative. This prevents future readers (including agents) from wasting time re-exploring the same dead end. Format: `Rejected: <approach> | <reason>` The reason is the important part — without it, the rejection isn't actionable. Only include alternatives you actually evaluated seriously.

**Confidence:** — Your assessment of how confident you are. `high` = well-tested change with a familiar pattern. `medium` = reasonable confidence but some uncertainty (e.g., a new dependency or less-explored code path). `low` = the change is experimental or rushed, and the reviewer should scrutinize it carefully.

**Scope-risk:** — How broadly the change could affect the system. `narrow` = localized change with clear blast radius. `moderate` = touches multiple modules or shared utilities. `broad` = changes core infrastructure, shared types, or build configuration.

**Directive:** — A message to whoever modifies this code next. Use for warnings like "don't change X without checking Y" or "this error handling is intentionally broad — verify upstream behavior before narrowing." A good directive prevents future regressions by communicating knowledge that isn't visible in the code.

**Tested:** — What verification was actually performed. Be specific: "unit tests pass", "manual test of login flow with expired token", "integration test of the payment webhook handler". This helps reviewers judge whether the testing was adequate for the change.

**Not-tested:** — Known gaps in verification. This is more valuable than pretending everything is covered. Examples: "Auth service cold-start > 500ms behavior", "Edge case when file is exactly 0 bytes", "Mobile Safari rendering of the new modal". Being honest about gaps helps prioritize QA effort and prevents false confidence.

**Reversibility:** — How cleanly the change can be undone. `clean` = revert is straightforward. `messy` = revert would require additional migrations or careful state management. `irreversible` = cannot be undone (e.g., data deletion, schema changes that destroy information).

**Related:** — Links to related commits, issues, tickets, or decision documents. Format: `Related: #1234` or `Related: commit abc123`.

### 5. Chinese trailer commentary

When the repo uses Chinese, add a brief Chinese explanation in the trailer value to avoid ambiguity for future readers. This is particularly important for Constraint and Directive trailers where the reasoning chain might not be obvious from the English keywords alone.

## Workflow

When asked to create a commit:

1. **Review the changes** — run `git diff --staged` and `git log --oneline -5` to understand what changed and how it fits the project's history.
2. **Identify the intent** — ask yourself: why is this change happening? What problem does it solve?
3. **Note constraints and trade-offs** — were there external limits? Did you reject any alternatives?
4. **Draft the message** — write the intent line first, then the body, then relevant trailers.
5. **Present the draft to the user** — show the full commit message and let the user confirm before running `git commit`. Never commit without explicit user approval.

When presenting a draft, show it in full so the user can review every part. Do not abbreviate or summarize the trailers.

## Example

Here is a complete Lore protocol commit message:

```
Prevent silent session drops during long-running operations

The auth service returns inconsistent status codes on token
expiry, so the interceptor catches all 4xx responses and
triggers an inline refresh.

Constraint: Auth service does not support token introspection（认证服务不支持令牌内省）
Constraint: Must not add latency to non-expired-token paths（不得为未过期令牌引入额外延迟）
Rejected: Extend token TTL to 24h | security policy violation（违反安全策略）
Rejected: Background refresh on timer | race condition with concurrent requests（并发竞态条件）
Confidence: high
Scope-risk: narrow
Directive: Error handling is intentionally broad (all 4xx) — do not narrow without verifying upstream behavior（错误处理有意覆盖所有4xx响应，在验证上游行为前不要缩小范围）
Tested: Single expired token refresh (unit)
Not-tested: Auth service cold-start > 500ms behavior（认证服务冷启动大于500ms的行为）
```
