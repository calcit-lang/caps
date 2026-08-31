# caps

`caps` is the package manager and immutable module-store manager for Calcit projects.
It resolves `deps.cirru`, Git/SemVer references and transitive dependencies, then
materializes verified revisions and installs a project-local `.calcit/modules/` view.

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

`caps` 与 Calcit 编译器独立维护。`caps` 包版本、项目声明的 `:calcit-version`、PATH
中的真实 Calcit toolchain 各自独立；`caps verify --toolchain` 会对它们以及
`@calcit/procs` 做一致性守门。仅在 CI/嵌入环境已自行验证 toolchain 时使用
`--calcit-version <version>` 覆盖探测结果。

稳定边界、恢复语义与迁移矩阵见 [extraction contract](docs/extraction-contract.md)。

## Development / 开发

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Issue 与 PR 的标题、正文和阶段性进度保持中英双语。生产功能通过 PR 合并；发布版本
从已验证的 `main` 打不带 `v` 前缀的 tag。

