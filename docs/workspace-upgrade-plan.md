# Workspace 升级计划

`caps tree --workspace` 在现有依赖图查询入口中增加多仓库视图。它只读取本地文件，
不访问 GitHub、不修改 checkout，也不创建 Caps module cache；远端 release、PR、CI 和
review 状态应由外部 adapter 补充。

## Inventory

Inventory 使用 Cirru EDN。路径相对于 inventory 文件本身解析：

```cirru
{}
  :schema-version |1
  :target-calcit |0.15.8
  :projects $ []
    {}
      :repository |calcit-lang/js-ffi
      :path |../js-ffi
      :current-ref |main
      :latest-release |0.10.5
    {}
      :repository |calcit-lang/respo.calcit
      :path |../respo.calcit
      :latest-release |0.16.104
      :protected true
      :source-migration true
```

必填字段：

- `:schema-version`：当前固定为 `|1`；
- `:target-calcit`：本轮迁移的精确 SemVer；
- `:projects`：项目列表，每项必须有 `:repository` 和 `:path`。

项目可选字段：

- `:deps-file`：相对于项目目录的依赖文件，默认 `deps.cirru`；
- `:current-ref`：本地 checkout 的已验证 ref；缺省时输出 `deps.cirru :version`；
- `:latest-release`：外部流程事先确认的最新 SemVer release（兼容 `v` 前缀）；缺省会报告 `missing-release`；
- `:state`：`:active`、`:archived` 或 `:excluded`；
- `:protected`：标记外部自动化不应直接修改的项目；
- `:source-migration`：已确认需要源码迁移；
- `:release-required`：外部证据已确认当前源码需要先发版。

Inventory 是离线证据，不声称 `:latest-release` 来自实时网络。后续 adapter 应记录远端
查询时间，并把确认后的值写入输入或单独合并结果。

## 使用与输出

默认输出稳定的 Cirru EDN：

```bash
caps tree --workspace workspace.cirru
```

需要兼容只接受 JSON 的外部工具时显式指定：

```bash
caps tree --workspace workspace.cirru --format json
```

输出包含：

- `:projects`：排序后的项目、metadata、依赖、分类和直接 blocker；
- `:layers`：依赖优先、同层可并行的稳定拓扑层；
- `:publish-first`：在消费者升级前需要发布的模块；
- `:cycles`：强连通循环；循环及其下游不会被伪装进线性层级；
- `:external-dependencies`：inventory 未收录、仍由外部解析的依赖。

分类按 `source-migration`、`release-required`、`released-dependency`、
`toolchain-only`、`current` 的优先级给出。非 SemVer（例如 `main`）的内部依赖会产生
`unpublished-ref` blocker，并把被引用模块列入 `publish-first`。`archived`、`excluded`、
cycle、缺少 release 证据也会成为明确 blocker。

这个计划只描述顺序和证据，不执行 fetch、建 PR、合并或发版。
