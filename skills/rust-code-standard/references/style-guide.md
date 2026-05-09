# Rust 编码风格规范

本文件汇总 Rust 官方编码风格规范。

## 来源

- [RFC 430: Rust Code Style](https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html)
- [rustfmt 默认配置](https://rust-lang.github.io/rustfmt/)
- [Rust API Guidelines: Naming](https://rust-lang.github.io/api-guidelines/naming.html)

## 目录

- [命名约定（RFC 430 / C-CASE）](#命名约定rfc-430--c-case)
- [转换方法命名（C-CONV）](#转换方法命名c-conv)
- [Getter 命名（C-GETTER）](#getter-命名c-getter)
- [迭代器方法命名（C-ITER）](#迭代器方法命名c-iter)
- [迭代器类型命名（C-ITER-TY）](#迭代器类型命名c-iter-ty)
- [Feature 命名（C-FEATURE）](#feature-命名c-feature)
- [命名一致性（C-WORD-ORDER）](#命名一致性c-word-order)
- [rustfmt 配置要点](#rustfmt-配置要点)

---

## 命名约定（RFC 430 / C-CASE）

| 项目 | 约定 |
|---|---|
| Crates | `snake_case` |
| 模块 | `snake_case` |
| 类型（struct, enum, trait, type alias） | `UpperCamelCase` |
| Enum 变体 | `UpperCamelCase` |
| 函数、方法 | `snake_case` |
| General constructors | `new` or `with_more_details` |
| Conversion constructors | `from_some_other_type` |
| 宏 | `snake_case!` |
| 局部变量 | `snake_case` |
| 常量 | `SCREAMING_SNAKE_CASE` |
| 静态变量 | `SCREAMING_SNAKE_CASE` |
| 类型参数 | 简洁的 `UpperCamelCase`，通常单字母：`T`, `U`, `K`, `V` |
| 生命周期 | 短小的 lowercase，通常单字母：`'a`, `'de`, `'src` |

### 命名细则

- 在 `UpperCamelCase` 中，缩写和复合词收缩视为一个单词：用 `Uuid` 而非 `UUID`，`Usize` 而非 `USize`，`Stdin` 而非 `StdIn`
- 在 `snake_case` 中，缩写和复合词收缩小写：`is_xid_start`
- 在 `snake_case` 或 `SCREAMING_SNAKE_CASE` 中，单词不应由单个字母组成（除非是最后一个词）：`btree_map` 而非 `b_tree_map`，但 `PI_2` 而非 `PI2`
- crate 名不应使用 `-rs` 或 `-rust` 前缀/后缀
- Cargo feature 名不使用无意义占位词：`std` 而非 `use-std` 或 `with-std`

## 转换方法命名（C-CONV）

| 前缀 | 开销 | 所有权变化 |
|---|---|---|
| `as_` | 零开销 | borrowed → borrowed |
| `to_` | 有开销 | borrowed → borrowed / borrowed → owned (non-Copy types) / owned → owned (Copy types) |
| `into_` | 可变开销 | owned → owned (non-Copy types) |

- 转换前缀 `as_` 和 `into_` 通常降低抽象层次（暴露视图或解构数据）
- 转换前缀 `to_` 通常保持在同一抽象层次，但改变表示形式
- 包装单一值的类型暴露内部值用 `into_inner()`（如 `BufReader::into_inner`）
- `mut` 修饰符出现在返回值类型中的位置：`as_mut_slice` 而非 `as_slice_mut`

示例：
- `str::as_bytes()` — 零开销，borrowed → borrowed
- `Path::to_str` — 昂贵的 UTF-8 校验，borrowed → borrowed
- `str::to_lowercase()` — Unicode 转换，borrowed → owned (non-Copy)
- `f64::to_radians()` — Copy 类型，owned → owned
- `String::into_bytes()` — 零开销，owned → owned (non-Copy)

## Getter 命名（C-GETTER）

- **不使用 `get_` 前缀**（少数例外如 `Cell::get`）
- 直接以字段名命名：`first()` 而非 `get_first()`，`first_mut()` 而非 `get_first_mut()`
- 运行时校验的 getter 考虑添加 `unsafe _unchecked` 变体

```rust
pub fn get(&self, index: K) -> Option<&V>;
pub fn get_mut(&mut self, index: K) -> Option<&mut V>;
unsafe fn get_unchecked(&self, index: K) -> &V;
unsafe fn get_unchecked_mut(&mut self, index: K) -> &mut V;
```

## 迭代器方法命名（C-ITER）

遵循 RFC 199：

```rust
fn iter(&self) -> Iter       // Iter implements Iterator<Item = &U>
fn iter_mut(&mut self) -> IterMut  // IterMut implements Iterator<Item = &mut U>
fn into_iter(self) -> IntoIter     // IntoIter implements Iterator<Item = U>
```

此约定适用于概念上同质的集合。非同质集合（如 `str`）使用专门的方法（`bytes()`, `chars()`）。

## 迭代器类型命名（C-ITER-TY）

迭代器方法的返回类型名称应与方法对应：

| 方法 | 返回类型 |
|---|---|
| `iter()` | `Iter` |
| `iter_mut()` | `IterMut` |
| `into_iter()` | `IntoIter` |
| `keys()` | `Keys` |
| `values()` | `Values` |

这些类型名通常以模块为前缀使用，如 `vec::IntoIter`。

## Feature 命名（C-FEATURE）

- Cargo feature 名不使用无意义占位词：用 `abc` 而非 `use-abc` 或 `with-abc`
- Feature 必须是加法性的（additive），绝不用否定命名如 `no-abc`

```toml
# Good
[features]
default = ["std"]
std = []

# Bad
use-std = []
with-std = []
no-std = []  # 否定命名，违反加法性
```

## 命名一致性（C-WORD-ORDER）

错误类型等复合名词应在 crate 内部保持一致的词序。标准库中常见的顺序有 **verb-object-error**（如 `ParseBoolError`、`JoinPathsError`）和 **object-verb-error**（如 `AddrParseError`）两种，关键是与 crate 内已有类型及标准库中相似功能保持一致。

## rustfmt 配置要点

项目使用默认 rustfmt 配置。关键行为：
- 缩进：4 空格
- 最大行宽：100（默认）
- `use` 语句合并与排序由 rustfmt 自动处理
- 尾随逗号：根据结构自动添加
- 链式调用：多行时每个方法一行

运行 `cargo fmt --check` 验证，运行 `cargo fmt` 自动修复。
