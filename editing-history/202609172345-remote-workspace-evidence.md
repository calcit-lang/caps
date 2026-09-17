# Reconcile remote workspace evidence / 对齐远端 Workspace 证据

- Added read-only local Git evidence and stable actionable states to workspace plans.
- Added explicit Cirru EDN or JSON remote-evidence reconciliation without network access in the Caps planner.
- Added an optional read-only GitHub adapter script that finds default-branch state and target-matching PR checks/reviews without adding an installed top-level command.
- Covered dirty/protected/archived/excluded projects, fetch-needed state, PR conflicts, failed checks, pending reviews, malformed evidence, and input immutability.

- 为 workspace 计划增加只读本地 Git evidence 和稳定 actionable state。
- 通过显式 Cirru EDN 或 JSON 文件对齐远端证据，Caps planner 本身不访问网络。
- 增加可选只读 GitHub adapter 脚本，读取 default branch 与目标版本匹配 PR 的 checks/reviews，不增加安装后的顶层命令。
- 覆盖 dirty/protected/archived/excluded、fetch-needed、PR 冲突、失败 checks、待 review、错误证据和输入不变性。
