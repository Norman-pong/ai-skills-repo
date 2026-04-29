---
name: github-pr-writer
description: Draft, normalize, review, or maintain GitHub pull request descriptions and PR draft documents. Use when Codex is asked to write a PR body, create a future PR plan, convert a plan into GitHub PR format, fill a repository pull_request_template.md, document scope/risks/tests for a PR, or check that a PR description follows GitHub-style review expectations.
---

# GitHub PR Writer

Use this skill to produce reviewable GitHub PR documents: concrete scope, clear boundaries, honest tests, linked evidence, and no wiki drift.

## Workflow

1. Discover the local contract.
   - Prefer the repository's `.github/pull_request_template.md` when it exists.
   - Also scan nearby `AGENTS.md`, `CONTRIBUTING.md`, release docs, or issue templates when they govern PR wording.
   - If no repo template exists, use [github-pr-template.md](references/github-pr-template.md).

2. Choose the artifact.
   - For an actual PR body, write the PR description directly or into the path the user requests.
   - For a future-stage or planned PR, create a draft under `.github/pr-drafts/<slug>.md` unless the repo has another convention.
   - Do not put one-off PR text into long-lived wiki pages. Link wiki/spec/source docs from the PR instead.

3. Draft from evidence.
   - Separate shipped behavior from planned, deferred, unverified, or excluded work.
   - Link relevant source docs, issues, commits, code paths, test logs, screenshots, or release gates.
   - Keep unchecked checklist items unchecked; never claim a command, browser test, or review ran unless it did.
   - Check only the change types the PR actually changes. A test plan does not make the PR a `Tests` change; a release risk note does not make it a `Build / release / tooling` change.

4. Apply boundaries and anti-examples.
   - Read [boundaries-tests-antipatterns.md](references/boundaries-tests-antipatterns.md) when the PR is non-trivial, future-facing, release-related, security-sensitive, or user asks for risk/testing guidance.
   - Redact secrets, tokens, private IPs, internal hostnames, customer names, and credentials unless the user explicitly requires them in a private repo artifact.

5. Validate before finishing.
   - If a PR document file was created or edited, run `python3 <skill>/scripts/check_pr_doc.py <file>`.
   - Run markdown relative-link checks for local links when practical.
   - In a git repo, run `git diff --check`.
   - For code changes, run the project tests implied by the PR template or explain unrun tests under `Not tested` / unchecked items.

## Output Rules

- Match the repository language and tone. If unknown, write concise professional Chinese when the surrounding repo uses Chinese; otherwise use English.
- Use GitHub Markdown and stable relative links.
- Keep the PR body review-oriented: why, what changed, risk, verification, screenshots, docs, and remaining gaps.
- Prefer a scoped PR draft over a giant plan. If the scope is too broad, split into multiple future PR drafts.
- When the user asks for a "standard GitHub PR document", preserve GitHub PR sections rather than inventing wiki-like taxonomy.
