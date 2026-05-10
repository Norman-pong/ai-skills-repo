---
name: deep-interview
version: "1.0.0"
description: 需求澄清与苏格拉底式深度访谈。触发词：深度访谈、采访我、问我所有问题、不要假设、ouroboros / deep interview、interview me、ask me everything、don't assume。将模糊想法转化为可执行的中文规格说明。
argument-hint: "[--quick|--standard|--deep] <想法或模糊描述>（默认 --standard）"
---

<Purpose>
Deep Interview 是一个"意图优先"的苏格拉底式澄清循环，发生在规划或执行之前。它通过有针对性的提问，将模糊的想法转化为可执行的规格说明。提问聚焦于：用户为什么想要这个变更、变更应该做到什么程度、什么应该明确排除在范围之外、以及 agent 可以在不经确认的情况下自主决定哪些事项。
</Purpose>

<When_to_Use>
- 请求宽泛、模糊或缺少具体验收标准
- 用户说了"深度访谈"、"采访我"、"问我所有问题"、"不要假设"或"ouroboros"
- 用户希望避免因需求定义不清而导致的实现偏差
- 在移交规划或执行技能之前，你需要一份需求 artifact
</When_to_Use>

<Do_Not_Use_When>
- 请求已有具体的文件/符号目标和明确的验收标准
- 用户明确要求跳过规划/访谈并立即执行
- 用户只需要轻量级的头脑风暴（使用通用规划技能代替）
- 已有完整的 PRD/规划文档，应该直接进入执行
</Do_Not_Use_When>

<Why_This_Exists>
执行质量通常受限于意图清晰度，而不仅仅是实现细节的缺失。单次扩展往往遗漏了用户为什么想要变更、范围应该在哪里停止、哪些权衡是不可接受的、以及哪些决策仍需要用户批准。本工作流应用苏格拉底式压力 + 量化模糊度评分，确保 agent 在启动时拥有一份明确、可测试、意图对齐的规格说明。
</Why_This_Exists>

<Prerequisites>
1. 检查项目中的 `AGENTS.md`（如果存在），获取项目特定的约定、编码风格和偏好，用以指导访谈问题。
2. 在向用户询问内部信息之前，先通过只读探索收集代码库事实。
3. 对于现有代码库（brownfield）工作，在提出第一个问题之前，先对 brownfield 与 greenfield 进行分类。
</Prerequisites>

<Depth_Profiles>
- **Quick (`--quick`)**：快速预规格审查；目标阈值 `<= 0.30`；最多 5 轮
- **Standard (`--standard`, 默认)**：完整需求访谈；目标阈值 `<= 0.20`；最多 12 轮
- **Deep (`--deep`)**：高严谨度探索；目标阈值 `<= 0.15`；最多 20 轮

如果未提供 flag，使用 **Standard**。
</Depth_Profiles>

<Execution_Policy>
- 每轮只问 ONE 个问题（绝不批量提问）
- 在询问实现细节之前，先问意图和边界
- 在应用下面的阶段优先级规则后，每轮针对清晰度最低的维度提问
- 将每个答案视为需要压力测试的声明：下一个问题通常应要求证据或例子、暴露隐藏假设、强制做出权衡或边界选择、或将症状重新框架为本质/根本原因
- 当当前答案仍然模糊时，不要为了覆盖范围而转向新的清晰度维度；停留在同一线程上，直到更深入一层、假设更清晰、或边界更紧凑
- 在结晶之前，完成至少一次明确的压力回访：用一个更深入、聚焦假设或聚焦权衡的跟进问题重新访问之前的答案
- 在向用户询问内部信息之前，通过只读探索收集代码库事实
- 在第一个访谈问题之前，始终运行预检上下文收集
- 减少用户工作量：只问最高杠杆的未解决问题，绝不问 agent 可以直接发现的代码库事实
- 对于 brownfield 工作，优先使用基于证据的确认性问题，例如"我在 Y 中发现了 X。这个变更是否应该遵循该模式？"
- 优先使用结构化用户输入工具；如果不可用，回退到简洁的纯文本单问题轮次
- 每次回答后重新评分模糊度并透明展示进度
- 除非用户明确选择带警告继续，否则在模糊度高于阈值时不移交执行
- 即使加权模糊度低于阈值，如果 `Non-goals` 或 `Decision Boundaries` 仍未解决，不要结晶或移交
- 将提前退出视为安全阀，而非默认成功路径
- 通过平台的状态机制（例如会话状态、检查点或基于文件的快照）持久化模式状态，以支持恢复安全
- **语言规则（强制）**：所有向用户提出的问题必须用中文（简体中文）。Round 提示可以保持英文结构标签，但问题正文必须是流畅的中文。
- **用户回答质量处理**：如果用户拒绝回答、回答完全无关、或要求跳过当前问题：
  - 第一次：礼貌重申该问题的重要性，尝试用不同角度重新提问
  - 第二次（同一问题的再次拒绝）：将该维度标记为 "用户未提供"，在评分中按当前清晰度估分，并记录风险注释
  - 始终允许用户说 "我不知道" —— 将其视为有效输入，转而探测用户的假设和约束边界
