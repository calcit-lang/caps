# Align workspace plan contracts / 对齐 Workspace 计划契约

- Reused Caps' existing compatibility for `v`-prefixed SemVer release tags.
- Rejected `tree --format` outside workspace mode instead of silently ignoring the option.
- Added contract coverage for both boundaries.
- Applied the canonical module-name validator to inventory and dependency keys, validated project SemVer fields, and excluded inactive consumers from release requirements.
- Strengthened read-only and nested Cirru output contract tests after review.

- 复用 Caps 已有的 `v` 前缀 SemVer release tag 兼容语义。
- `tree --format` 未进入 workspace 模式时明确报错，不再静默忽略选项。
- 为两个边界补充契约测试。
- 对 inventory 与依赖键统一应用模块名校验，校验项目 SemVer 字段，并避免 inactive consumer 触发发版要求。
- 根据 review 加强全部输入文件只读以及 Cirru 嵌套输出的契约测试。
