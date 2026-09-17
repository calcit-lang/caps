# Align workspace plan contracts / 对齐 Workspace 计划契约

- Reused Caps' existing compatibility for `v`-prefixed SemVer release tags.
- Rejected `tree --format` outside workspace mode instead of silently ignoring the option.
- Added contract coverage for both boundaries.

- 复用 Caps 已有的 `v` 前缀 SemVer release tag 兼容语义。
- `tree --format` 未进入 workspace 模式时明确报错，不再静默忽略选项。
- 为两个边界补充契约测试。
