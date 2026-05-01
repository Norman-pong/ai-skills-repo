# sklink CLI Spec

## Why
需要一个统一的 CLI，将本仓库 `skills/` 下的技能以软链接方式分发到不同 Agent/平台的配置目录中，避免手动复制与维护不一致。

## What Changes
- 新增 Rust CLI：`sklink`
- 读取配置表：`~/.config/sklink/config.toml`
- 根据参数 `-i`（指定多个 skill）与 `-o`（指定平台名或 `all`）执行软链接创建/更新
- 不会自动创建目标目录；当目标目录不可用时跳过并输出 warning
- 支持幂等重复执行（已正确链接则跳过）

## Impact
- Affected specs: skills 分发/安装流程
- Affected code: 新增 Rust crate（CLI）；不修改现有 `skills/*` 内容

## ADDED Requirements

### Requirement: Rust 编写规范
系统 SHALL 遵循官方 Rust API Guidelines（公开 API 命名、错误处理、文档/可用性等）。

#### 测试规范
- 单元测试 SHALL 采用“镜像风格”：测试模块结构与 `src/` 下模块结构保持一致（便于定位与覆盖）。
- 边界与异常场景 SHALL 补全测试覆盖（例如：缺失配置、平台不存在、skill 不存在、目标路径冲突、软链接指向不一致等）。

### Requirement: 配置表加载
系统 SHALL 从默认路径加载配置：`~/.config/sklink/config.toml`。

#### 配置格式（TOML）
```toml
[platforms.cursor]
targets = [
  # 在该目录下为每个 skill 创建一个同名软链接
  { dir = "~/.config/cursor/skills" },
]

[platforms.trae]
targets = [
  { dir = "~/.config/trae/skills" },
]
```

#### 约束
- `~` 在路径中 SHALL 展开为用户 HOME。
- `platforms.<name>.targets[].dir` 表示目标“skills 目录”（工具在该目录下创建软链接项）。
- repo skills dir 的发现规则：
  - 在仓库根目录运行：使用 `./skills`
  - 在 `skills/` 目录内运行：使用当前目录
  - 若无法发现 repo skills dir：回退到 local store

---

### Requirement: CLI 参数
系统 SHALL 提供如下参数：
- `-i <skill_name>`：可重复传入，用于选择要安装的 skill；支持多个 `-i`
- `-o <platform_name|all>`：选择目标平台；`all` 表示配置表内全部平台

#### 默认行为
- 当未传 `-i` 时，系统 SHALL 默认选择发现到的 skills 目录下的全部 skills（子目录名即 skill 名）。

#### Scenario: 指定技能 + 指定平台
- **WHEN** 用户执行 `sklink -i software-engineer -i legal-counsel -o cursor`
- **THEN** 工具在配置的 `platforms.cursor.targets[*].dir` 下为每个 skill 创建/更新软链接

#### Scenario: 全量平台
- **WHEN** 用户执行 `sklink -o all`
- **THEN** 工具对配置表内所有平台执行同样的链接逻辑

---

### Requirement: 软链接创建策略
系统 SHALL 针对每个目标 `dir` 与每个选中的 `skill` 创建如下软链接（链接目标为 local store 中对应 skill 目录）：
- link_path: `<target_dir>/<skill>`
- link_target: `~/.config/sklink/skills/<skill>`

#### 行为细则
- 若 `target_dir` 不存在或不可用：SHALL 跳过并输出 warning（不会自动创建）。
- 若 `link_path` 不存在：SHALL 创建软链接。
- 若 `link_path` 已存在且是正确的软链接（指向 `link_target`）：SHALL 跳过。
- 若 `link_path` 已存在但不是正确软链接（包括指向其他位置、为普通文件、为目录等）：SHALL 报错并返回非 0 退出码（避免误删用户文件）。

#### local store 处理
- 工具 SHALL 先将 skill 复制到 local store，再创建上述软链接。

#### 输出
- 工具 SHALL 在 stdout 输出每个创建/跳过/失败的项（简短、可读）。

## MODIFIED Requirements
无

## REMOVED Requirements
无