</Execution_Policy>

<Steps>

## Phase 0: Preflight Context Intake（预检上下文收集）

1. 解析 `{{ARGUMENTS}}` 并派生一个短任务 slug。
2. 尝试从工作区加载任何现有的相关上下文快照（例如 `.agent/interviews/context-{slug}-*.md`）。
3. 如果不存在快照，创建一个最小上下文快照，包含：
   - 任务陈述
   - 期望结果
   - 用户提出的解决方案（他们要求什么）
   - 可能的意图假设（他们为什么可能想要这个）
   - 已知事实/证据
   - 约束条件
   - 未知/开放问题
   - 决策边界未知
   - 可能的代码库接触点
4. 将快照保存到工作区的 `.agent/interviews/context-{slug}-{timestamp}.md`（UTC `YYYYMMDDTHHMMSSZ`），并在会话状态中引用它。

## Phase 1: Initialize（初始化）

1. 解析 `{{ARGUMENTS}}` 和深度配置（`--quick|--standard|--deep`）。
2. 检测项目上下文：
   - 运行只读探索以分类 **现有代码库（brownfield）** vs **新项目（greenfield）**。
   - 对于 brownfield，在提问之前收集相关代码库上下文。
3. 初始化会话状态：

```json
{
  "active": true,
  "current_phase": "deep-interview",
  "state": {
    "interview_id": "<uuid>",
    "profile": "quick|standard|deep",
    "type": "greenfield|brownfield",
    "initial_idea": "<user input>",
    "rounds": [],
    "current_ambiguity": 1.0,
    "threshold": 0.3,
    "max_rounds": 5,
    "challenge_modes_used": [],
    "codebase_context": null,
    "current_stage": "intent-first",
    "current_focus": "intent",
    "context_snapshot_path": ".agent/interviews/{slug}-{timestamp}.md"
  }
}
```

4. 用配置、阈值和当前模糊度宣布启动。

## Phase 2: Socratic Interview Loop（苏格拉底访谈循环）

重复直到（模糊度 `<= threshold` **且** 压力回访完成 **且** 就绪门控明确），或用户带警告退出，或达到最大轮数。

### 2a) Generate next question（生成下一个问题）
使用：
- 原始想法
- 先前问答轮次
- 当前维度得分
- 现有代码库上下文（如有）
- 激活的挑战模式注入（Phase 3）

针对得分最低的维度，但遵守阶段优先级：
- **Stage 1 — Intent-first（意图优先）**：Intent、Outcome、Scope、Non-goals、Decision Boundaries
- **Stage 2 — Feasibility（可行性）**：Constraints、Success Criteria
- **Stage 3 — Brownfield grounding（现有代码库锚定）**：Context Clarity（仅 brownfield）

