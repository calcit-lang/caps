# Release review follow-up / 发布审查修正

- Split verification, crates.io publication, and GitHub Release upload into jobs with least-privilege permissions.
- Stage the verified binary as a short-lived workflow artifact, publish the crate, and expose release assets only after publication succeeds.
- Use retry-safe `gh release upload --clobber` and pin the new artifact transfer actions to immutable commit SHAs.
- Document that `0.1.0` must be backfilled before `setup-calcit` selects it, or that both projects must select a newer release containing the assets.

- 将验证、crates.io 发布和 GitHub Release 上传拆分为最小权限的独立 job。
- 先暂存已验证的短期 workflow artifact，发布 crate 成功后才公开 Release assets。
- 使用可安全重试的 `gh release upload --clobber`，并将新增的 artifact actions 固定到不可变 commit SHA。
- 明确 `setup-calcit` 选择 `0.1.0` 前必须补齐产物，否则两个项目应同步选择包含产物的新版本。
