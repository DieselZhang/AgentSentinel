# 06 — Dashboard：对比页 + 手动上传

**What to build:** 用户在对比页输入两个 Run ID 看到并排对比，上传页粘贴 JSON 或上传文件后运行列表可见。

**Blocked by:** 05 — Dashboard：运行列表 + 详情页

**Status:** ready-for-agent

- [ ] CompareView 页面（输入两个 Run ID → 并排展示评分、耗时、token、工具调用、告警）
- [ ] UploadView 页面（表单填写 + 文件上传 + 自动解析 JSON）
- [ ] `GET /api/runs/compare?ids=id1,id2` API 实现
- [ ] `GET /api/runs/:id/report` Markdown 报告导出 API 实现