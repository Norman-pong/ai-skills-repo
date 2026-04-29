# Google AIP Design Guidance For Elysia APIs

## Contents

- [Purpose](#purpose)
- [Core AIPs](#core-aips)
- [Resource-Oriented Design](#resource-oriented-design)
- [Resource Names And Paths](#resource-names-and-paths)
- [Standard Methods](#standard-methods)
- [Custom Methods](#custom-methods)
- [Pagination And Lists](#pagination-and-lists)
- [Errors](#errors)
- [Lifecycle State And Soft Delete](#lifecycle-state-and-soft-delete)
- [Resource Hierarchy Decisions](#resource-hierarchy-decisions)
- [Applying AIP To Non-Google REST APIs](#applying-aip-to-non-google-rest-apis)

## Purpose

Use Google API Improvement Proposals (AIPs) as a design checklist for externally visible Elysia APIs. AIPs are not a requirement to mimic Google RPC/protobuf infrastructure; they are a source of durable API surface conventions.

Prefer host-project conventions when they are already established. Use AIP guidance to catch unclear resource names, overly RPC-like endpoints, inconsistent list responses, vague errors, and unsafe compatibility choices.

Primary official references:

- `https://google.aip.dev/121` resource-oriented design
- `https://google.aip.dev/122` resource names
- `https://google.aip.dev/131` through `https://google.aip.dev/136` standard and custom methods
- `https://google.aip.dev/158` pagination
- `https://google.aip.dev/193` errors
- `https://google.aip.dev/general` full index

## Core AIPs

Check these first for typical Elysia REST work:

- AIP-121: model APIs as resources, relationships, schemas, and mostly standard methods.
- AIP-122: define stable resource names and hierarchical paths.
- AIP-131 to AIP-135: Get, List, Create, Update, Delete method shapes.
- AIP-136: custom methods for actions that do not fit standard methods.
- AIP-158: pagination request/response shape and validation.
- AIP-160: filtering when collection filtering is needed.
- AIP-164: soft delete behavior.
- AIP-180 and AIP-185: compatibility and versioning for durable public APIs.
- AIP-193: standardized error responses.
- AIP-211: authorization checks when auth semantics affect API visibility.
- AIP-216: resource states.

## Resource-Oriented Design

Before route design, name the resources first:

1. List resources as nouns, not actions.
2. Define parent-child hierarchy and ownership.
3. Define each resource schema.
4. Attach methods to resources, preferring standard methods.

Avoid mirroring the database schema exactly. API resources may combine, hide, or reshape storage details to preserve a stable external contract.

Strict AIP expects most mutable resources to support read-back through Get, and collections to support List unless the resource is singleton or deliberately hidden. For internal or non-Google APIs, document why Get/List is intentionally omitted.

## Resource Names And Paths

For durable APIs, prefer canonical resource names and paths:

- Use plural collection segments: `/publishers/{publisher}/books/{book}`.
- Alternate collection and resource ID segments.
- Prefer lower-case, URL-safe resource IDs; document user-specified ID formats.
- Include a canonical `name` field in resource response models when the API needs stable resource references, for example `publishers/123/books/les-miserables`.
- Use simple `id` fields only when the host project already standardizes on ID-only resources or the API is internal.
- Avoid tuple identifiers in response models; prefer one canonical string name for references.
- Use aliases like `users/me` only for request lookup convenience; responses should return canonical names.

## Standard Methods

Map common operations to standard method semantics unless the host project has a deliberate REST convention:

| Operation | HTTP | Path shape | Response |
| --- | --- | --- | --- |
| Get | `GET` | `/{resourceName}` | resource |
| List | `GET` | `/{collection}` | `{ resources, nextPageToken? }` or project equivalent |
| Create | `POST` | `/{collection}` | created resource |
| Update | `PATCH` | `/{resourceName}` | updated resource |
| Delete | `DELETE` | `/{resourceName}` | empty response, deleted resource, or project convention |

Prefer `PATCH` for partial updates. Use `PUT` only when the project intentionally supports full replacement and has compatibility rules for added fields.

For Create, keep the created resource in the request body. If caller-chosen IDs are supported, document the ID field and conflict behavior.

For Update, define which fields are mutable. If the API needs partial update precision, use an update mask, changed-field list, or project-standard equivalent.

Keep ordinary Update for side-effect-light data changes. State transitions, publication, moderation, recovery, and other operations with distinct authorization or invariants should use named custom methods. For durable APIs, define concurrency behavior with ETags, updated-at preconditions, version columns, or the project-standard equivalent; return 409 or the project-equivalent conflict error on stale writes.

## Custom Methods

Use custom methods only when standard methods do not fit the semantics. Good candidates include state transitions and imperative operations, such as publish, archive, approve, retry, cancel, or rotate.

Guidelines:

- Prefer resource-based custom methods: `POST /articles/{article}:publish`.
- Use collection-based custom methods only for collection-wide actions: `POST /articles:batchPublish`.
- Use `GET` only for side-effect-free custom reads; use `POST` for mutations.
- Name service methods as verb+noun, such as `publishArticle`, not `updateArticleStatus` when the operation has distinct rules.
- Do not use custom methods to avoid designing resources or filters. For example, prefer list/search filters over `getByAuthor` style endpoints when it is a collection query.

If the host project does not use colon-style custom method paths, adapt the convention while preserving the same design intent, such as `POST /articles/{article}/publish`.

## Pagination And Lists

For collection endpoints, design pagination deliberately:

- Include `pageSize`/`page_size` and `pageToken`/`page_token` equivalents unless the project standardizes on offset/page.
- Document default and maximum page size.
- Treat `pageSize = 0` or omitted page size as the default, following AIP-158 unless the host project differs.
- Coerce page sizes above the maximum down to the maximum.
- Reject negative or invalid page sizes.
- Return `nextPageToken`/`next_page_token` or the project-standard continuation field.
- Make page tokens opaque and URL-safe. Do not expose SQL cursors, offsets, internal IDs, or parseable implementation details as API contracts.
- Do not encode authorization into page tokens. Every page request must rerun auth and visibility filtering.
- Keep non-pagination query parameters stable across page-token requests.
- Allow `pageSize` to change on later page-token requests, but treat changes to filters, sort, visibility scope, or parent resource as invalid.
- Define sort and filter fields as whitelists.
- Enforce public/protected visibility in the service/database query, not only in the route handler.

For simple internal APIs, offset pagination may be acceptable if already established; still document defaults, max limits, stable ordering, and invalid input behavior.

For filtering, prefer the host project’s existing query parameter style. Use AIP-160-style `filter` only when a durable public API needs extensible structured filtering. Otherwise, use explicit allowlisted query params such as `tag`, `author`, `state`, `q`, and `sort`. Do not add `getByAuthor` or `getByTag` custom methods when List with filters is the real operation.

## Errors

Use the project’s error helpers, but keep error semantics consistent and machine-readable:

- Use stable error codes or reason strings when clients may branch on errors.
- Return validation details for field-specific failures when the project supports them.
- Distinguish authentication, authorization, not found, conflict, and validation errors.
- Avoid making clients parse human-readable messages.
- Keep message language consistent with the project’s API convention.

Recommended HTTP mapping for REST APIs:

- 400 or 422: invalid request according to project convention.
- 401: missing or invalid authentication.
- 403: authenticated actor lacks permission.
- 404: caller has permission for the scope but the resource does not exist.
- 409: uniqueness, state, or concurrency conflict.

AIP-style default: check permission before existence when the caller lacks permission for the resource or parent resource, and return 403 for permission denial. Some products intentionally return 404 on public surfaces to hide resource existence; treat that as a security-policy deviation from AIP, state it in the plan, and test it consistently across that surface.

## Lifecycle State And Soft Delete

For resources with lifecycle workflows, prefer a `state` field over `status` unless the host project already standardizes on `status`.

Guidelines:

- `state` is normally output-only or service-controlled.
- Do not allow ordinary Create/Update to set arbitrary lifecycle states unless the project explicitly supports it.
- Use custom methods for transitions: publish, archive, submit, approve, reject, delete, undelete, purge, or project-specific verbs.
- Invalid transitions should return a precondition/conflict error according to project convention, commonly 409.
- Tests should cover valid transitions, invalid transitions, authorization failures, and visibility after transitions.

For soft delete:

- Decide hard delete vs soft delete before implementing Delete.
- If resources can be restored, model delete metadata such as `deleteTime` and `purgeTime` or project equivalents.
- Use `POST /{resource}:undelete` or the host project’s equivalent path for restore operations.
- Public lists should normally exclude deleted resources; admin lists should require explicit filtering to include them.
- Delete may return the updated soft-deleted resource or the project-standard empty response; document the chosen contract.

## Resource Hierarchy Decisions

Prefer canonical collections for resources with their own lifecycle. Treat alternate views as filters unless they create independent ownership.

Examples:

- Articles/posts are usually primary resources.
- Comments are usually child resources of articles when their lifecycle depends on the parent article.
- Tags/categories are lookup or collection resources; keep them inside a parent module until they need independent CRUD, uniqueness rules, or reuse across modules.
- `/users/{user}/articles` and `/tags/{tag}/articles` are often filtered article collections, not separate resource types.
- Admin routes are usually a management surface over the same resource, not separate `AdminArticle` resources unless the response model, lifecycle, or ownership truly differs.

## Applying AIP To Non-Google REST APIs

When adapting AIP to Elysia/TypeScript:

- Translate proto field names to the project’s JSON naming style (`pageToken` vs `page_token`).
- Preserve established route prefixes and versioning style.
- Do not introduce protobuf-specific constructs unless the project already uses them.
- Mention AIP tradeoffs in the plan when choosing a project convention over AIP shape.
- Add route tests for AIP-sensitive behavior: method/path shape, list pagination, custom method status, and error mapping.

Small adaptation examples:

- AIP custom method: `POST /articles/{article}:publish`; project path equivalent: `POST /articles/{article}/publish`. Preserve the custom-method semantics and tests even if the path style changes.
- AIP cursor pagination: `pageSize`, `pageToken`, `nextPageToken`; project offset equivalent: `page`, `limit`, `total`. Preserve documented defaults, max limits, stable ordering, invalid parameter behavior, and visibility filtering.
