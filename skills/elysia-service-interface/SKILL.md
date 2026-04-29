---
name: elysia-service-interface
description: Create or update modular Elysia service APIs in TypeScript backends with route/plugin/service organization. Use when Codex needs to add REST endpoints, design REST resource names, pagination, API versioning, error semantics, TypeBox request/response schemas, service-layer behavior, Elysia plugin registration, route/service tests, or evaluate module cohesion, aggregate boundaries, auth boundaries, persistence, and whether new code is justified instead of blindly adding files or abstractions.
---

# Elysia Service Interface

## Overview

Use this skill to implement Elysia APIs by following the host project’s existing module conventions. Treat nearby modules as the source of truth, then preserve a clean split between transport contracts, route composition, and business logic.

Read `references/module-pattern.md` before adding a new module, changing a public API contract, or touching unfamiliar server code. Read `references/google-aip.md` before designing or renaming externally visible REST resources, methods, pagination, errors, or API versions.

## Preflight Judgment

Before writing code, decide whether new code is justified.

1. Identify the user-visible behavior or API contract being changed.
2. Search existing modules, services, schemas, plugins, and shared helpers for reusable behavior.
3. Decide whether the change belongs in an existing module, a new module, or shared infrastructure.
4. Prefer extending an existing cohesive module when the behavior uses the same domain nouns, data ownership, authorization rules, and lifecycle.
5. Create a new module only when the behavior has a distinct domain boundary, independent route prefix, separate persistence/integration ownership, or different security/lifecycle concerns.
6. Extract shared code only after at least two real callers need the same behavior and the shared abstraction has a stable name.
7. If the request can be solved by deleting, moving, or reusing code, do that before adding new code.
8. For public or durable API surfaces, check Google AIP guidance before finalizing resource names, methods, pagination, filtering, errors, and versioning. Follow host-project conventions first when they intentionally differ, but call out the tradeoff.

## Cohesion And Dispersion Boundaries

Keep code cohesive when:

- The operations manipulate the same aggregate or resource.
- The same service owns validation, persistence, and external integration.
- Routes share one API prefix and one OpenAPI tag.
- Tests naturally set up the same fixtures.

Split or disperse code when:

- A file starts mixing unrelated domain nouns or unrelated external systems.
- Route handlers need different auth, rate limit, lifecycle, or deployment assumptions.
- A service has to import from several sibling modules to finish a normal operation.
- Adding one endpoint would force many optional fields or branching paths into existing models.
- Tests need heavy unrelated setup just to exercise the new behavior.

Do not split just to make files smaller. Do split when ownership, invariants, or failure modes are different.

## Workflow

1. Inspect the nearest existing module shape, naming, error helpers, test runner, and route registration style.
2. Write a short implementation plan with the cohesion decision: reuse existing module, create new module, or extract shared helper.
3. If the API surface is new or durable, map resources and methods using `references/google-aip.md`.
4. Define schemas/models for params, query, body, responses, and expected error responses.
5. Implement service behavior with plain typed inputs and outputs; keep it independent from Elysia request contexts.
6. Compose Elysia routes with validators, error handling, and API metadata in the project’s existing style.
7. Register routes through the project’s existing composition or registration path, preserving order and plugin boundaries.
8. Add focused tests for business rules; add route tests when validation, auth, serialization, or status codes matter.
9. Run focused verification first, then broader checks when shared contracts or app registration changed.

## File Responsibilities

- `model.ts`, `models.ts`, or equivalent: TypeBox schemas and derived TypeScript types. Match existing naming (`Schema`, `Model`, or local convention).
- `service.ts` or equivalent: Domain behavior, persistence calls, integration calls, normalization, and expected failures. Avoid Elysia-specific request context here.
- `index.ts`, `routes.ts`, or equivalent: Elysia plugin/controller composition. Keep route handlers thin.
- `__tests__`, `*.test.ts`, or project test location: Business rule tests first; route tests for transport boundaries.

## Elysia Route Checklist

For each endpoint, specify:

- Method and path, including route prefix.
- Resource name and hierarchy for durable APIs; prefer noun resources and standard methods before custom action endpoints.
- Whether the route is public or protected.
- `params`, `query`, and `body` schemas when inputs exist.
- Success response schema and expected error response schemas.
- Error handling path using the project’s existing error helpers, including status-code mapping for expected failures.
- API documentation metadata if the project exposes Swagger/OpenAPI: tags, summary, description, security, params/query/body/response schemas, and examples when the project uses them.
- Auth/security metadata when the surrounding project uses it.

## Complex Feature Checklist

For features with multiple resources, roles, or lifecycle states, add these notes to the implementation plan before editing:

- Aggregate roots and ownership: which module owns each resource, child resource, lookup table, and lifecycle.
- Route groups by policy: public, protected user, admin/staff, webhook, or internal routes.
- Authorization matrix: actor type, ownership rule, allowed states, forbidden states, and expected status codes.
- Persistence plan: tables, indexes, uniqueness, foreign keys, soft-delete policy, transactions, migrations, and test cleanup.
- Lifecycle commands: named service methods for state transitions, not generic status mutation when invariants differ.
- List contract: pagination, max limit, sort whitelist, filters, visibility rules, and stable ordering.
- Cross-module dependencies: avoid bidirectional service imports; prefer stable read helpers, policy helpers, or database constraints.

## Coding Rules

- Match the project’s module structure, imports, naming, language, comments, and test runner.
- Use TypeScript ESM and strict types. Avoid `any`; prefer `unknown` with narrowing when input is dynamic.
- Reuse existing error classes, response envelopes, database helpers, gateway/client helpers, and test utilities.
- Avoid business logic in route handlers.
- Avoid new dependencies unless explicitly requested or already established in the project.
- Keep comments sparse and in the project’s existing language.
- Keep API-visible strings consistent with the repository’s localization and documentation rules.

## Verification

Prefer focused commands discovered from `package.json`, such as:

```bash
pnpm --filter <server-package> test <path-to-test>
pnpm --filter <server-package> typecheck
```

If package filters are unknown, inspect scripts first and run the narrowest valid command. Use broader test/typecheck/lint commands when the change affects shared helpers, route registration, auth, or generated API types.

Add runtime or integration verification when typechecking is not enough:

- Route registration changed: run route-level smoke tests with `app.handle`, HTTP tests, or the project’s integration test harness.
- Auth/plugin order changed: test at least one public route and one protected route.
- OpenAPI/response schemas changed: run schema generation, snapshot, or generated client checks if the project has them.
- Error status behavior changed: test the visible 401/403/404/409/422 or project-equivalent status mapping.
- Error helper behavior changed: test one service-level expected failure and one route-level serialized error envelope.
