# sklink crates.io 发布脚本 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在仓库内固化一套发布到 crates.io 的命令/脚本，实现 SemVer bump（patch/minor/major）、自动修改 `sklink/Cargo.toml` 版本号、跑 fmt/clippy/test、Lore 提交、打 tag、cargo publish。

**Architecture:** 增加一个 bash 脚本（单文件）负责 orchestrate 发布流程；不引入项目运行时依赖，仅依赖 git 与 cargo。默认不做 git push，避免误推；用户可手动 push 或后续加开关。

**Tech Stack:** bash, git, cargo (fmt/clippy/test/publish)

---

## 文件结构与改动范围

**新增：**
- `scripts/release-sklink`（可执行脚本）
- `scripts/README.md`（脚本说明与示例）

**修改：**
- `sklink/README.md`（增加“发布”章节链接到 scripts 文档，避免重复写长说明）

---

### Task 1: 写回归测试（脚本级“可验证性”）

**Files:**
- Create: `scripts/tests/release_sklink.bats`（如果仓库不想引入 bats，则改为纯 bash 的自测脚本）

- [ ] **Step 1: 决定测试框架**
  - 约束：不新增依赖的前提下，使用纯 bash 写最小自测脚本（推荐）

- [ ] **Step 2: 写最小自测用例（纯 bash）**
  - 用例 1：`patch` 能把 `0.1.0` → `0.1.1`
  - 用例 2：`minor` 能把 `0.1.9` → `0.2.0`
  - 用例 3：`major` 能把 `0.9.9` → `1.0.0`
  - 用例 4：非法参数返回非 0，并打印 usage
  - 用例 5：dirty git worktree 时拒绝执行

- [ ] **Step 3: 运行自测脚本**
  - Run: `bash scripts/tests/release_sklink.sh`
  - Expected: exit 0

- [ ] **Step 4: Lore 提交**
  - Commit 只包含 tests（如果新增）

---

### Task 2: 实现 `scripts/release-sklink`（SemVer bump + 全流程）

**Files:**
- Create: `scripts/release-sklink`

- [ ] **Step 1: 脚本接口定义**
  - `scripts/release-sklink [patch|minor|major]`
  - 默认：patch
  - `--dry-run`：只打印将执行的动作，不修改/不提交/不发布（便于审核）

- [ ] **Step 2: 前置检查**
  - git worktree clean：`git status --porcelain` 必须为空
  - 当前分支允许发布（只提示不强制，避免过度约束）
  - cargo 可用：`command -v cargo`

- [ ] **Step 3: 读写版本号**
  - 从 `sklink/Cargo.toml` 解析 `version = "x.y.z"`
  - 计算新版本（major/minor/patch）
  - 写回 `sklink/Cargo.toml`（只替换 version 行）

- [ ] **Step 4: 校验与回归**
  - 在 `sklink/` 下执行：
    - `cargo fmt --check`
    - `cargo clippy -- -D warnings`
    - `cargo test`

- [ ] **Step 5: Lore 提交（版本 bump）**
  - `git add sklink/Cargo.toml`
  - `git commit` 使用 Lore 协议（trailer 中文）

- [ ] **Step 6: 打 tag**
  - `git tag vX.Y.Z`

- [ ] **Step 7: 发布**
  - `cargo publish`（cwd: `sklink/`）
  - 失败时退出非 0；并提示用户检查登录状态与 token

- [ ] **Step 8: dry-run 验证**
  - Run: `scripts/release-sklink patch --dry-run`（不应修改工作区）
  - Expected: 输出动作列表并 exit 0

- [ ] **Step 9: Lore 提交**
  - 如 Task 2 包含脚本与文档，需要单独提交（或与 Task 3 合并，按你偏好）

---

### Task 3: 文档化脚本用法

**Files:**
- Create: `scripts/README.md`
- Modify: `sklink/README.md`

- [ ] **Step 1: scripts/README.md**
  - 写清楚：
    - `cargo login` 前置要求
    - `scripts/release-sklink patch|minor|major`
    - 不会自动 `git push`（需要手动执行 `git push && git push --tags`）
    - 常见失败排查（token、2FA、crate 名冲突等）

- [ ] **Step 2: sklink/README.md 增加发布入口**
  - 增加一个简短 “发布” 段落，链接到 `scripts/README.md`

- [ ] **Step 3: Lore 提交**
  - 提交文档变更

---

### Task 4: 最终回归

**Files:**
- Modify:（仅在修复遗漏时）

- [ ] **Step 1: repo 自检**
  - `git status --porcelain` 必须为空

- [ ] **Step 2: sklink 回归**
  - Run: `cargo test`（cwd: `sklink/`）
  - Expected: PASS

- [ ] **Step 3: Lore 提交**
  - 如果 Task 4 有额外修复，补最后一次提交

