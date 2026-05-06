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

# 安装 skill 到本机技能仓库（local store）
# - 在仓库根目录运行：可用 skill 名（从 ./skills/<skill> 解析）
# - 在 skills/ 目录内运行：可用 skill 名（从 ./<skill> 解析）
sklink -i software-engineer -i legal-counsel

# 也可以传路径（shell 展开后的 ./skills/* 也能工作）
sklink -i ./skills/software-engineer -i ./skills/legal-counsel

# 从 GitHub 仓库安装（优先提取 repo/skills/*；否则 repo 根目录视为单个 skill）
sklink -i https://github.com/org/repo

# 同步本机技能仓库到平台目录（读取 config.toml）
sklink --async -p all
sklink --async -p kimi

# 安装后立即同步到平台（install + async 可组合）
sklink -i software-engineer --async -p kimi

# 输出 skill 到项目目录（默认 .agent/skills/，会自动创建目录）
sklink -o software-engineer

# 指定输出目录
sklink -o software-engineer --dir .agent/skills

# 导出（复制）而不是软链
sklink -o software-engineer --export
```

### 模式

#### Install（安装到本机技能仓库）

```bash
sklink -i|--install <SRC>... [--force] [--async [-p|--platform <PLATFORM|all>]]
```

- `<SRC>`：skill 名、目录路径（支持 `./`、shell 展开）、或 Git 仓库 URL
- `--force`：允许覆盖 local store 中同名 skill（覆盖前会备份）
- `--async`：安装完成后立即同步到平台目录（见下）

#### Sync（同步到平台目录）

```bash
sklink --async [-p|--platform <PLATFORM|all>]
```

- `-p/--platform` 仅在 `--async` 模式下有效

#### Output（输出到项目目录）

```bash
sklink -o|--output <SKILL>... [--dir <DIR>] [--export]
```

- `--dir` 默认 `.agent/skills/`（会自动创建）
- `--export`：复制目录而不是创建软链

### 开发运行（cargo run）

```bash
# 注意：cargo run 需要用 `--` 分隔 cargo 参数与 CLI 参数
cargo run -- --help
cargo run -- list
cargo run -- list --installed
cargo run -- --async -p all
```

### 行为说明

- `-i` 会将来源目录复制到 local store `~/.config/sklink/skills/<skill>`
  - 若 local store 中已存在同名 skill：默认报错；仅 `--force` 才允许覆盖（覆盖会先将旧目录备份到 `~/.config/sklink/backups/`）
- `--async` 会从 local store 同步软链接到 config 指定的 target dir：
  - 若 target dir 不存在或不可用：跳过并输出 warning（不会自动创建）
  - 若链接不存在：创建软链接 `<target_dir>/<skill> -> <local_store>/<skill>`
  - 若已存在且为正确软链接：跳过并输出 `skipped`
  - 若已存在但不是正确软链接（普通文件/目录/指向其他目标）：报错退出（避免误删用户文件）
- `-o/--output` 默认创建软链 `<out_dir>/<skill> -> <local_store>/<skill>`
  - `--export` 改为复制目录内容

### skill 解析与优先级

当 `-i` 传入的是“skill 名”（不是路径）时：

- 会从当前运行目录推导 repo skills dir（`./skills` 或当前 `skills/` 目录）
- 仅作为“安装来源”使用 `repo skills dir`；local store 是安装目标，不参与来源优先级选择

当 `-i` 传入的是“路径”时：

- 支持 `~`、`./relative/path`、包含 `/` 的路径
- 该路径必须是目录；skill 名取该目录的最后一段名称

当 `-i` 传入的是“Git 仓库 URL”时：

- 若仓库内存在 `skills/`：提取 `skills/*` 作为多个 skill
- 否则：将仓库根目录视作单个 skill（skill 名默认为仓库名）

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

### 发布

发布到 crates.io 的流程已固化为脚本，见 [scripts/README.md](file:///Users/norman/workspace/zhimingcool/chore/scripts/README.md)
