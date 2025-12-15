@echo off
chcp 65001 >nul
echo 清理 IronForge 构建缓存...
echo.

cd /d "%~dp0"

echo [1/3] 清理 dist 目录...
if exist dist (
    rmdir /s /q dist
    echo   ✓ dist 目录已删除
) else (
    echo   - dist 目录不存在
)

echo [2/3] 清理 target/wasm32-unknown-unknown 目录...
if exist target\wasm32-unknown-unknown (
    rmdir /s /q target\wasm32-unknown-unknown
    echo   ✓ wasm32-unknown-unknown 目录已删除
) else (
    echo   - wasm32-unknown-unknown 目录不存在
)

echo [3/3] 清理浏览器缓存提示...
echo.
echo ════════════════════════════════════════════════════════════
echo   清理完成！
echo ════════════════════════════════════════════════════════════
echo.
echo   下一步操作：
echo   1. 在浏览器中按 Ctrl+Shift+Delete 清除缓存
echo   2. 或者按 Ctrl+F5 强制刷新页面
echo   3. 重新运行 trunk serve 启动前端
echo.
echo   如果问题仍然存在，请：
echo   - 关闭所有浏览器标签页
echo   - 清除浏览器缓存和 Cookie
echo   - 使用隐私模式打开 http://127.0.0.1:8080
echo.
pause

