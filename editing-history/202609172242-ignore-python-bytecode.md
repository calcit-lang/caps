# Keep Python bytecode out of packages / 排除 Python 字节码

- Ignore Python bytecode caches created by adapter tests so `cargo package` sees a clean tree in CI.
- 排除 adapter 测试生成的 Python 字节码缓存，保证 CI 中 `cargo package` 面对干净工作区。
