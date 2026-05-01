# sklink CLI v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 sklink CLI v2（破坏兼容）：`-o` 移除并改为 `-p/--platform`，`-i` 增加 `--install`，显式平台不存在时错误退出，新增 `completions` 用于 tab 补全，并补齐更明确的错误提醒。

**Architecture:** 保持现有 clap derive 架构与“默认动作=安装”不变；在参数层做破坏兼容调整；新增 completions 子命令使用 clap_complete 输出脚本；错误提示通过集中化的错误打印增强可操作性。

**Tech Stack:** Rust 2021, clap derive, thiserror, toml, serde, tempfile（tests）, clap_complete（新增）

---

## 文件结构与改动范围

**修改：**
- [main.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/main.rs)（CLI 参数、平台选择逻辑、completions 子命令入口、错误输出 hint）
- [error.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/error.rs)（新增平台不存在错误、可用平台列表、config hint）
- [README.md](file:///Users/norman/workspace/zhimingcool/chore/sklink/README.md)（用法示例与说明同步 v2）
- [Cargo.toml](file:///Users/norman/workspace/zhimingcool/chore/sklink/Cargo.toml)（新增依赖 clap_complete）

**修改测试：**
- [cli_symlink_install.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/tests/cli_symlink_install.rs)（更新 `-o` → `-p`，新增平台不存在断言、`--install` 覆盖）
- [cli_list.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/tests/cli_list.rs)（如存在 `-o` 用法则更新）
- [cli_init.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/tests/cli_init.rs)（如存在 `-o` 用法则更新）

---

### Task 1: 破坏兼容参数调整（-o → -p/--platform，-i 增加 --install）

**Files:**
- Modify: [main.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/main.rs)
- Modify: [README.md](file:///Users/norman/workspace/zhimingcool/chore/sklink/README.md)
- Test: sklink/tests/*.rs

- [ ] **Step 1: 更新 clap 参数定义**
  - 删除原 `-o`/`platform` 定义
  - 增加 `-p, --platform <PLATFORM|all>`
  - 为 `-i` 增加长选项 `--install`
  - 保持 `--force` 与 subcommands 行为不变

- [ ] **Step 2: 更新 README 用法示例**
  - 替换所有 `-o` 示例为 `-p` 或 `--platform`
  - 新增一处示例展示 `--install`
  - 增加 completions 使用示例占位（真实实现于 Task 3）

- [ ] **Step 3: 更新测试用例参数**
  - 全量替换测试中 `-o` 为 `-p`
  - 补一条用例：`--install` 与 `-i` 一致（至少覆盖 1 条安装路径）

- [ ] **Step 4: 运行测试**
  - Run: `cargo test`（cwd: `sklink/`）
  - Expected: PASS

- [ ] **Step 5: 自检与提交（Lore Commit Protocol）**
  - Run: `git diff` / `git status`（确认无无关变更）
  - Commit message 需符合 Lore 协议（trailer 中文）

---

### Task 2: 平台不存在时错误退出 + 提示可用平台（含 hint）

**Files:**
- Modify: [main.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/main.rs)
- Modify: [error.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/error.rs)
- Test: sklink/tests/*.rs

- [ ] **Step 1: 增加平台不存在错误类型**
  - 在 `AppError` 增加 `PlatformNotFound { platform: String, available: Vec<String> }`（或等价结构）
  - `Display`（thiserror `#[error(...)]`）中包含平台名与可用平台列表

- [ ] **Step 2: 调整平台选择逻辑**
  - 当用户显式指定非 `all` 的平台且 config 不包含该平台：返回 `AppError::PlatformNotFound`
  - 当 `all`：按现有逻辑遍历

- [ ] **Step 3: 增强 config 相关错误的 hint**
  - 当错误为 ConfigRead/ConfigParse/ConfigAlreadyExists（或 init 失败相关）时，在 stderr 追加一行：`hint: run 'sklink init' to generate a default config`
  - 约束：不改变 stdout 语义

- [ ] **Step 4: 更新/新增测试**
  - 新增用例：`-p not-exist` 返回非 0；stderr 包含 “platform not found” 且包含 available platforms（使用写入的 config 平台列表）

- [ ] **Step 5: 运行测试**
  - Run: `cargo test`（cwd: `sklink/`）
  - Expected: PASS

- [ ] **Step 6: 自检与提交（Lore Commit Protocol）**
  - Run: `git diff` / `git status`
  - Commit message 需符合 Lore 协议（trailer 中文）

---

### Task 3: 新增 completions 子命令（tab 补全）

**Files:**
- Modify: [Cargo.toml](file:///Users/norman/workspace/zhimingcool/chore/sklink/Cargo.toml)
- Modify: [main.rs](file:///Users/norman/workspace/zhimingcool/chore/sklink/src/main.rs)
- Modify: [README.md](file:///Users/norman/workspace/zhimingcool/chore/sklink/README.md)
- Test: sklink/tests/*.rs

- [ ] **Step 1: 增加依赖**
  - `clap_complete`（版本与 clap 4.5.x 兼容）

- [ ] **Step 2: 增加 `completions` 子命令**
  - 形态：`sklink completions <SHELL>`
  - stdout 输出补全脚本
  - 支持 shell：bash/zsh/fish/powershell/elvish（以 clap_complete 的 enum 为准）

- [ ] **Step 3: README 补全安装说明**
  - 增加示例：`sklink completions zsh > _sklink`
  - 说明写入到对应 shell 的补全目录（仅给最小提示，不写太多平台相关细节）

- [ ] **Step 4: 测试覆盖**
  - 新增用例：`sklink completions zsh` 返回 0，stdout 非空且包含 `sklink`

- [ ] **Step 5: 运行测试**
  - Run: `cargo test`（cwd: `sklink/`）
  - Expected: PASS

- [ ] **Step 6: 自检与提交（Lore Commit Protocol）**
  - Run: `git diff` / `git status`
  - Commit message 需符合 Lore 协议（trailer 中文）

---

### Task 4: 端到端回归与最终审核

**Files:**
- Modify:（仅在需要修复遗漏时）

- [ ] **Step 1: 全量测试**
  - Run: `cargo test`（cwd: `sklink/`）
  - Expected: PASS

- [ ] **Step 2: 静态检查（如仓库已有）**
  - Run: `cargo fmt --check`（如失败则 `cargo fmt` 并提交）
  - Run: `cargo clippy -- -D warnings`（如仓库已有 clippy 习惯）

- [ ] **Step 3: 最终自检**
  - 确认 `sklink --help` 中不再出现 `-o`
  - 确认错误信息包含 hint/available platforms

- [ ] **Step 4: 最终提交（如 Task 2/3/4 产生额外修复）**
  - Commit message 需符合 Lore 协议（trailer 中文）

