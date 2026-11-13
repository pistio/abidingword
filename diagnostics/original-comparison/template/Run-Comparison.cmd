@echo off
setlocal
cd /d "%~dp0"
if not exist "tools\node\node.exe" (
  echo Package incomplete. Extract the ENTIRE ZIP before running this launcher.
  pause
  exit /b 1
)
"tools\node\node.exe" "run-comparison.mjs"
set "AW_COMPARISON_EXIT=%ERRORLEVEL%"
echo.
echo The comparison has ended. See the printed evidence ZIP path above.
echo The installed application was not changed.
pause
exit /b %AW_COMPARISON_EXIT%
