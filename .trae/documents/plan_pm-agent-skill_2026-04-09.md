---
title: 产品经理 Agent Skill 规划（调研后调整版）
date: 2026-04-09
mode: plan
---

## Summary

为本仓库新增一个“产品经理（PM）全流程教练”技能文档，参考市面上成熟的产品规划类 Skill 设计（多模式、框架化步骤、可交付物、质量门禁与可迭代的上下文累积），并按本仓库约定落地为 `skills/<skill-name>/skill.md`。默认交付物以“竞品与调研”模块为主，其他交付物（PRD/票据/指标/路线图）作为可选扩展。

## Current State Analysis（基于仓库实情）

- 现有技能文档位于 `skills/<name>/skill.md`：
  - [skill.md（stock-analysis）](file:///c:/repo/zhimingcool/chore/skills/stock-analysis/skill.md)：使用 YAML frontmatter（含 name/display_name/version/language/description/tags）+ 工作流/模板/证据来源规则。
  - [skill.md（novel-craft）](file:///c:/repo/zhimingcool/chore/skills/novel-craft/skill.md)：同样是“流程化 + 产物模板 + 质量门禁”的风格，但文件分隔符与字段转义不完全统一。
- 仓库目前无 `.trae/` 目录；本计划文件已在 `.trae/documents/` 下创建，后续执行阶段会新增技能目录与技能文件。

## Market Research Snapshot（用于对标的成熟形态）

重点参考两类“成熟做法”并吸收其可迁移能力：

1) 多模式 + 框架库 + 开发交接思路（以 Product Playbook 为代表）
- 特征：提供“快速/完整/改版/功能扩展/直接实作”等模式；覆盖 JTBD、RICE、PR-FAQ、Pre-mortem、North Star 等框架；强调“变更传播”和“交接包产出”。来源示例：Product Playbook README（GitHub）https://github.com/Kaminoikari/product-playbook/blob/main/README.zh-TW.md

2) “人出粗稿 → 模型做结构化整理 → 人筛选 → 再扩写”的 PRD 工作法（偏现实可用）
- 特征：不要求模型从 0 到 1 写完 PRD；更适合做结构优化、补齐缺漏、统一风格、生成 Markdown 交付。来源示例：GitHub Issue 经验总结 https://github.com/1024XEngineer/CialloClaw/issues/14

本技能会把上述两点落实为：多模式流程（默认“教练式提问”）+ 强制证据/假设/风险/验收口径 + 可选交接产物。

## Proposed Changes（执行阶段将做的具体改动）

### 1) 新增技能文档目录

- 新增目录：`skills/product-manager/`
- 新增文件：`skills/product-manager/skill.md`
- 命名选择：
  - `name`: `product-manager`
  - `display_name`: `product-manager（产品经理全流程教练）`
  - `language`: `zh-CN`
  - `version`: `1.0.0`
  - `description`：必须包含“做什么 + 什么时候触发/使用”，并覆盖用户常见提法（如：产品规划、竞品分析、PRD、需求拆解、路线图、指标等）

### 2) 技能内容结构（对齐仓库既有风格）

`skills/product-manager/skill.md` 拟采用与 stock-analysis 类似的结构化骨架：

- 使用前提与边界（合规/事实性/不编造数据与引用）
- 输入（对话契约）：最小必要信息 + 推荐补充信息
- 模式（Mode）：
  - 快速模式：30–60 分钟产出“机会描述 + 初步竞品表 + 风险假设清单 + 下一步调研计划”
  - 完整模式：覆盖 Discovery → Define → Develop → Deliver 的全流程步骤（但默认输出仍聚焦调研产物）
  - 改版模式：已有产品/功能的诊断、问题树与迭代方案
  - 功能扩展模式：在既有产品上新增单一功能的范围界定与验收标准
- 默认产物（用户已选择“竞品与调研”为默认）：
  - `research.md`：竞品清单、定位/差异点、功能矩阵、定价与商业模式对比、用户口碑与渠道线索、机会洞察、假设清单
- 可选扩展产物（按用户指令再生成，不强制默认）：
  - `PRD.md`（含用户故事与验收标准）
  - `tickets.md`（可开票）
  - `metrics.md`（北极星/输入指标/埋点口径）
  - `roadmap.md`（里程碑）
- 质量门禁（Hard Gate）：
  - 关键断言必须标注来源或标记“待核验”
  - 对每个核心结论至少给出 1 条反例/失败路径（Pre-mortem）
  - 输出必须包含“假设清单 + 待验证计划”

### 3) 竞品调研产物模板（research.md）

在技能中内置可复制的 Markdown 模板（执行阶段会写入技能文件），至少包括：

- 竞品表（名称/市场/人群/定位/定价/核心功能/差异点/证据链接）
- 功能矩阵（功能点 × 竞品，标注支持程度与证据）
- 定位与信息传达（首页一句话、价值主张、关键用词）
- 商业化与增长线索（定价、套餐、获客渠道、内容策略）
- 风险与不确定性（信息质量分级：官方 > 权威媒体 > 社媒线索）
- 机会洞察与建议（HMW 问题、机会树节点）

## Assumptions & Decisions（已根据本次对话锁定）

- 覆盖范围选择：全流程规划（而非仅 PRD 写作或仅增长）。
- 默认交互风格：教练式提问（先补齐信息与决策收敛，再产出文档）。
- 默认产物：竞品与调研（research.md）；其他产物作为可选扩展。
- 不引入外部 API 集成；信息获取仅通过公开网络检索/页面抓取（并遵守引用与合规边界）。

## Verification（执行阶段的验收方式）

- 结构验收：新技能 `skills/product-manager/skill.md` 具备完整 YAML frontmatter 字段，且可读性与现有技能一致。
- 触发验收：description 中包含明确“何时使用”的触发语句与典型用户问法。
- 产物验收：research 模板可直接复制使用；包含来源要求、反例认证与假设/验证计划。
- 一致性验收：不引入与仓库现有技能冲突的命名/格式约定（优先对齐 stock-analysis 的 frontmatter 结构）。

