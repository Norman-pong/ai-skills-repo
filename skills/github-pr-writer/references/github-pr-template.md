# GitHub PR Template Reference

Use this when a repository does not provide `.github/pull_request_template.md`, or when creating a future PR draft that should follow common GitHub review expectations.

## Standard Template

```markdown
## Summary

<!-- Explain why this PR is needed and what it changes in 2-4 sentences. -->

## Change Type

- [ ] Feature
- [ ] Bug fix
- [ ] Refactor / cleanup
- [ ] Documentation
- [ ] Build / release / tooling
- [ ] Tests

## Related Context

<!-- Link issues, tasks, discussions, specs, incidents, upstream constraints, or source docs. Write "None" if not applicable. -->

## Key Changes

-

## Scope Boundaries and Risks

<!-- State what is delivered, what is excluded, and what remains risky or unverified. -->

- Delivered:
- Not included:
- Known risks:

## Testing and Verification

<!-- List commands, browser/manual checks, review passes, and results. Leave planned checks unchecked. -->

- [ ] Unit tests:
- [ ] Integration tests:
- [ ] Build / typecheck / lint:
- [ ] Browser / manual verification:
- [ ] Documentation / link check:
- [ ] Reviewer or subagent review:

## Documentation Impact

- [ ] No documentation update needed
- [ ] README / usage docs updated
- [ ] Architecture / design docs updated
- [ ] Release notes / migration docs updated
- [ ] Deferred work, risks, or follow-up plan recorded

## Screenshots or Recordings

<!-- Required for UI, demo, visual, browser, or playback changes. Otherwise write "No visual change." -->

## Pre-Merge Checklist

- [ ] No generated artifacts, dependency folders, local credentials, large logs, or private data are committed.
- [ ] No unrelated formatting, renames, or cleanup are mixed in.
- [ ] Public API, compatibility, performance, security, and release impacts are explained when relevant.
- [ ] Commit messages follow the repository convention.
```

## Chinese Section Names

Use these names when the repository prefers Chinese:

- `摘要`
- `变更类型`
- `关联事项`
- `主要变更`
- `能力边界与风险`
- `测试与验证`
- `文档影响`
- `截图或录屏`
- `提交前检查`

## Future PR Draft Adjustment

For a future-stage PR draft, add one sentence near the top:

```markdown
当前仅交付 PR 草案文档，不代表该功能已交付。
```

Keep implementation checkboxes unchecked unless the implementation already exists and has been verified.
