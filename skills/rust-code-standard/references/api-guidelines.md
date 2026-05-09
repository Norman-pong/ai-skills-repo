# Rust API 设计规范

本文件汇总 Rust API Guidelines 核心规范。

## 来源

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [RFC 1687: Crate Level Documentation](https://rust-lang.github.io/rfcs/1687-doc-comments-for-traits.html)
- [RFC 1574: API Documentation](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)

## 目录

- [公共 trait 实现（C-COMMON-TRAITS）](#公共-trait-实现c-common-traits)
- [线程安全（C-SEND-SYNC）](#线程安全c-send-sync)
- [转换 trait（C-CONV-TRAITS）](#转换-traitc-conv-traits)
- [集合 trait（C-COLLECT）](#集合-traitc-collect)
- [错误类型（C-GOOD-ERR）](#错误类型c-good-err)
- [文档规范](#文档规范)
- [泛型与灵活性（C-GENERIC / C-CALLER-CONTROL）](#泛型与灵活性c-generic--c-caller-control)
- [中间结果暴露（C-INTERMEDIATE）](#中间结果暴露c-intermediate)
- [Trait 对象安全（C-OBJECT）](#trait-对象安全c-object)

---

## 公共 trait 实现（C-COMMON-TRAITS）

新类型应主动实现所有适用的公共 trait：

| trait | 说明 | 优先级 |
|---|---|---|
| `Copy` | 小型值类型 | 按需要 |
| `Clone` | 需要显式复制的类型 | 推荐 |
| `Debug` | 所有公共类型 | **必须** |
| `Display` | 所有公共类型（错误消息、日志输出） | **必须** |
| `Default` | 有合理默认值时 | 推荐 |
| `PartialEq` / `Eq` | 可比较的类型 | 推荐 |
| `PartialOrd` / `Ord` | 可排序的类型 | 按需要 |
| `Hash` | 可作为 HashMap key 的类型 | 按需要 |

注意：`Default` 和 `new()` 可以共存。`new()` 是 Rust 的构造函数约定。

## 线程安全（C-SEND-SYNC）

`Send` 和 `Sync` 是编译器自动推导的 auto-trait。在涉及原始指针的类型中，需确保线程安全语义正确。

验证方式（编译期断言）：

```rust
#[test]
fn test_send() {
    fn assert_send<T: Send>() {}
    assert_send::<MyType>();
}

#[test]
fn test_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<MyType>();
}
```

## 转换 trait（C-CONV-TRAITS）

应实现的标准转换 trait：
- `From<T>`：当转换总是成功时
- `TryFrom<T>`：当转换可能失败时（自动获得 `Into` / `TryInto`）
- `AsRef<T>` / `AsMut<T>`：当需要借用转换时

不应直接实现 `Into<T>` 或 `TryInto<T>` —— 它们有基于 `From`/`TryFrom` 的 blanket impl。

```rust
// Good: From<u16> for u32（小整数转大整数总是成功）
impl From<u16> for u32 { /* ... */ }

// Good: TryFrom<u32> for u16（可能溢出）
impl TryFrom<u32> for u16 { /* ... */ }
```

## 集合 trait（C-COLLECT）

集合类型应实现：
- `FromIterator<T>`：从迭代器创建新集合
- `Extend<T>`：向现有集合添加元素

## 错误类型（C-GOOD-ERR）

- 错误类型必须实现 `std::error::Error`
- 错误类型应实现 `Send + Sync`
- 错误类型应实现 `Display`
- **绝不使用 `()` 作为错误类型**
- 优先使用具体错误类型（即使是 unit struct）
- `Display` 输出应小写、无尾随标点、简洁
- 不实现 `Error::description()`（已废弃，用 `Display` 替代）

```rust
// Bad
fn do_thing() -> Result<Wow, ()>

// Good
#[derive(Debug)]
struct DoError;
impl std::fmt::Display for DoError { /* ... */ }
impl std::error::Error for DoError { /* ... */ }
fn do_thing() -> Result<Wow, DoError>
```

错误消息示例：
- `"unexpected end of file"`
- `"invalid IP address syntax"`
- `"provided string was not \`true\` or \`false\`"`

## 文档规范

### Crate 级文档（C-CRATE-DOC）

- `lib.rs` 顶部使用 `//!` 写 crate 级文档
- 包含：一句话描述、使用示例、功能说明
- 见 [RFC 1687](https://rust-lang.github.io/rfcs/1687-doc-comments-for-traits.html)

### 所有公共项必须有示例（C-EXAMPLE）

- 每个公共 module、trait、struct、enum、function、method、macro 都应有 rustdoc 示例
- 示例应展示 "为什么用" 而非 "怎么用"
- 示例使用 `?` 而非 `unwrap` 或 `try!`（C-QUESTION-MARK）

```rust
/// 返回参数加 2 的结果。
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let result = my_crate::add_two(3)?;
/// assert_eq!(result, 5);
/// # Ok(())
/// # }
/// ```
pub fn add_two(n: i32) -> Result<i32, MyError> { /* ... */ }
```

### Errors / Panics / Safety 章节（C-FAILURE）

公共函数文档必须包含（如适用）：
- `# Errors`：说明可能返回的错误条件
- `# Panics`：说明 panic 条件
- `# Safety`：unsafe 函数的调用者不变式

### 超链接（C-LINK）

- 使用 `[`TypeName`]` 链接到类型，在文档末尾添加 `[`TypeName`]: <target>`
- 见 [RFC 1574](https://rust-lang.github.io/rfcs/1574-more-api-documentation-conventions.html)

## 泛型与灵活性（C-GENERIC / C-CALLER-CONTROL）

- 函数应最小化对参数的假设：用 `IntoIterator` 而非 `&Vec<T>` 或 `&[T]`
- 需要所有权时直接接收所有权，不要借入后 clone
- 不需要所有权时借入，不要接收所有权后丢弃

```rust
// Good: 最宽泛的约束
fn process<I: IntoIterator<Item = i64>>(iter: I) { /* ... */ }

// Bad: 过度约束
fn process(c: &Vec<i64>) { /* ... */ }
```

## 中间结果暴露（C-INTERMEDIATE）

函数在计算答案时产生的相关中间数据，如对用户有用则应暴露：
- `Vec::binary_search` 返回 `Result<usize, usize>`（找到或应插入的位置）
- `HashMap::insert` 返回 `Option<V>`（之前的值）

## Trait 对象安全（C-OBJECT）

- 可能被当作 trait object 使用的 trait 应保持对象安全
- 需要排除的方法加 `where Self: Sized` bound

```rust
trait MyTrait {
    fn object_safe(&self, i: i32);
    fn not_object_safe<T>(&self, t: T) where Self: Sized;
}
```
