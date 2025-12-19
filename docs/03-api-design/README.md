# API 设计与对接（以 OpenAPI 为准）

本目录下的早期“端点清单/集成指南/快速参考”已被收敛为 **OpenAPI 单一真相源**，避免文档与真实路由漂移。

## 权威来源

- OpenAPI：`GET http://localhost:8088/openapi.yaml`
- 业务接口前缀：`/api/v1/...`
- 健康检查：`GET /api/health`（别名：`GET /health`），详细探活：`GET /healthz`
- 统一响应：`{ code, message, data }`

## 非托管安全原则（强约束）

- 前端负责：助记词/私钥生成、派生、签名、本地加密存储。
- 后端仅接收：地址、公钥、交易公开数据、订单/记录元数据。
- **禁止**：任何接口/示例/日志上传或记录 `mnemonic` / `private_key` / 钱包密码。

> 如需新增对接说明：请从 OpenAPI 生成或摘取，避免手写端点列表。
