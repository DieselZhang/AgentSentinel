# 05 — Dashboard：运行列表 + 详情页

**What to build:** 用户打开 Dashboard，看到运行列表（卡片含安全评分），点击进入详情页，看到安全评分圆环、工具调用时间线、安全告警列表。

**Blocked by:** 04 — 评测服务器 + 数据存储

**Status:** ready-for-agent

- [ ] Pinia store（loadRuns、loadRun、loadCompare）
- [ ] RunList 页面（卡片网格 + 搜索筛选 + 安全评分徽章）
- [ ] RunCard 组件（任务名、状态、模型、耗时、评分）
- [ ] SafetyScore 组件（圆环 SVG 评分图，绿/黄/红三色）
- [ ] Timeline 组件（工具调用时间线，blocked/error 高亮）
- [ ] RunDetail 页面（安全告警区 + 时间线 + 运行统计）
- [ ] Axios API client 封装