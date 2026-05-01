# 发布脚本

## sklink 发布到 crates.io

### 前置条件

- 已完成登录：`cargo login`
- 当前 git 工作区干净（无未提交变更）

### 用法

在仓库根目录执行：

```bash
scripts/release-sklink patch
scripts/release-sklink minor
scripts/release-sklink major
```

预演（不修改、不提交、不打 tag、不发布）：

```bash
scripts/release-sklink patch --dry-run
```

### 脚本做了什么

- 自动对 `sklink/Cargo.toml` 的版本号做 SemVer bump
- 执行回归（在 `sklink/` 下）：
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`
- 生成一条 Lore commit（只提交 `sklink/Cargo.toml`）
- 打 tag：`vX.Y.Z`
- `cargo publish` 发布到 crates.io
- 最后提示手动执行：
  - `git push`
  - `git push --tags`

### 常见失败

- `cargo publish` 提示未登录：先执行 `cargo login`
- `cargo publish` 提示版本已存在：说明 bump 失败或你已发布过该版本；检查 `sklink/Cargo.toml` 的 version 与 tag 是否一致