每次回答后的跟进压力阶梯：
1. 要求具体例子、反例或证据信号来支撑最新声明
2. 探测使声明成立的隐藏假设、依赖或信念
3. 强制做出边界或权衡：你会明确不做什么、推迟什么或拒绝什么？
4. 如果答案仍在描述症状，在继续之前将其重新框架为本质/根本原因

当同一线程具有最高杠杆时，优先在多个轮次中停留。没有压力的广度不是进步。

详细维度：
- Intent Clarity — 用户为什么想要这个
- Outcome Clarity — 他们想要的最终状态是什么
- Scope Clarity — 变更应该做到什么程度
- Constraint Clarity — 必须保持的技术或业务限制
- Success Criteria Clarity — 完成将如何被评判
- Context Clarity — 现有代码库理解（仅 brownfield）

`Non-goals` 和 `Decision Boundaries` 是强制就绪门控。尽早询问它们，并持续重访直到明确。

### 2b) Ask the question（提问）
使用运行时中可用的结构化用户输入工具，并展示：

```
Round {n} | Target: {weakest_dimension} | Ambiguity: {score}%

{question}
```

**注意**：`{question}` 必须是中文（简体中文），即使在技术标签中使用英文术语。

### 2c) Score ambiguity（评分模糊度）
在 `[0.0, 1.0]` 范围内为每个维度评分**清晰度（clarity）**，附理由 + 差距。
清晰度 = 1 表示该维度完全明确，0 表示完全模糊。

greenfield 模糊度公式：
`ambiguity = 1 - (intent_clarity × 0.30 + outcome_clarity × 0.25 + scope_clarity × 0.20 + constraints_clarity × 0.15 + success_clarity × 0.10)`

brownfield 模糊度公式：
`ambiguity = 1 - (intent_clarity × 0.25 + outcome_clarity × 0.20 + scope_clarity × 0.20 + constraints_clarity × 0.15 + success_clarity × 0.10 + context_clarity × 0.10)`

**为什么 `Non-goals` 和 `Decision Boundaries` 不参与加权评分？**

这两个维度是**硬门控（hard gates）**，而非连续评分维度：
- 它们只有两种状态：明确 / 未明确
- 即使所有评分维度的清晰度都很高，只要任一硬门控未明确，规格仍然不可执行
- 因此它们不进入模糊度公式，而是作为独立的通过/阻塞条件

就绪门控：
- `Non-goals` 必须明确
- `Decision Boundaries` 必须明确
- 压力回访必须完成：至少一个之前的答案已被证据、假设或权衡跟进重新访问
- 如果任一门控未解决，或压力回访不完整，即使加权模糊度低于阈值，也要继续访谈

### 2d) Report progress（报告进度）
显示加权分解表、就绪门控状态（`Non-goals`、`Decision Boundaries`）和下一个聚焦维度。

### 2e) Persist state（持久化状态）
将轮次结果和更新后的得分追加到会话状态。

### 2f) Round controls（轮次控制）
- 在第一次明确假设探测和一个持续跟进完成之前，不提供提前退出
- 第 4 轮起：允许带风险警告的明确提前退出
- 在配置中点（例如根据配置的第 3/6/10 轮）发出软警告
- 在配置的 `max_rounds` 处硬封顶

## Phase 3: Challenge Modes（挑战模式 — 假设压力测试）

在适用时各使用一次。这些是常规升级工具，不是罕见的救援手段：

- **Contrarian（反对者）**（第 2 轮+，或当答案基于未经测试的假设时立即使用）：挑战核心假设
- **Simplifier（简化者）**（第 4 轮+，或当范围扩张快于结果清晰度时）：探测最小可行范围
- **Ontologist（本体论者）**（第 5 轮+ 且模糊度 > 0.25，或当用户持续描述症状时）：要求本质级重新框架

在状态中跟踪已使用的模式以防止重复。

## Phase 4: Crystallize Artifacts（结晶 Artifact）

当阈值达到时（或用户带警告退出 / 硬封顶）：

1. 将访谈记录摘要写入：
   - `.agent/interviews/transcript-{slug}-{timestamp}.md`
2. 将执行就绪规格写入：
   - `.agent/specs/deep-interview-{slug}.md`

