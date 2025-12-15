#!/bin/bash
# 批量修复前端编译警告脚本

cd "$(dirname "$0")"

echo "修复前端编译警告..."

# 修复swap.rs中的mut警告
sed -i 's/let mut \(.*\)_sig =/let \1_sig =/g' src/pages/swap.rs
sed -i 's/let mut \(.*\)_mut =/let \1_mut =/g' src/pages/swap.rs

# 修复未使用的变量
sed -i 's/let is_unauthorized =/let _is_unauthorized =/g' src/features/auth/hooks.rs
sed -i 's/let usdt_address_clone2 =/let _usdt_address_clone2 =/g' src/components/molecules/stablecoin_balance.rs
sed -i 's/let usdc_address_clone1 =/let _usdc_address_clone1 =/g' src/components/molecules/stablecoin_balance.rs
sed -i 's/if let Err(e) =/if let Err(_e) =/g' src/pages/dashboard.rs

echo "警告修复完成！"

