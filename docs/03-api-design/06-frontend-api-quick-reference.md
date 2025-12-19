# 前端 API 集成快速参考（已弃用）

为避免与后端真实路由/响应格式不一致，本快速参考已停止维护。

## ✅ 当前权威做法

- OpenAPI：`GET http://localhost:8088/openapi.yaml`
- 业务接口前缀：`/api/v1/...`
- 统一响应：`{ code, message, data }`

## 非托管原则

- 不上传、不记录：私钥/助记词/钱包密码。
