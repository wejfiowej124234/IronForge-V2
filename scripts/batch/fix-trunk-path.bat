@echo off
:: 修复trunk路径问题
:: 创建target目录（如果不存在）
if not exist "target" (
    mkdir target
    echo [OK] Created target directory
) else (
    echo [INFO] target directory already exists
)

echo [OK] Trunk path issue fixed
echo [INFO] You can now run: trunk serve --release

