# Rust 测试规范

本文件汇总 Rust 官方测试组织规范。

## 来源

- [The Rust Book: Writing Automated Tests](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust By Example: Unit Testing](https://doc.rust-lang.org/rust-by-example/testing/unit_testing.html)
- [Rust By Example: Integration Testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
- Cargo 手册：测试目标（test targets）

## 目录

- [测试组织模型](#测试组织模型)
- [单元测试](#单元测试)
- [集成测试](#集成测试)
- [Doc 测试](#doc-测试)
- [测试命名规范](#测试命名规范)
- [断言宏](#断言宏)
- [测试 Result 返回值](#测试-result-返回值)
- [Panic 测试](#panic-测试)
- [忽略测试](#忽略测试)
- [运行特定测试](#运行特定测试)
- [测试覆盖检查](#测试覆盖检查)
- [Async 测试](#async-测试)
- [Clippy 测试相关 lint](#clippy-测试相关-lint)

---

## 测试组织模型

Rust 支持三种测试位置：

| 类型 | 位置 | 用途 |
|---|---|---|
| 单元测试 | 源码文件内 `#[cfg(test)] mod tests` | 测试私有函数和单个模块 |
| 集成测试 | `tests/` 目录下的 `.rs` 文件 | 测试 crate 公共 API |
| Doc 测试 | rustdoc `/// ``` ` 代码块 | 验证文档示例可编译运行 |

官方目录结构：

```
my-crate/
├── src/
│   ├── lib.rs
│   └── some_module.rs
└── tests/
    └── integration_test.rs
```

## 单元测试

单元测试有两种常见组织方式，**以 300 行为分界**选择：

> **规则：源文件行数超过 300 行时，单元测试必须分离到独立文件。**

### 方式一：内联测试（`#[cfg(test)]`）

适合**文件 ≤ 300 行**、需要测试私有函数的场景。

```rust
// src/some_module.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(1, 2), 3);
    }
}
```

- `#[cfg(test)]` 保证 `cargo build` 时不编译测试代码，不影响产物体积
- 使用 `use super::*;` 导入外部作用域名称
- **可以测试私有函数**（这是内联的主要优势）
- 缺点是会让源文件变长

### 方式二：分离到单独测试文件

适合测试量大、希望保持源文件整洁的场景。将单元测试放在 `src/` 下的独立模块文件中：

```
src/
├── lib.rs
├── parser.rs
└── parser_tests.rs   # 单元测试单独存放
```

```rust
// src/parser_tests.rs
#[cfg(test)]
mod tests {
    use crate::parser::*;

    #[test]
    fn test_parse() {
        // ...
    }
}
```

```rust
// src/lib.rs
pub mod parser;

#[cfg(test)]
mod parser_tests;
```

- 源文件更干净，阅读业务逻辑不受干扰
- 只能测试公共 API（无法访问私有函数）
- 如果私有逻辑复杂需要测试，考虑将其拆分为内部模块（`pub(crate)`）

## 集成测试

```rust
// tests/integration_test.rs
use my_crate::add;

#[test]
fn test_add_integration() {
    assert_eq!(add(2, 2), 4);
}
```

- 每个 `tests/` 下的 `.rs` 文件编译为独立的 test crate
- 集成测试只能使用被测 crate 的公共 API
- 共享代码可放在 `tests/common/mod.rs` 中

## Doc 测试

```rust
/// 返回参数加 2 的结果。
///
/// # Examples
///
/// ```
/// let result = my_crate::add_two(3);
/// assert_eq!(result, 5);
/// ```
pub fn add_two(n: i32) -> i32 {
    n + 2
}
```

- `cargo test` 自动编译和运行 doc 测试
- 示例代码中的 `assert!` 和 `panic!` 会使测试失败
- 使用 `#` 开头的隐藏行设置测试环境

## 测试命名规范

- 函数名描述行为：`fn add_two_positive_numbers()`
- 拒绝路径加否定词：`fn rejects_negative_input()`
- 使用蛇形命名：`fn invalid_transition_rejected()`
- 测试模块名固定为 `tests`

## 断言宏

| 宏 | 用途 |
|---|---|
| `assert!(expr)` | 表达式为 true |
| `assert_eq!(left, right)` | 两边相等 |
| `assert_ne!(left, right)` | 两边不等 |
| `assert!(result.is_ok())` | Result 为 Ok |
| `assert!(result.is_err())` | Result 为 Err |

## 测试 Result 返回值

Rust 2018+ 测试函数可返回 `Result<T, E>`，支持 `?` 运算符：

```rust
#[test]
fn test_with_question_mark() -> Result<(), Box<dyn std::error::Error>> {
    let val = some_fallible_op()?;
    assert_eq!(val, 42);
    Ok(())
}
```

## Panic 测试

```rust
#[test]
#[should_panic]
fn test_divide_by_zero_panics() {
    divide(1, 0);
}

#[test]
#[should_panic(expected = "divide by zero")]
fn test_divide_by_zero_specific_message() {
    divide(1, 0);
}
```

## 忽略测试

```rust
#[test]
#[ignore = "requires network"]
fn test_network_call() {
    // ...
}
```

运行被忽略的测试：`cargo test -- --ignored`

## 运行特定测试

```bash
# 匹配名称
cargo test runtime

# 精确匹配
cargo test execute_intent_full_execution_success

# 单测试文件
cargo test --test agent_runtime
```

## 测试覆盖检查

- 每个公共函数至少一个 happy path 测试
- 每个错误分支至少一个拒绝路径测试
- 边界条件（空输入、极大值、零值）必须有测试

## Async 测试

使用 `#[tokio::test]`（或项目选用的 async runtime 的 test macro）：

```rust
#[tokio::test]
async fn async_operation_completes() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

- 在测试中 `await` 可以替代 `block_on`
- async 测试中的超时由 test runner 控制

## Clippy 测试相关 lint

```bash
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings
```

测试代码也受 clippy 约束，包括：
- `unwrap_used`：**库代码**中应替换为 `?` 或显式错误处理；**测试代码**中使用 `unwrap` / `expect` 是广泛接受的标准做法
- `expect_used`：同上，测试代码中使用 `expect("原因")` 便于定位失败位置
- `missing_panics_doc`：不适用于测试函数
