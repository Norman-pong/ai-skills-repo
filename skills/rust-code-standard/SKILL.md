---
name: rust-code-standard
description: 执行 Rust 官方编码规范检查，包括命名约定（RFC 430）、API 设计指南（C-COMMON-TRAITS、C-CONV 等）、文档规范（C-EXAMPLE、C-FAILURE）及测试组织规范。当用户要求检查 Rust 代码质量、运行 cargo fmt/clippy、审查代码风格、准备 PR 或验证 crate 是否符合官方约定时触发。
---

# Rust 编码规范

执行 Rust 代码质量检查并强制官方编码约定。

## 质量门禁

按顺序运行以下两条命令：

```bash
# 1. 格式检查
cargo fmt --check

# 2. 静态分析
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings
```

任一失败则停止并修复后再继续。

## 官方参考文档

在实现或审查特定领域时加载对应参考文档：

- **编码风格与命名**: 参见 [references/style-guide.md](references/style-guide.md) — RFC 430 命名约定、rustfmt 配置、转换方法（C-CONV）、getter（C-GETTER）、迭代器命名（C-ITER / C-ITER-TY）
- **API 设计与文档**: 参见 [references/api-guidelines.md](references/api-guidelines.md) — 公共 trait（C-COMMON-TRAITS / C-SEND-SYNC）、转换 trait（C-CONV-TRAITS）、错误类型（C-GOOD-ERR）、文档规范（C-EXAMPLE / C-QUESTION-MARK / C-FAILURE）、泛型设计（C-GENERIC / C-INTERMEDIATE）、trait 对象安全（C-OBJECT）
- **测试组织**: 参见 [references/testing-guide.md](references/testing-guide.md) — 官方测试目录结构、单元/集成/doc-test 分工、断言宏、panic 测试、测试命名、async 测试

## 速查表

### 命名（RFC 430 / C-CASE）

| 项目 | 约定 |
|---|---|
| Crates | `snake_case`（优先单字） |
| 类型/枚举/Trait | `UpperCamelCase` |
| 函数/方法/模块 | `snake_case` |
| 常量/静态 | `SCREAMING_SNAKE_CASE` |
| 缩写 | 视为一个单词：`Uuid`, `is_xid_start` |

### 转换方法（C-CONV）

| 前缀 | 开销 | 所有权 |
|---|---|---|
| `as_` | 零 | borrowed → borrowed |
| `to_` | 有 | borrowed → borrowed / borrowed → owned (non-Copy) / owned → owned (Copy) |
| `into_` | 可变 | owned → owned (non-Copy) |

### Getter（C-GETTER）

- 不使用 `get_` 前缀：`first()` 而非 `get_first()`
- 可变引用：`first_mut()` 而非 `get_first_mut()`

### 公共 Trait（C-COMMON-TRAITS）

新类型应主动实现：`Copy`, `Clone`, `Debug`, `Display`, `Default`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`

### 错误类型（C-GOOD-ERR）

- 实现 `std::error::Error + Send + Sync`
- **不使用 `()` 作为错误类型**
- `Display` 输出小写、无尾随标点

### 文档（C-EXAMPLE / C-FAILURE）

- 每个公共项有 rustdoc 示例
- 示例使用 `?` 而非 `unwrap`
- 公共函数包含 `# Errors` / `# Panics` / `# Safety`（如适用）

### 测试组织

- 单元测试：**文件 ≤ 300 行**时源文件内 `#[cfg(test)] mod tests`；**超过 300 行**必须分离到独立测试文件
- 集成测试：`tests/*.rs`，只使用公共 API
- Doc 测试：公共函数文档中的 `/// ``` ` 代码块

## 工作流

1. 运行 `cargo fmt --check` — 用 `cargo fmt` 自动修复
2. 运行 `cargo clippy --workspace --all-targets --all-features --tests -- -D warnings` — 修复所有警告
3. 报告结果：通过 / 失败，并给出具体错误数量和位置

## 常见修复

| Clippy 警告 | 典型修复 | 对应规范 |
|---|---|---|
| `unnecessary_wraps` | 若函数永不失败则移除 `Result` | C-GOOD-ERR |
| `too_many_arguments` | 引入配置结构体或 Builder 模式简化参数 | C-GENERIC |
| `cast_possible_truncation` | 使用 `try_into()` 或 `saturating_*` | C-CONV-TRAITS |
| `missing_errors_doc` | 添加 `# Errors` 章节 | C-FAILURE |
| `missing_panics_doc` | 添加 `# Panics` 章节 | C-FAILURE |
| `unwrap_used` | 库代码替换为 `?` 或显式错误处理；测试代码可适度使用 | C-GOOD-ERR |
