# IronCore 后端 API 参考（已弃用）

本文件曾包含手写的端点与响应示例，但容易与真实实现漂移。

## ✅ 当前权威做法

- **以 OpenAPI 为准**：`GET http://localhost:8088/openapi.yaml`
- **业务接口统一前缀**：`/api/v1/...`
- **健康检查**：`GET /api/health`（别名 `GET /health`），`GET /healthz`
- **统一响应结构**：`{ code, message, data }`

## 安全约束（非托管）

后端不得接收或记录：助记词、私钥、钱包密码。

> 如需对接说明，请在本仓库中引用 OpenAPI 的真实 schema/路径，而不是维护手写清单。
