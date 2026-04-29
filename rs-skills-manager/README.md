## rs-skills-manager

将仓库中的 `skills/` 目录下各 skill（子目录）通过软链接安装到不同平台/Agent 的配置目录中，避免复制带来的漂移。

### 配置

默认读取：`~/.config/rs-skills-manager/config.toml`

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

# 在仓库根目录运行：默认使用 ./skills 作为 skills 目录
# 安装指定技能到指定平台（-i 可重复）
rs-skills-manager -i software-engineer -i legal-counsel -o kimi

# 安装到配置中全部平台
rs-skills-manager -o all

# 不传 -o 时默认 all
rs-skills-manager
```

### 开发运行（cargo run）

```bash
# 注意：cargo run 需要用 `--` 分隔 cargo 参数与 CLI 参数
cargo run -- --help
cargo run -- -o all
```

### 行为说明

- 若目标目录不存在：自动创建
- 若链接不存在：创建软链接 `<target_dir>/<skill> -> <repo_skills_dir>/<skill>`
- 若已存在且为正确软链接：跳过并输出 `skipped`
- 若已存在但不是正确软链接（普通文件/目录/指向其他目标）：报错退出（避免误删用户文件）
