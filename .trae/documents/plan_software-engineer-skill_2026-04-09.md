---
title: 程序员 Agent Skill 规划（增量瀑布 + 强制门禁）
date: 2026-04-09
mode: plan
---

## Summary

新增一个“程序员/软件工程师（Software Engineer）”agent 技能文档，严格按软件工程流程（增量瀑布模型）组织对话与交付物，并通过强制门禁约束“未完成上游产物不得进入下游阶段（尤其不得先写代码）”。默认产出覆盖：需求与验收、架构与设计、计划与任务、测试与质量四类文档。

## Current State Analysis（基于仓库实情）

- 仓库技能文档约定：`skills/<skill-name>/skill.md`，顶部使用 YAML frontmatter（`name/display_name/version/language/description/tags`）。<mccoremem id="03fwvxg6398g50a14wtu3bov7" />
- 已有技能采用“流程化 + 模板化 + 门禁/红线”的写法，可对齐其结构与表达：
  - [skill.md（product-manager）](file:///c:/repo/zhimingcool/chore/skills/product-manager/skill.md)：教练式提问 + 多模式 + 默认产物模板。
  - [skill.md（legal-counsel）](file:///c:/repo/zhimingcool/chore/skills/legal-counsel/skill.md)：Hard Gate 门禁写法与强制提示。
  - [skill.md（novel-craft）](file:///c:/repo/zhimingcool/chore/skills/novel-craft/skill.md)：工作流与打回标准示例。

## Intent & Decisions（已锁定）

- 过程模型：增量瀑布（每个增量完整走一遍：需求→设计→实现→测试→验收→回顾）。
- 默认交付物：需求与验收、架构与设计、计划与任务、测试与质量。
- 门禁策略：强制门禁（缺关键产物就继续补齐，不允许直接进入编码/实现）。

## Proposed Changes（执行阶段将做的具体改动）

### 1) 新增技能目录与文件

- 新增目录：`skills/software-engineer/`
- 新增文件：`skills/software-engineer/skill.md`
- YAML frontmatter（对齐仓库约定）：
  - `name`: `software-engineer`
  - `display_name`: `software-engineer（程序员：增量瀑布工程交付）`
  - `version`: `1.0.0`
  - `language`: `zh-CN`
  - `description`：必须覆盖“做什么 + 什么时候调用”，触发语包括：需求拆解、设计方案、实现计划、测试计划、代码实现、修Bug、上线交付、工程规范等。
  - `tags`：`engineering`、`waterfall`、`quality-gate`、`testing`、`architecture`、`tool` 等。

### 2) 技能主体结构（增量瀑布 + 强制门禁）

`skills/software-engineer/skill.md` 采用可执行流程与模板：

1. 使用前提与边界
   - 以交付与质量为目标，禁止“未定义验收标准就开工编码”
   - 不臆造依赖/库/基础设施；若需要使用某库，必须先确认仓库已使用或明确引入计划
   - 安全红线：不输出/不记录密钥；不引入明显不安全方案

2. 增量瀑布模型（核心）
   - 定义“增量（Increment）”的边界：每次只交付可验收的一小块
   - 每个增量的阶段序列：
     - Phase 1 需求与验收（Requirements）
     - Phase 2 架构与设计（Design）
     - Phase 3 实现（Implementation）
     - Phase 4 测试（Testing）
     - Phase 5 验收与发布（Acceptance/Release）
     - Phase 6 回顾与变更记录（Postmortem/Change Log）

3. 强制门禁（Hard Gates）
   - Gate R：未完成“范围/非目标/验收标准/边界条件”不得进入设计与实现
   - Gate D：未完成“接口/数据模型/关键流程/失败模式”不得进入实现
   - Gate I：未完成“测试计划+至少关键用例清单”不得进入验收
   - Gate A：未完成“验收记录/回滚方案/风险清单”不得进入发布

4. 变更控制（适配瀑布）
   - 需求变更必须走“变更请求（CR）”模板：变更原因、影响范围、回归测试、排期影响
   - 维护可追溯性：需求 → 设计 → 任务 → 测试用例 → 验收项 的映射表

### 3) 默认产物模板（四类文档）

在 skill 中内置可复制模板（执行阶段写入 skill.md），默认生成以下文件内容骨架：

- `requirements.md`
  - 背景/目标、范围/非目标、用户故事、约束、边界条件、验收标准、风险与假设
- `architecture.md` / `api_design.md` / `data_model.md`
  - 组件图/模块边界、接口契约、数据模型、关键流程、错误处理与降级
- `plan.md` / `tasks.md`
  - 增量划分、里程碑、依赖、风险、任务拆解与验收映射（traceability）
- `test_plan.md` / `test_cases.md` / `quality_gate.md`
  - 测试范围与策略、关键用例、边界与失败用例、质量门禁清单

### 4) 输出格式与交互风格

- 默认教练式推进：先补齐关键信息，再产出文档，再进入实现
- 任何跳阶段请求：拒绝直接跳过门禁，但可提供“缺口清单 + 需要用户确认的最小信息”

## Verification（执行阶段验收方式）

- 结构验收：`skills/software-engineer/skill.md` frontmatter 字段完整且与仓库现有技能一致。
- 流程验收：文档中明确“增量瀑布阶段序列”与“强制门禁”条款，且可执行（有检查项）。
- 模板验收：四类默认产物模板可直接复制使用，且包含需求-设计-测试-验收的可追溯字段。
- 安全验收：规则中包含不记录密钥、不引入不安全方案的约束。

