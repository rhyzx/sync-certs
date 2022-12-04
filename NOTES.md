# 方案调研

## Deployment(watch)
常驻 pod 通过 watch<sup>1</sup> 监听 secrets 变化。

1. watch 是通过 http 实现，无法长久保持连接<sup>2</sup>
2. 重启监听后，为了防止重复处理已处理过的 secrets，需管理 `resourceVersion`<sup>1</sup>，
  也就需要持久化 `resourceVersion`，可使用 `emptyDir`（pod重启不会清理）。
3. 常驻意味着长期占用 CPU/RAM，`watch` 是否吃 k8s 资源也未知。


## CronJob
定期执行更新

1. 无法通过 `resourceVersion` 防重复处理，因为历史记录默认只有5分钟<sup>1</sup>。
2. 如需持久化，无法使用 `emptyDir`，任务间无法共享数据。
3. 不存在 `modifiedTimestamp` 这样的信息来防重复处理。
4. 可以保存 json 数据进 annotation，通过 `lastAppliedDigest` 的机制来防重复提交到 cdn。


## 最终妥协的方案
cronjob: 每天跑一次遍历所有证书并更新


## 期望的完美方案
k8s 提供针对资源更新后自动执行的 Job


## Refs
1. https://kubernetes.io/docs/reference/using-api/api-concepts/#efficient-detection-of-changes
2. https://stackoverflow.com/questions/33480560/kubernetes-api-server-drops-watch-connections