规格应包含（用中文撰写）：
- 元数据（配置、轮次、最终模糊度、阈值、上下文类型）
- 上下文快照引用/路径
- 清晰度分解表
- 意图（用户为什么想要这个）
- 期望结果
- 范围内
- 范围外 / Non-goals
- 决策边界（agent 可以在不经确认的情况下决定什么）
- 约束条件
- 可测试的验收标准
- 暴露的假设 + 解决方案
- 压力回访发现（哪个答案被重访了，什么发生了变化）
- 任何基于仓库的确认问题的 brownfield 证据 vs 推断注释
- 技术上下文发现
- 完整或精简的访谈记录

## Phase 5: Execution Bridge（执行桥接）

在 artifact 生成后，使用明确的移交合同呈现执行选项。将 deep-interview 规格视为当前需求的真实来源，并在移交中保留意图、非目标、决策边界、验收标准和任何残余风险警告。

### 1. **规划桥接（Planning Bridge — 推荐）**
- **消费者角色：** 任何负责架构设计、可行性分析或 PRD 编写的规划阶段 agent/skill
- **输入 Artifact：** `.agent/specs/deep-interview-{slug}.md`（可选地附带访谈记录/上下文快照以支持可追溯性）
- **消费者行为：** 将 deep-interview 规格视为需求真实来源。默认不重复访谈；而是围绕已澄清的意图和边界 refine 架构/可行性。
- **跳过 / 已满足阶段：** 需求发现、模糊度澄清、早期意图-边界引出
- **预期输出：** 规范规划 artifact（例如 PRD、架构文档、测试规格）
- **最适合：** 需求已足够清晰可以停止访谈，但架构验证 / 共识规划仍然可取
- **下一步推荐：** 使用批准的规划 artifact 配合执行技能或 agent

### 2. **执行桥接（Execution Bridge）**
- **消费者角色：** 任何负责代码实现、测试或部署的执行阶段 agent/skill
- **输入 Artifact：** `.agent/specs/deep-interview-{slug}.md`
- **消费者行为：** 将 deep-interview 规格作为已澄清的执行简报。保留意图、非目标、决策边界和验收标准作为规划/执行的绑定上下文。
- **跳过 / 已满足阶段：** 初始需求发现和模糊度降低
- **预期输出：** 规划/执行进度、QA 证据和验证 artifact
- **最适合：** 已澄清的规格已足够强，可以直接规划 + 执行，无需额外的共识门控
- **下一步推荐：** 继续执行 agent 的验证流程

### 3. **多 Agent 协调桥接（Multi-Agent Bridge）**
- **消费者角色：** 需要协调多个并行 agent/lane 的编排层
- **输入 Artifact：** `.agent/specs/deep-interview-{slug}.md`
- **消费者行为：** 将规格视为协调并行工作的共享执行上下文。保留已澄清的意图、非目标、决策边界和验收标准作为共同约束。
- **跳过 / 已满足阶段：** 需求澄清和早期模糊度降低
- **预期输出：** 针对共享规格的协调多 agent 执行
- **最适合：** 任务足够大、多线程或对阻塞敏感，值得协调并行执行而非单一持久循环

### 4. **进一步精炼（Refine further）**
- **输入 Artifact：** 现有访谈记录、上下文快照和当前规格草稿
- **调用：** 继续访谈循环
- **消费者行为：** 重新进入提问以解决最高杠杆的剩余不确定性
- **跳过 / 已满足阶段：** 除已捕获上下文外无其他
- **预期输出：** 模糊度更低、边界更紧凑、未解决假设更少的规格
- **最适合：** 残余模糊度仍然太高、用户想要更强的清晰度、或阈值以上/提前退出警告表明风险太大无法干净继续
- **下一步推荐：** 一旦规格足够澄清，返回上面的某个执行移交合同

**Residual-Risk Rule（残余风险规则）：** 如果访谈通过提前退出、硬封顶完成或阈值以上带警告继续而结束，在移交中明确保留该残余风险状态，以便下游技能/agents 知道它们继承了一份部分澄清的简报。

