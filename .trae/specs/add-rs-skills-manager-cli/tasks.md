# Tasks
- [x] Task 1: 初始化 Rust CLI 工程
  - [x] 创建新的 Rust binary crate（名称：`rs-skills-manager`），并加入到仓库中
  - [x] 定义基础命令行参数：`-i`（可重复）与 `-o`（platform|all）

- [x] Task 2: 实现配置读取与解析
  - [x] 从 `~/.config/rs-skills-manager/config.toml` 加载并解析 TOML
  - [x] 实现 `~` 展开与路径规范化
  - [x] 实现当 `repo_skills_dir` 缺省时的默认解析（`./skills`）

- [x] Task 3: 实现 skills 发现与选择
  - [x] 当未传 `-i`：扫描 `repo_skills_dir` 下的子目录名作为 skills 列表
  - [x] 当传入 `-i`：校验 skill 存在，否则返回非 0

- [x] Task 4: 实现软链接安装逻辑
  - [x] 对 `-o all` 遍历全部平台；否则只处理指定平台
  - [x] 针对每个 `targets[].dir` 创建目录并创建软链接 `<dir>/<skill> -> <repo_skills_dir>/<skill>`
  - [x] 幂等：正确链接跳过；冲突（非正确软链接/非软链接）报错退出
  - [x] 输出每项结果（created/skipped/error）

- [x] Task 5: 验证与回归
  - [x] 添加单元测试（镜像风格）：配置解析、路径展开、skills 发现
  - [x] 补全边界/异常测试：缺失配置、平台不存在、skill 不存在、目标路径冲突、软链接指向不一致等
  - [x] 添加集成测试：临时目录下创建目标目录并验证软链接行为（macOS/Linux）
  - [x] 确保实现风格参考官方 Rust API Guidelines（公开 API、错误类型、命名与可用性）
  - [x] 增加基本使用说明（README 或 `--help` 文案）

# Task Dependencies
- Task 2 depends on Task 1
- Task 3 depends on Task 2
- Task 4 depends on Task 2, Task 3
- Task 5 depends on Task 4
