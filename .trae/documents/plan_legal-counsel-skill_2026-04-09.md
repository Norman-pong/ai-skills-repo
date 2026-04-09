---
title: 法律顾问 Agent Skill 规划（严谨引用版）
date: 2026-04-09
mode: plan
---

## Summary

新增一个“法律顾问（Legal Counsel）”agent 技能文档，遵循本仓库 `skills/<skill-name>/skill.md` 约定与既有写法（流程化 + 模板化 + 质量门禁）。该技能默认面向中国大陆法域，以“通用法律咨询”场景为主，核心要求是：任何法律条文/司法解释/规范性文件引用必须可核验且有明确出处；无法核验则不引用，并明确提示用户应咨询执业律师获取正式法律意见。

## Current State Analysis（基于仓库实情）

- 技能文档目录约定：`skills/<skill-name>/skill.md`，顶部使用 YAML frontmatter（`name/display_name/version/language/description/tags`）。<mccoremem id="03fwvxg6398g50a14wtu3bov7" />
- 已存在 3 个技能示例可对齐格式与风格：
  - [skill.md（stock-analysis）](file:///c:/repo/zhimingcool/chore/skills/stock-analysis/skill.md)：强调证据链与反例认证，适合复用“来源门禁”结构。
  - [skill.md（product-manager）](file:///c:/repo/zhimingcool/chore/skills/product-manager/skill.md)：教练式提问 + 多模式 + 模板化产物，适合复用“上下文引导/Hard Gate”设计。
  - [skill.md（novel-craft）](file:///c:/repo/zhimingcool/chore/skills/novel-craft/skill.md)：流程化与质量门禁示例。

## Intent & Decisions（已锁定）

- 法域：默认中国法（大陆）。
- 场景：通用法律咨询（先分流案件类型与风险分级，复杂/高风险强制建议转人工律师）。
- 引用门槛：无法核验就不引用；不确定则忽略并提示用户咨询律师。
- 风格：严谨、结构化、避免口语化与夸大确定性。

## Proposed Changes（执行阶段将做的具体改动）

### 1) 新增技能目录与文件

- 新增目录：`skills/legal-counsel/`
- 新增文件：`skills/legal-counsel/skill.md`
- YAML frontmatter（对齐仓库约定）：
  - `name`: `legal-counsel`
  - `display_name`: `legal-counsel（法律顾问：严谨引用与风险分级）`
  - `version`: `1.0.0`
  - `language`: `zh-CN`
  - `description`：必须包含“做什么 + 什么时候调用”，覆盖触发语：法律咨询、法条依据、合同纠纷、劳动争议、侵权、婚姻家事、公司合规、起诉/仲裁准备等。
  - `tags`：`legal`、`compliance`、`risk`、`citations`、`tool` 等。

### 2) 技能结构与内容（严谨引用版）

`skills/legal-counsel/skill.md` 内容以“可执行的流程 + 门禁规则 + 可复制模板”为主，建议包含：

1. 使用前提与边界（强制）
   - 明确不构成法律意见，不建立律师-当事人关系
   - 不替代执业律师；遇到高风险/高复杂度情形必须建议线下/正式律师咨询
   - 不编造法条、司法解释、案例编号、裁判文书；不确定即不引用

2. 信息收集（教练式）
   - 必问清单：时间、地点、主体身份、关键事实、证据现状、诉求目标、是否有书面材料
   - 风险分级：紧急（诉讼时效/强制措施/财产保全）→ 高（金额大/刑事风险/跨境）→ 中 → 低

3. 适用法域与问题分流
   - 先判定中国大陆法域与事项类型（劳动/合同/侵权/婚姻/公司/行政/刑事等）
   - 不满足法域或信息不足时：仅给“风险提示 + 补充信息清单 + 建议咨询律师”

4. 证据与引用门禁（Hard Gate，核心）
   - 任何引用必须同时具备：
     - 文件名称（全称）
     - 具体条款号（如“第X条”）
     - 效力状态与版本信息（现行有效/已修订/废止；发布日期或施行日期）
     - 可定位的来源线索（官方发布渠道/权威法律数据库页面标题与时间）
   - 不满足以上任一项：不得引用条文号；仅可给出“原则性提示”，并标注“需律师核验”
   - 明确区分：法律/行政法规/部门规章/地方性法规/司法解释/规范性文件/征求意见稿

5. 分析输出模板（默认）
   - 采用 IRAC/Issue-Rule-Application-Conclusion 结构，但 Rule 部分只有在通过引用门禁时才允许写入条文依据
   - 输出必须包含：
     - 事实摘要（用户提供 vs 待补充）
     - 争点清单（Issues）
     - 可行路径（Options）
     - 风险与不确定性（含“为什么需要律师介入”）
     - 下一步行动清单（收集材料/证据、沟通话术、时间节点）

6. 可选产物模板
   - `materials_checklist.md`：不同案件类型的材料清单与证据留存方式
   - `law_memo.md`：法律备忘录模板（仅在门禁通过时可填条文）
   - `letter_to_lawyer.md`：把用户口述整理成“给律师的案情摘要”（降低沟通成本）

### 3) 风险升级与强制转人工规则

在技能中明确“必须建议找执业律师”的触发条件（示例）：
- 刑事风险或被立案/传唤/拘留相关
- 重大金额、重大人身损害、跨境因素、群体性事件
- 时效/期限临近（诉讼时效、仲裁时限、行政复议/诉讼期限）
- 用户要求给出确定胜诉率、代写正式法律文书并承担后果、或要求规避法律监管

## Verification（执行阶段的验收方式）

- 结构验收：`skills/legal-counsel/skill.md` frontmatter 字段完整，风格与现有技能一致。
- 触发验收：description 明确“何时调用”，覆盖常见法律咨询问法。
- 严谨性验收：
  - 技能文本中包含“引用门槛 Hard Gate”且规则清晰可执行。
  - 示例模板中不会出现“编造条文/编号”的鼓励性写法。
  - 明确写出“无法核验则不引用 + 建议咨询执业律师”的用户提示话术。

