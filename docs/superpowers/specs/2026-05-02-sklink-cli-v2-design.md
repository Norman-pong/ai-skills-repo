---
title: sklink CLI v2 设计（破坏兼容）
date: 2026-05-02
scope: sklink
---

## 背景与目标

sklink 是一个将 skills 安装到平台/Agent 目标目录的工具：先把 skill 复制到本机 local store，再在目标目录创建软链接。

本次改动目标：

- 调整参数命名以符合更常见的 Unix CLI 习惯（破坏兼容）
- 当用户显式指定的平台不存在时返回错误（非 0 退出），增强脚本可靠性
- 为安装模式增加 `-i` 的长选项
- 增加 tab 补全能力，并提供更明确的失败提示（reminder）

## 非目标

- 不增加 stdin 管道输入、json 输出、quiet 模式等额外功能
- 不增加 sub-minute 调度或自动化能力
- 不改变安装逻辑的核心安全边界（不覆盖用户已有普通文件/目录）

## CLI 形态（v2）

### 总体规则

- 保持现有“默认动作 = 安装”不变：不带子命令时执行安装流程
- 继续使用 `clap` derive 生成 `--help`
- 本次允许破坏兼容：移除 `-o`，以 `-p/--platform` 作为唯一平台选择参数

### 命令与选项

#### 安装（默认动作）

```
sklink [--install <SKILL|PATH> ...] [--platform <PLATFORM|all>] [--force]
```

- `-i, --install <SKILL|PATH>`：可重复
  - 传 skill 名：在 repo skills dir / local store 中解析
  - 传路径：必须指向目录，skill 名取目录名
- `-p, --platform <PLATFORM|all>`：不传默认 `all`
- `--force`：允许覆盖 local store 中同名 skill（覆盖前备份）

兼容性变更：

- 移除 `-o`（原 short option），不再兼容旧用法

#### init

```
sklink init [--force]
```

#### list

```
sklink list [--installed]
```

#### completions（新增）

```
sklink completions <SHELL>
```

- 通过 stdout 输出对应 shell 的补全脚本
- 支持 shell 范围：bash/zsh/fish/powershell/elvish（以 `clap_complete` 支持为准）

## 行为与语义

### 平台不存在的语义（关键变更）

当用户通过 `-p/--platform` 显式指定一个平台名，且 config 中不存在该平台：

- 打印错误（stderr）
- 退出码非 0（建议为 1）
- 错误信息需包含：
  - 不存在的平台名
  - 可用平台名列表（若能读取 config 且存在 platforms）

当 `--platform all`：

- 遍历 config 中所有平台执行安装

### 输出与可脚本化约束

- stdout：只输出“可消费的结果信息”
  - 安装：`created <link> -> <target>` / `skipped <link>`
  - `list`：每行一个 skill 名
  - `completions`：补全脚本内容
- stderr：warning / error

### 提醒（reminder）与错误信息（UX）

本次“提醒”收敛在两类高频失败：

1) 配置缺失/不可读/解析失败

- 在现有错误信息基础上，追加一行行动建议：
  - `hint: run 'sklink init' to generate a default config`

2) 平台不存在

- 错误信息追加：
  - `available platforms: <a>, <b>, ...`

说明：

- hint 只在错误场景输出，不在成功路径增加噪音
- hint 走 stderr

## Tab 补全设计

### 交付方式

通过 `sklink completions <shell>` 生成补全脚本：

- 生成动作由程序完成，用户按各 shell 的惯例安装
- 文档中给出示例（例如：`sklink completions zsh > _sklink`）

### 依赖

需要引入 `clap_complete` 作为 sklink 的运行时依赖（与当前 clap 版本匹配）。

## 测试策略（回归锁定）

- 更新/新增集成测试（sklink/tests）覆盖：
  - `-p/--platform` 安装成功路径
  - 平台不存在时退出码非 0，stderr 包含 “platform not found” 与 available platforms
  - `--install` 与 `-i` 行为一致
  - `completions zsh` 输出非空且包含 `sklink` 关键字

## 变更清单（实现层面）

- 更新 clap 参数定义：
  - `-o` 删除
  - 新增 `-p/--platform`
  - `-i` 增加 `--install`
  - 新增子命令 `completions`
- 调整平台选择逻辑：
  - 显式平台不存在时返回错误
- 统一错误输出增加 hint：
  - 针对 config 相关错误与 platform not found 错误
- 更新 README 用法示例

