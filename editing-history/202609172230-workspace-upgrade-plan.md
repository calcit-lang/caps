# Offline workspace upgrade plan / 离线 Workspace 升级计划

- Extended `caps tree` with a read-only workspace inventory mode instead of adding a new top-level command.
- Added deterministic dependency-first layers, cycle reporting, release evidence, blockers, project metadata, and upgrade classifications.
- Kept Cirru EDN as the default machine-readable format and made JSON an explicit compatibility option.
- Added CLI fixtures covering released dependencies, unpublished branch refs, stable ordering, malformed input, and the no-mutation boundary.

- 在现有 `caps tree` 下增加只读 workspace inventory 模式，没有新增顶层命令。
- 增加确定性的依赖优先层级、循环报告、release 证据、阻塞原因、项目 metadata 与升级分类。
- 保持 Cirru EDN 为默认机器可读格式，JSON 仅作为显式兼容选项。
- 增加 CLI fixture，覆盖已发布依赖、未发布 branch ref、稳定顺序、错误输入和不修改本地状态的边界。
