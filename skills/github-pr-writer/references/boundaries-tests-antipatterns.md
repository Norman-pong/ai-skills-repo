# Boundaries, Tests, and Anti-Examples

Load this reference for non-trivial PRs, future-stage plans, release gates, compatibility work, security-sensitive changes, or when the user explicitly asks what to test or what to forbid.

## Boundary Rules

- Do not confuse a PR description with a durable wiki/spec. A PR records one reviewable change; link long-lived docs instead of duplicating them.
- Do not describe planned work as delivered. Use labels like `planned`, `future`, `deferred`, `not included`, or unchecked checklist items.
- Do not claim tests, browser checks, screenshots, benchmarks, audits, or reviews were run unless there is evidence.
- Do not check unrelated change types. A PR can mention tests, release risk, or security risk without being a `Tests`, `Build / release`, or `Security` PR.
- Do not include secrets, tokens, cookies, credentials, private keys, customer identifiers, internal hostnames, private IPs, or non-redacted production URLs.
- Do not broaden the PR to solve adjacent problems. Put follow-ups under risks, not included, or future work.
- Do not submit generated artifacts, dependency folders, local build outputs, copied binaries, or large logs unless the repo explicitly treats them as source.
- Do not mix one-off PR text into wiki entrypoints, architecture docs, or release docs. Update those only when the actual source of truth changes.
- Do not replace repo-specific templates with the generic template when a local template exists.

## Testing Guidance

Choose the lightest verification that proves the PR claim.

Documentation-only PR:

- Run a structural check against required PR sections.
- Check local Markdown links.
- Run whitespace checks such as `git diff --check`.
- Do not run product builds unless the docs affect package contents, release manifests, generated docs, or published artifacts.

Code PR:

- Run formatter, lint/typecheck, unit tests, and integration tests relevant to touched files.
- Add or update tests for changed behavior before claiming coverage.
- Record exact commands and results.
- Put unrun checks in `Not tested` or leave them unchecked with a reason.

UI, browser, media, or demo PR:

- Include screenshots or recordings when visual output changes.
- Verify local build and at least one real browser path.
- Record browser, OS, device/runtime capability, source URL shape, and known unsupported cases.

Release or package PR:

- Run package smoke tests in an environment that resembles consumer usage.
- Verify import paths, exported files, assets, workers, WASM/binary paths, MIME requirements, and registry/package metadata.
- Distinguish package publication success from runtime release readiness.

Security or privacy-sensitive PR:

- Check for leaked secrets, private URLs, logs, tokens, and customer data.
- State trust boundaries, auth changes, permission changes, and residual risk.

## Forbidden Anti-Examples

Bad: claiming future work is done.

```markdown
## Summary

This PR adds complete MP4 list playback with seek and H.265 support.

## Testing

- [x] All tests pass
```

Why it is forbidden: it gives no evidence, overclaims completeness, and marks tests as passed without commands.

Bad: leaking private infrastructure.

```markdown
Tested with https://10.1.100.182/record/private-camera-01.mp4 using token=...
```

Why it is forbidden: it exposes private network shape and credentials.

Bad: hiding exclusions.

```markdown
## Risks

None.
```

Why it is forbidden: every meaningful PR has compatibility, testing, operational, or maintenance boundaries.

Bad: mixing docs roles.

```markdown
I added the whole PR body to docs/architecture.md so future agents can find it.
```

Why it is forbidden: architecture docs should hold durable facts, not one-off review narratives.

Bad: using a generic template over a local one.

```markdown
The repository has .github/pull_request_template.md, but I replaced it with my preferred format.
```

Why it is forbidden: local repository conventions outrank generic skill defaults.

Bad: over-checking change types.

```markdown
## Change Type

- [x] Feature
- [x] Build / release / tooling
- [x] Tests
```

Why it is forbidden: a feature PR with a test plan is not automatically a test-infrastructure or build-tooling change.
