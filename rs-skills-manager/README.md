## rs-skills-manager

将仓库中的 `skills/` 目录下各 skill（子目录）通过软链接安装到不同平台/Agent 的配置目录中，避免复制带来的漂移。

### 配置

默认读取：`~/.config/rs-skills-manager/config.toml`

本机技能仓库（持久化存储）：`~/.config/rs-skills-manager/skills`

初始化（推荐）：

```bash
rs-skills-manager init
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

### 用法

```bash
# 初始化配置
rs-skills-manager init

# 查看可用技能列表（优先从 ./skills 或当前 skills/ 目录发现；否则回退到本机技能仓库）
rs-skills-manager list

# 查看已安装到哪里（读取 config.toml 中的 targets 并扫描目标目录）
rs-skills-manager list --installed

# 在仓库根目录运行：默认使用 ./skills 作为 skills 目录
# 在 skills/ 目录内运行：默认使用当前目录作为 skills 目录
# 安装指定技能到指定平台（-i 可重复；既支持 skill 名，也支持路径）
rs-skills-manager -i software-engineer -i legal-counsel -o kimi

# 也可以传路径（便于在任意目录运行；shell 展开后的 ./skills/* 也能工作）
rs-skills-manager -i ./skills/software-engineer -o kimi

# 安装到配置中全部平台
rs-skills-manager -o all

# 不传 -o 时默认 all
rs-skills-manager

# 若本机技能仓库中已存在同名 skill，必须显式 --force 才允许覆盖（覆盖会先备份旧目录再写入）
rs-skills-manager --force -i software-engineer -o kimi
```

### 开发运行（cargo run）

```bash
# 注意：cargo run 需要用 `--` 分隔 cargo 参数与 CLI 参数
cargo run -- --help
cargo run -- list
cargo run -- list --installed
cargo run -- -o all
```

### 行为说明

- skills 会先复制到本机技能仓库 `~/.config/rs-skills-manager/skills/<skill>`，再从目标目录创建软链接
- 若目标目录不存在或不可用：跳过并输出 warning（不会自动创建）
- 若本机技能仓库中已存在同名 skill：默认报错；仅在指定 `--force` 时才允许覆盖（覆盖会先将旧目录备份到 `~/.config/rs-skills-manager/backups/`）
- 若链接不存在：创建软链接 `<target_dir>/<skill> -> <local_store>/<skill>`
- 若已存在且为正确软链接：跳过并输出 `skipped`
- 若已存在但不是正确软链接（普通文件/目录/指向其他目标）：报错退出（避免误删用户文件）
