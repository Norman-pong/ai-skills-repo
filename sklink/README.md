## sklink

将仓库中的 `skills/` 目录下各 skill（子目录）通过软链接安装到不同平台/Agent 的配置目录中，避免复制带来的漂移。

### 术语与目录

- skill：一个目录（例如 `skills/software-engineer/`），目录名即 skill 名
- repo skills dir：仓库中的 `./skills/` 目录（或在 `skills/` 目录内运行时的当前目录）
- local store：本机持久化技能仓库，默认 `~/.config/sklink/skills`
- target dir：某个平台下的一个安装目标目录（来自 `config.toml`）

### 配置

默认读取：`~/.config/sklink/config.toml`

本机技能仓库（持久化存储）：`~/.config/sklink/skills`

初始化（推荐）：

```bash
sklink init
```

默认会生成如下平台配置：

```toml
[platforms.kimi]
targets = [
  { dir = "~/.kimi/skills" },
]

[platforms.trae]
targets = [
  { dir = "~/.trae/skills" },
]
```

配置结构（TOML）：

```toml
[platforms.<platform_name>]
targets = [
  { dir = "<target_dir>" },
]
```

- `<platform_name>`：平台名（`-p/--platform` 传入的值）
- `<target_dir>`：目标目录路径，支持 `~`，也支持相对路径（相对当前运行目录解析）

### 用法

```bash
# 初始化配置
sklink init

# 查看可用技能列表（优先从 ./skills 或当前 skills/ 目录发现；否则回退到本机技能仓库）
sklink list

# 查看已安装到哪里（读取 config.toml 中的 targets 并扫描目标目录）
sklink list --installed

# 生成补全脚本（以 zsh 为例）
sklink completions zsh > _sklink

# 在仓库根目录运行：默认使用 ./skills 作为 skills 目录
# 在 skills/ 目录内运行：默认使用当前目录作为 skills 目录
# 安装指定技能到指定平台（-i 可重复；既支持 skill 名，也支持路径）
sklink -i software-engineer -i legal-counsel -p kimi

# 使用长选项安装（等价于 -i）
sklink --install software-engineer -p kimi

# 也可以传路径（便于在任意目录运行；shell 展开后的 ./skills/* 也能工作）
sklink -i ./skills/software-engineer -p kimi

# 安装到配置中全部平台
sklink -p all

# 不传 -p 时默认 all
sklink

# 若本机技能仓库中已存在同名 skill，必须显式 --force 才允许覆盖（覆盖会先备份旧目录再写入）
sklink --force -i software-engineer -p kimi
```

### 安装模式（默认动作）

不带子命令时执行安装流程：

```bash
sklink [-i|--install SKILL|PATH ...] [-p|--platform PLATFORM|all] [--force]
```

- `-i, --install <SKILL|PATH>`：可重复
  - 传 skill 名：会在 local store / repo skills dir 中解析
  - 传路径：必须指向一个目录，skill 名取目录名
- `-p, --platform <PLATFORM|all>`：不传默认 `all`
- `--force`
  - 允许覆盖 local store 中的同名 skill（覆盖前会备份）
  - 影响部分解析优先级（见“skill 解析与优先级”）

### 开发运行（cargo run）

```bash
# 注意：cargo run 需要用 `--` 分隔 cargo 参数与 CLI 参数
cargo run -- --help
cargo run -- list
cargo run -- list --installed
cargo run -- -p all
```

### 行为说明

- skills 会先复制到 local store `~/.config/sklink/skills/<skill>`，再从 target dir 创建软链接
- 若 target dir 不存在或不可用：跳过并输出 warning（不会自动创建）
- 若 local store 中已存在同名 skill：
  - 批量安装（不传 `-i`）时：默认直接使用 store 版本（不报错），除非指定 `--force` 才会用当前来源覆盖 store
  - 指定安装（传 `-i`）时：默认报错；仅在指定 `--force` 时才允许覆盖（覆盖会先将旧目录备份到 `~/.config/sklink/backups/`）
- 若链接不存在：创建软链接 `<target_dir>/<skill> -> <local_store>/<skill>`
- 若已存在且为正确软链接：跳过并输出 `skipped`
- 若已存在但不是正确软链接（普通文件/目录/指向其他目标）：报错退出（避免误删用户文件）

### skill 解析与优先级

当 `-i` 传入的是“skill 名”（不是路径）时：

- 同名 skill 在 local store 存在：默认使用 local store
- local store 不存在但 repo skills dir 可用：使用 repo skills dir
- 同名 skill 同时在 local store 与 repo skills dir 存在：
  - 当前实现：带 `--force` 时优先 repo skills dir；不带 `--force` 时优先 local store

当 `-i` 传入的是“路径”时：

- 支持 `~`、`./relative/path`、包含 `/` 的路径
- 该路径必须是目录；skill 名取该目录的最后一段名称

### `list` 输出说明

- `sklink list`：输出可用 skill 名列表（每行一个）
- `sklink list --installed`：扫描配置中的 target dir，并按树形展示内容：
  - `[L]`：软链接
    - `ok`：软链接指向 local store 下的同名 skill
    - `outside-store`：软链接存在但不指向 local store 下的同名 skill
    - `unknown-store`：无法确定 local store（例如 canonicalize 失败）
    - `broken:<err>`：软链接损坏或无法解析
  - `[D]`：目录
  - `[F]`：普通文件
  - `[?]`：未知类型或读取失败

### 常见问题

- 为什么安装时提示 `warning: target dir not found (skipped)`？
  - 目标目录不存在或不可访问时会跳过，不会自动创建目录；请先手动创建并确保权限正确
- 为什么提示“已存在但不是正确软链接”就直接失败？
  - 这是刻意的安全边界：工具不会删除/覆盖用户已有文件或目录，避免误操作

### 平台与系统限制

- 当前使用 Unix 软链接实现（macOS/Linux）
