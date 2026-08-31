# caps contributor guide / caps 协作指南

## Scope / 边界

This repository owns package resolution, immutable module storage, project views,
native artifact realization and related diagnostics. It does not own Calcit parsing,
runtime, Snapshot loading or compiler semantics.

本仓库负责依赖解析、不可变模块存储、项目 view、native artifact realization 及相关
诊断；不负责 Calcit 语言解析、运行时、Snapshot 加载和编译器语义。

Native ABI constants and raw types must come directly from `calcit_native_ffi` with
default features disabled. Do not add a dependency on the `calcit` crate.

## Workflow / 流程

- Keep Issue and PR titles, bodies and progress updates bilingual.
- Preserve CLI stdout/stderr and exit-status contracts.
- Add tests for normal, malformed, interrupted and concurrent paths.
- Before each commit, add a timestamped note under `editing-history/`.
- Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo test --all-targets`.

## Version ownership / 版本所有权

- `CARGO_PKG_VERSION` is the caps release only.
- `deps.cirru :calcit-version` is the project toolchain and native build identity.
- Installed Calcit is probed through `calcit --version`/`calcit -v`, unless an explicit
  verified override is supplied.

