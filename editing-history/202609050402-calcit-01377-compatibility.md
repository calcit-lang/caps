# Calcit 0.13.77 compatibility evidence / Calcit 0.13.77 兼容证据

UTC: 2026-09-04T20:02:45Z

## English

- Added exact Calcit and `@calcit/procs` 0.13.77 to the supported caps 0.1.0 compatibility matrix.
- Recorded fresh-runner evidence from three native binding repositories and one dependency-free
  native project. Their workflows exercised strict module installation, exact toolchain checks,
  native realization where applicable, and repository-specific gates.
- Kept caps 0.1.0 independently versioned, preserved the existing 0.13.72 row, and made no runtime,
  resolver, module-store, native ABI, package-version, or publishing-authorization change; release
  workflow action references were hardened separately.
- Pinned checkout, Rust toolchain, Rust cache, and crates publishing actions to reviewed immutable
  commits; checkout credentials remain disabled and the release job keeps its existing OIDC/token scope.
- Reviewed the existing generic SemVer and exact toolchain-enforcement tests. No new version-literal
  test was added: it would duplicate the already version-independent contract without exercising a
  new Caps code path; the real consumer runs provide the required compatibility evidence.

## 中文

- 将精确 Calcit 与 `@calcit/procs` 0.13.77 加入 caps 0.1.0 的正式支持矩阵。
- 记录三个 native binding 仓库和一个无依赖 native 项目的全新 runner 证据；这些 workflow
  覆盖 strict 模块安装、精确工具链、适用时的 native realization 与仓库自身门禁。
- 保持 caps 0.1.0 独立版本与既有 0.13.72 行，不修改 runtime、解析器、module store、native ABI、
  包版本或发布授权；release workflow 的 Action 引用另行完成固定化加固。
- 将 checkout、Rust toolchain、Rust cache 与 crates 发布 Action 固定到已审阅的不可变 commit；
  checkout 凭据继续关闭，release job 保持原有 OIDC/token 权限边界。
- 已审阅通用 SemVer 与精确工具链守门测试。没有加入只替换版本字面量的新测试，因为它不会覆盖
  新的 Caps 代码路径；真实消费者运行才是本次兼容声明所需的证据。
