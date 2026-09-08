# Fix release upload repository / 修复 Release 上传仓库定位

- Passed the explicit GitHub repository to `gh release upload` because the least-privilege upload job intentionally has no checkout and therefore no local Git remote.
- Recovered the `0.1.1` release from its verified workflow artifact and confirmed both public assets before applying this permanent workflow fix.

- 为 `gh release upload` 显式传入 GitHub 仓库；最小权限上传 job 按设计不 checkout，因此本地没有 Git remote 可供自动推断。
- 使用已验证的 workflow artifact 恢复 `0.1.1` Release，并确认两个公开产物后补上此永久修复。
