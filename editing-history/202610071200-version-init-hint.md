# Executable version-init hints / 可执行的版本初始化提示

- Fixed the missing-`:version` warning and the `version get`/`version bump` errors to print `caps <deps-file> version set <version>`; the deps file is a top-level positional and must precede the subcommand (calcit-lang/caps#12).
- Shell-quoted deps paths that contain spaces, quotes or other special characters, and prefixed `./` to relative paths starting with `-`.
- Added unit tests that parse the hint with the real argh grammar, a regression test that the old order is rejected, and CLI contract tests that execute the printed hints through `sh` for the default and a custom deps file.

- 修复缺少 `:version` 时的 warning 以及 `version get`/`version bump` 错误提示，改为输出 `caps <deps-file> version set <version>`；deps 文件是顶层 positional 参数，必须位于子命令之前（calcit-lang/caps#12）。
- 对包含空格、引号等特殊字符的 deps 路径做 shell 引用，并为以 `-` 开头的相对路径加 `./` 前缀。
- 增加单元测试用真实 argh 语法解析提示命令、回归测试确认旧顺序会被拒绝，并增加 CLI 契约测试在默认与自定义 deps 文件下通过 `sh` 实际执行打印出的提示。
