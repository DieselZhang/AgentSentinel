# 04 — 评测服务器 + 数据存储

**What to build:** 启动 eval-server 后，用户可通过 API 上传 trace 数据并查询运行列表，系统自动计算安全评分、检测安全告警，CLI 新增 upload 子命令。

**Blocked by:** 02 — Agent 运行时核心 + CLI 运行命令

**Status:** ready-for-agent

- [ ] SQLite 数据库初始化（runs 表 + safety_alerts 表 + 索引）
- [ ] 安全评分引擎（危险操作 40% + 任务完成 30% + 效率 20% + 稳定性 10%）
- [ ] 安全告警检测（扫描 events JSON 中的危险命令和敏感路径）
- [ ] `POST /api/runs` 上传 trace 接口
- [ ] `GET /api/runs` 运行列表接口（支持筛选 task_name、min_score）
- [ ] `GET /api/runs/:id` 运行详情接口
- [ ] CLI upload 子命令实现