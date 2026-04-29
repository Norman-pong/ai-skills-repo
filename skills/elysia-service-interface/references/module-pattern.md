# Elysia Module Pattern

## Contents

- [Purpose](#purpose)
- [Boundary Decision](#boundary-decision)
- [`model.ts`](#modelts)
- [`service.ts`](#servicets)
- [`index.ts` Or `routes.ts`](#indexts-or-routests)
- [App Registration](#app-registration)
- [Error Handling](#error-handling)
- [Swagger And OpenAPI](#swagger-and-openapi)
- [Complex Domain Patterns](#complex-domain-patterns)
- [Tests](#tests)
- [Smell Checks](#smell-checks)

## Purpose

This reference describes a reusable Elysia module pattern. Adapt names and file layout to the host project instead of forcing this exact structure.

Common module shapes:

```text
modules/<feature>/
├── index.ts      # Elysia routes/plugin
├── model.ts      # TypeBox schemas and exported types
├── service.ts    # Business logic
└── __tests__/    # Focused tests
```

Other valid names include `routes.ts`, `models.ts`, `controller.ts`, or colocated test files. Follow the existing repository.

## Boundary Decision

Use this decision table before adding module code.

| Question | Existing module | New module | Shared helper |
| --- | --- | --- | --- |
| Same domain resource and same lifecycle? | Yes | No | No |
| Different route prefix or OpenAPI tag? | Maybe | Yes | No |
| Different auth or operational policy? | Maybe | Yes | No |
| Same logic needed by two or more modules? | No | No | Maybe |
| New behavior only adapts one endpoint? | Yes | No | No |
| Existing module would gain unrelated nouns? | No | Yes | Maybe |

If the answer is uncertain, make the smallest reversible change that preserves clear ownership.

## `model.ts`

Define TypeBox schemas with `t` from `elysia`, then export TypeScript types via `Static`.

```ts
import { t, type Static } from "elysia"

export const CreateThingRequestModel = t.Object(
  {
    name: t.String({ minLength: 1, description: "Name" }),
  },
  { description: "Create thing request" }
)

export type CreateThingRequest = Static<typeof CreateThingRequestModel>

export const ThingResponseModel = t.Object(
  {
    id: t.String({ description: "Thing ID" }),
    name: t.String({ description: "Name" }),
  },
  { description: "Thing response" }
)

export type ThingResponse = Static<typeof ThingResponseModel>
```

Guidelines:

- Keep request, response, params, and query schemas explicit.
- Use `t.Optional(...)` for optional fields.
- Use `t.Union([...])` for nullable or discriminated shapes.
- Use descriptions/examples only when the project’s API docs benefit from them.
- Avoid widening response models to fit unrelated variants; split responses or endpoints when semantics differ.

## `service.ts`

Services should be independent from Elysia request objects. Accept plain typed inputs and return plain typed outputs. Throw project-standard errors for expected failures.

```ts
import { BadRequestError } from "../../common/errors"
import type { CreateThingRequest, ThingResponse } from "./model"

export abstract class ThingService {
  static createThing(input: CreateThingRequest): ThingResponse {
    const name = input.name.trim()

    if (!name) {
      throw new BadRequestError("Name is required")
    }

    return {
      id: crypto.randomUUID(),
      name,
    }
  }
}
```

Class services, named functions, and factory functions are all acceptable when they match the host project. Do not introduce dependency injection, repositories, or generic service bases unless the project already uses them or the immediate change truly needs them.

## `index.ts` Or `routes.ts`

Route files compose Elysia plugins/controllers. They should validate inputs, call services, and map response schemas.

```ts
import { Elysia } from "elysia"
import { ErrorModel, createErrorResponse } from "../../common/errors"
import { ThingService } from "./service"
import {
  CreateThingRequestModel,
  ThingResponseModel,
} from "./model"

export const thingRoutes = new Elysia({
  name: "Controller.Thing",
  prefix: "/things",
})
  .model({
    createThingRequest: CreateThingRequestModel,
    thingResponse: ThingResponseModel,
    errorResponse: ErrorModel,
  })
  .onError(({ error }) => createErrorResponse(error))
  .post(
    "/",
    async ({ body }) => ThingService.createThing(body),
    {
      body: "createThingRequest",
      response: {
        200: "thingResponse",
        400: "errorResponse",
      },
      detail: {
        tags: ["Things"],
        summary: "Create thing",
        description: "Create a new thing",
      },
    }
  )
```

Inline schemas are fine for route-local params and tiny responses. Shared `.model(...)` names are better when the same schema appears across multiple routes or in OpenAPI output.

## App Registration

Find the real entry point before wiring routes. Common locations include:

- `apps/*/src/index.ts`
- `src/index.ts`
- `src/app.ts`
- `src/server.ts`
- `src/routes.ts`

Preserve ordering. Public routes, protected routes, auth plugins, CORS, Swagger, static serving, and gateway proxies often depend on registration order.

## Error Handling

Follow the host project’s error contract instead of inventing new envelopes.

Guidelines:

- Services should throw project-standard errors for expected failures.
- Routes or global plugins should translate errors into the project-standard serialized response.
- Do not catch expected service errors in every handler just to rewrap them.
- Do not return `{ error: ... }` manually from services unless the host project already uses result objects.
- Keep validation errors, auth errors, not-found errors, conflict errors, and unexpected errors distinguishable.
- Route response schemas should include every expected API-visible error status.
- Tests should assert both the service exception/result and the route-level serialized status/body when the API contract changes.

Unexpected errors should flow through the project’s global error handler or logging policy. Add local `try/catch` only to add context, translate third-party errors into project errors, or release resources.

## Swagger And OpenAPI

When the project exposes Swagger/OpenAPI, keep route metadata synchronized with runtime behavior.

For every documented route, check:

- `detail.tags` matches the resource or route group.
- `summary` is short and action-specific.
- `description` explains semantics, not implementation details.
- `params`, `query`, `body`, and `response` schemas match the actual handler contract.
- Protected routes include the project-standard security metadata.
- Custom methods and state transitions document side effects and expected error statuses.
- List endpoints document pagination defaults, maximum limits, filtering, sorting, and visibility rules.
- Generated OpenAPI/client output is refreshed or tested when the project has a generation step.

Do not document fields that the handler never returns. Do not return fields missing from the documented response schema unless the project deliberately allows open response objects.

## Complex Domain Patterns

### Multi-Aggregate Boundaries

For features with multiple related nouns, identify aggregate roots before creating files. Do not create one broad module just because the user names one product area.

Prefer:

- Auth/session ownership stays in the existing auth module or auth plugin.
- User profile or admin user management becomes a user module only when it has its own routes and lifecycle.
- Content resources with lifecycle rules own their service and schemas.
- Child resources may become separate modules when they have independent permissions, moderation, lifecycle, or route prefixes.
- Lookup resources stay inside the parent module until they need independent CRUD, uniqueness rules, or reuse by multiple modules.
- Permissions/RBAC should usually be a shared auth helper/plugin unless the product exposes permission-management APIs.

Avoid bidirectional service imports. If cross-aggregate validation is needed, use stable read helpers, policy helpers, database constraints, or a narrowly named shared helper.

### Public, Protected, And Admin Route Groups

When a resource has different visibility or auth policies, split route composition by policy instead of branching heavily inside handlers.

A module may export multiple Elysia route groups:

- `publicArticleRoutes` for anonymous read-only endpoints.
- `protectedArticleRoutes` for logged-in user actions.
- `adminArticleRoutes` for staff/admin operations.

Each group should have its own prefix or policy assumptions, API metadata, auth assumptions, and tests. Preserve app registration order: public routes first, then auth plugin, then protected/admin routes unless the host project uses a different established composition model.

### Authorization Matrix

Before implementing protected endpoints, write a compact permission matrix in the plan:

- Actor type: anonymous, authenticated user, owner, moderator/editor, admin, service account, or project-specific role.
- Allowed resource states.
- Forbidden states and expected status code.
- Ownership check: user id, tenant id, role, permission string, policy helper, or database constraint.

Keep route handlers thin. Extract authorization checks into service-level policy helpers or existing auth utilities. Do not scatter raw role string checks across unrelated services unless the project already does so.

### Persistence And Migrations

When service behavior needs persistence, inspect the existing database layer before editing. Specify:

- Tables and ownership module.
- Primary keys, unique constraints, indexes, and foreign keys.
- Soft-delete vs hard-delete behavior.
- Timestamp and actor/audit conventions.
- Transaction boundaries for multi-table writes.
- Migration location and startup ordering.
- Test cleanup strategy.

Use transactions for operations that update multiple related tables, such as creating content with relation rows or deleting a parent resource with children.

### Lifecycle And State Transitions

For resources with workflow states, model state transitions explicitly in service methods. Prefer named commands such as `publishArticle`, `archiveArticle`, `approveComment`, or project-specific equivalents over generic `updateStatus` when permissions or invariants differ.

Tests must cover valid transitions, invalid transitions, authorization failures, and resulting visibility in public endpoints.

### List Query Contract

For collection endpoints, define a consistent query and response shape:

- Page/cursor and limit with defaults and max limit.
- Sort fields as a whitelist.
- Filter fields and their public/protected differences.
- Total/count semantics if returned.
- Stable ordering for pagination.

Public list services must enforce visibility filters in the service layer, not only in route handlers.

### Error Semantics

Use project-standard error helpers, but choose status semantics deliberately:

- 401 for missing or invalid authentication.
- 403 for authenticated actors without permission.
- 404 for resources that do not exist when the caller has permission for the scope.
- 409 for uniqueness or state conflicts.
- 400/422 for request validation according to project convention.

If a public surface intentionally uses 404 to hide resource existence from unauthorized callers, record that as a security policy choice and test it consistently. Otherwise, follow the usual distinction: 401 for unauthenticated, 403 for authenticated but unauthorized, 404 for missing resources.

Add route tests for each API-visible error status introduced.

## Tests

Use the repository’s test runner. For Bun projects:

```ts
import { describe, expect, it } from "bun:test"
import { ThingService } from "../service"
import { BadRequestError } from "../../../common/errors"

describe("ThingService", () => {
  it("creates a thing", () => {
    const result = ThingService.createThing({ name: "demo" })
    expect(result.name).toBe("demo")
  })

  it("rejects blank names", () => {
    expect(() => ThingService.createThing({ name: "" })).toThrow(BadRequestError)
  })
})
```

Add route tests when the change depends on Elysia validation, route registration, auth order, HTTP status codes, headers, or serialization. Service tests are enough for pure business rules.

For role-based modules, add a compact test matrix before implementation. Cover anonymous vs authenticated vs owner vs elevated role, visibility of lifecycle states, forbidden operations leaving persistence unchanged, validation failures at route level, and serialization/status-code behavior for public and protected routes.

Unit and route test checklist:

- Service tests cover success, expected domain failures, edge validation, state transitions, and persistence side effects.
- Route tests cover request validation, auth boundaries, status codes, response schemas, headers, and serialized error envelopes.
- Persistence tests isolate data with transactions, test databases, cleanup helpers, or project-standard fixtures.
- External clients should be mocked or faked at the service boundary unless the project already has integration-test infrastructure.
- OpenAPI or generated-client tests should run when schema metadata changes.
- Tests should prove forbidden operations do not mutate persistence.

## Smell Checks

Pause and reconsider if:

- A new endpoint needs more than one unrelated service to complete a basic request.
- A request model gains many optional fields to serve different workflows.
- A service method name contains “And” or multiple domain verbs.
- A new shared helper has only one caller.
- A route handler contains database queries, external API orchestration, or complex branching.
- Tests require unrelated fixtures from several modules.
- A new module exports types that are immediately imported back by many existing modules.
- Role checks are duplicated across services instead of going through a policy/helper boundary.
- A public list endpoint trusts route-level filtering rather than service/database visibility constraints.
- Route docs claim statuses or fields that are not covered by schemas or tests.
- Tests only cover service success paths and never exercise route serialization or error envelopes.

These signs do not always block the change, but they require a boundary explanation in the plan or final report.