**IMPORTANT：** Deep-interview 是一个需求模式。在移交时，使用上面的合同调用选定的技能/agent。**不要**在 deep-interview 内部直接实现。

</Steps>

<Tool_Usage>
- 使用只读探索（subagent、glob/grep 或等效工具）进行代码库事实收集
- 每轮访谈优先使用结构化用户输入工具
- 如果结构化问题工具不可用，使用纯文本单问题轮次并保持相同的阶段顺序
- 使用平台状态机制实现可恢复的模式状态
- 在 `.agent/interviews/` 下读取/写入上下文快照
- 在 `.agent/interviews/` 和 `.agent/specs/` 下保存访谈记录/规格 artifact
</Tool_Usage>

<Escalation_And_Stop_Conditions>
- 用户说停止/取消/中止 -> 持久化状态并停止
- 模糊度停滞 3 轮（+/- 0.05）-> 强制使用 Ontologist 模式一次
- 达到最大轮数 -> 带明确残余风险警告继续
- 所有维度 >= 0.9 -> 即使在最大轮数之前也允许提前结晶
</Escalation_And_Stop_Conditions>

<Final_Checklist>
- [ ] 预检上下文快照存在于 `.agent/interviews/context-{slug}-{timestamp}.md`
- [ ] 每轮显示模糊度评分
- [ ] 在实现细节之前使用意图优先阶段优先级
- [ ] 在活动阶段内使用最弱维度靶向
- [ ] 在结晶之前至少发生一次明确的假设探测
- [ ] 至少一次持续跟进/压力回访深化了之前的答案
- [ ] 在阈值处触发挑战模式（如适用）
- [ ] 访谈记录写入 `.agent/interviews/transcript-{slug}-{timestamp}.md`
- [ ] 规格写入 `.agent/specs/deep-interview-{slug}.md`
- [ ] brownfield 相关问题在适用时使用基于证据的确认
- [ ] 提供移交选项（规划桥接、执行桥接、多 Agent 协调桥接）
- [ ] 所有向用户提出的问题均为中文（语言合规）
- [ ] 在此模式下不直接执行实现
</Final_Checklist>

<Advanced>
## 建议配置（可选）

平台可以通过以下配置向量暴露这些设置（环境变量、设置或 frontmatter 覆盖）：

| 变量 / 设置                | 默认值     | 描述                                         |
|---------------------------|-----------|---------------------------------------------|
| `DEEP_INTERVIEW_PROFILE`  | `standard`| 默认深度配置                                 |
| `QUICK_THRESHOLD`         | `0.30`    | Quick 配置的模糊度阈值                        |
| `STANDARD_THRESHOLD`      | `0.20`    | Standard 配置的模糊度阈值                     |
| `DEEP_THRESHOLD`          | `0.15`    | Deep 配置的模糊度阈值                         |
| `QUICK_MAX_ROUNDS`        | `5`       | Quick 配置的最大轮数                          |
| `STANDARD_MAX_ROUNDS`     | `12`      | Standard 配置的最大轮数                       |
| `DEEP_MAX_ROUNDS`         | `20`      | Deep 配置的最大轮数                           |
| `ENABLE_CHALLENGE_MODES`  | `true`    | 启用 Contrarian/Simplifier/Ontologist 模式   |

## 恢复

如果中断：
1. 检查 `.agent/interviews/context-{slug}-*.md` 找到最新的上下文快照
2. 检查 `.agent/interviews/transcript-{slug}-*.md` 找到最新的访谈记录
3. 重新运行 deep-interview 技能，传入相同的任务描述
4. 如果平台支持会话状态恢复，优先使用；否则从上述 artifact 重建状态并继续

## 推荐的 3 阶段管道

```
deep-interview -> planning -> execution
```

- 阶段 1（deep-interview）：清晰度门控
- 阶段 2（planning）：可行性 + 架构门控
- 阶段 3（execution）：实现 + QA + 验证门控
</Advanced>

Task: {{ARGUMENTS}}
