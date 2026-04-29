---
name: llm-wiki
description: Use this skill when organizing, refactoring, ingesting, querying, or linting project documentation as an LLM-friendly wiki. Apply it to docs, README files, architecture notes, runbooks, API references, release notes, and agent-facing knowledge bases when the goal is a concise index, clear cross-links, single sources of truth, or durable documentation governance.
---

# LLM Wiki

Use this skill to turn scattered project documents into a durable wiki layer for humans and agents. The model is inspired by the LLM Wiki pattern: preserve raw sources, maintain curated pages, keep a compact index, and record how knowledge changes over time.

## Core Model

- `sources`: raw truth such as code, tests, existing docs, commits, logs, external references, and user decisions. Do not rewrite sources just to make the wiki tidy.
- `wiki`: curated Markdown pages that explain stable project knowledge.
- `schema`: the local rules for page categories, ownership, linking, freshness, and validation.
- `index`: the main entry page. It should route readers and agents to the right page, not duplicate every detail.
- `log`: a short maintenance record for important ingests, refactors, link changes, and known stale areas.

## Workflow

1. Inventory docs and sources.
   - List current docs, README files, generated docs, and likely entrypoints.
   - Identify the audiences and common tasks: onboarding, API use, operations, architecture, release, troubleshooting, migration.
   - Mark generated outputs or vendored docs as non-source unless the project explicitly treats them as maintained docs.

2. Classify pages.
   - `index`: navigation and current top-level state.
   - `getting-started`: first successful use path.
   - `reference`: API, CLI, schema, config, and package contracts.
   - `architecture`: boundaries, data flow, design decisions.
   - `operations`: build, test, deploy, release, benchmark, troubleshooting.
   - `integration`: migration guides and third-party compatibility.
   - `history`: plans, snapshots, logs, status reports.

3. Choose single sources of truth.
   - Put each fact in one authoritative page.
   - Replace repeated long explanations with links.
   - Keep the index brief: current conclusion, task routes, key warnings, and links.
   - When two pages disagree, resolve from sources first; if unresolved, record uncertainty instead of smoothing it over.

4. Refactor safely.
   - Prefer conservative edits first: compress, link, and clarify before moving files.
   - Avoid renaming or moving pages unless the user explicitly approves the migration.
   - Keep relative links valid from both source docs and packaged/generated copies when those docs are distributed.
   - Do not edit generated output directories as source; update the generator or source docs instead.

5. Maintain the log.
   - Record date, action, sources inspected, pages changed, and follow-up risks.
   - The log is not a changelog for every typo. Use it for structural changes and important knowledge decisions.

## Operations

### ingest

Use when adding knowledge from sources into the wiki.

- Read the source and the relevant existing page.
- Update the authoritative page only.
- Add or adjust links from the index if the topic becomes a common route.
- Add a log entry when the ingest changes navigation, ownership, or project assumptions.

### query

Use when answering from the wiki.

- Read the index first.
- Follow only the links needed for the question.
- Distinguish sourced facts from inference.
- If the answer reveals a durable missing fact, propose or apply an ingest after answering when the user asked for maintenance.

### lint

Use when checking wiki health.

- Check broken Markdown links.
- Find orphan pages that no index or topic page references.
- Find duplicated facts across pages.
- Find pages that mix index, reference, architecture, and release status in one place.
- Verify generated/distributed docs do not link to files that are absent from the distributed artifact.

### refactor

Use when reorganizing docs without changing product behavior.

- Start by locking the intended reading paths.
- Shorten index pages before expanding topic pages.
- Move detail from the index into existing topic pages when possible.
- Leave compatibility redirects or clear references if files are moved.
- Run link and formatting validation before reporting completion.

## Output Standard

For implementation work, report:

- the chosen wiki structure,
- the authoritative page for each major fact family,
- links or pages changed,
- validation performed,
- remaining stale or ambiguous areas.

For review work, lead with broken links, contradictory facts, missing entrypoints, stale generated docs, and overgrown index pages.
