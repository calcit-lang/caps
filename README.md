# caps

`caps` is the package manager and immutable module-store manager for Calcit projects.
It resolves `deps.cirru`, Git/SemVer references and transitive dependencies, then
materializes verified revisions and installs a project-local `.calcit/modules/` view.
It is a production tool, not a compiler subcommand or workflow template. Calcit core
owns language/runtime and module-loading semantics; `calcit-native-ffi` owns native ABI
constants and raw types.

`caps` is maintained independently from the Calcit compiler. The three versions have
separate ownership:

- `caps --version` reports the package-manager version;
- `deps.cirru :calcit-version` declares the project toolchain;
- `caps verify --toolchain` probes the installed `calcit` and checks
  `@calcit/procs` against that declaration.

Use `--calcit-version <version>` only when CI or an embedding environment has already
verified a toolchain that is not discoverable through `PATH`.

```bash
cargo install calcit-caps
caps
caps tree
caps why calcit-lang/calcit.std
caps status
caps verify --toolchain
caps upgrade --all
```

The default manifest is `deps.cirru` in the current directory. Every command also
accepts an explicit manifest path as the positional input.

## 中文

`caps` 是 Calcit 项目的包管理器与不可变模块存储管理器。它负责解析
`deps.cirru`、Git/SemVer 引用和递归依赖，并把校验后的 revision 安装成项目本地的
`.calcit/modules/` view。

它是生产包管理工具，不是编译器子命令或 workflow 模板。Calcit core 负责语言、
运行时与模块加载语义；`calcit-native-ffi` 负责 native ABI 常量与底层类型。

`caps` 与 Calcit 编译器独立维护。`caps` 包版本、项目声明的 `:calcit-version`、PATH
中的真实 Calcit toolchain 各自独立；`caps verify --toolchain` 会对它们以及
`@calcit/procs` 做一致性守门。仅在 CI/嵌入环境已自行验证 toolchain 时使用
`--calcit-version <version>` 覆盖探测结果。

稳定边界、恢复语义与迁移矩阵见 [extraction contract](docs/extraction-contract.md)。

## Compatibility and releases / 兼容与发布

- caps has its own SemVer cadence and does not release in lockstep with Calcit;
- the project toolchain remains the exact `deps.cirru :calcit-version`;
- module store/project-view paths and recovery behavior follow the frozen extraction
  contract;
- native verification consumes ABI v1 from `calcit_native_ffi 0.1.3` with default
  features disabled;
- production releases are created from verified `main` through the crates.io workflow.

| caps release | `deps.cirru :calcit-version` | `@calcit/procs` | Native modules | Support status / 支持状态 |
| --- | --- | --- | --- | --- |
| `0.1.0` | exact `0.13.72` | exact `0.13.72` when present | C-safe ABI v1, verified with `calcit_native_ffi 0.1.3` and default features disabled | Stable; promoted from the cross-project-verified rc.2 implementation / 稳定版；由已通过跨项目验证的 rc.2 同一实现提升 |
| `0.1.0-rc.2` | exact `0.13.72` | exact `0.13.72` when present | C-safe ABI v1, verified with `calcit_native_ffi 0.1.3` and default features disabled | Published candidate; Calcium Workflow and Respo install/tree/status/verify smoke passed / 候选已发布，真实项目 smoke 已通过 |

`caps` can parse other exact Calcit SemVer declarations, but they are not a supported
release combination until added to this table after cross-project smoke. The installed
`calcit` and `@calcit/procs`, when present, must exactly match the declared project
toolchain.

Migration and core cutover are tracked by
[calcit#546](https://github.com/calcit-lang/calcit/issues/546) and
[calcit#554](https://github.com/calcit-lang/calcit/issues/554), with core cutover in
[calcit#555](https://github.com/calcit-lang/calcit/issues/555). Shared ABI consumer
tracking is in
[calcit-native-ffi#9](https://github.com/calcit-lang/calcit-native-ffi/issues/9).

`caps` 独立采用 SemVer，不与 Calcit 锁步发布；项目 toolchain 仍由
`deps.cirru :calcit-version` 精确声明。store/view 路径、恢复语义和 native ABI v1
兼容边界遵循 extraction contract。native verifier 直接使用关闭默认 features 的
`calcit_native_ffi 0.1.3`；生产版本只从验证通过的 `main` 经 crates.io workflow 发布。
其他 Calcit SemVer 虽可被解析，但只有经过跨项目 smoke 并加入上表后才构成正式支持
组合。候选发布与 core cutover 由上述 Issues 追踪。

Release evidence / 发布验证：从 crates.io 安装 `calcit-caps 0.1.0-rc.2` 后，
Calcium Workflow 在 fresh store 上通过 `caps --ci`、`tree`、`status`、
`verify --toolchain` 与 Calcit check，其中 `calcit-wss` 和 `calcit.std` 的 native
realization/receipt 均通过；Respo 通过同组 caps smoke 与 27/27 tests。`0.1.0`
由这份已验证实现直接提升，发布后再抽样安装确认 binary 与 crates.io 可恢复。

## Development / 开发

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Issue 与 PR 的标题、正文和阶段性进度保持中英双语。生产功能通过 PR 合并；发布版本
从已验证的 `main` 打不带 `v` 前缀的 tag。
