# Linux release artifact / Linux 发布产物

- Extended the release workflow to compile `caps` once on `ubuntu-latest` and attach the executable to the GitHub release.
- Added `caps-release-manifest.json` with the release version, SHA-256 digest, and byte size so consumers can verify both fresh downloads and cached binaries.
- Kept the release scope intentionally limited to Linux x64; no macOS, Windows, or additional architecture matrix was added.

- 扩展 release workflow，在 `ubuntu-latest` 上编译一次 `caps` 并把可执行文件附加到 GitHub Release。
- 新增包含版本、SHA-256 和文件大小的 `caps-release-manifest.json`，供调用方校验首次下载和缓存命中的二进制。
- 发布范围按要求仅限 Linux x64，没有增加 macOS、Windows 或其他架构矩阵。
