# 测试说明

## WASM测试环境配置

### 前置要求
- Node.js (v16+)
- wasm-pack (最新版本)

### 安装wasm-pack
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### 运行测试
```bash
# 在IronForge目录下运行
wasm-pack test --headless --firefox
# 或使用浏览器运行
wasm-pack test --chrome
```

### 测试文件说明
- `swap_validation_test.rs` - 交换验证逻辑测试（金额验证、代币选择验证）
- `address_validation_test.rs` - 地址验证测试（以太坊地址、比特币地址）
- `cache_test.rs` - 缓存服务测试
- `error_handling_test.rs` - 错误处理逻辑测试

### 注意事项
- WASM测试需要在浏览器环境中运行
- 某些API（如web_sys）只能在浏览器环境中使用
- 测试使用`wasm-bindgen-test`框架

