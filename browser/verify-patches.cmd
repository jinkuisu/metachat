@echo off
cd /d F:\chromium_src\src
git checkout 150.0.7844.0 >nul 2>&1
git checkout -- . >nul 2>&1
echo === MetaChat patch verification ===
git apply --check --reject --whitespace=fix F:\metacaht\browser\patches\metachat-all.patch
if %ERRORLEVEL% equ 0 (echo [PASS]) else (echo [FAIL])
