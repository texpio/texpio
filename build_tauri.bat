@echo off
setlocal
pushd "%~dp0" || exit /b 1

if exist ".tools\bun\1.4.4\package\bin\bun.exe" set "PATH=%CD%\.tools\bun\1.4.4\package\bin;%PATH%"
if exist ".tools\node\node-v24.18.0-win-x64\node.exe" set "PATH=%CD%\.tools\node\node-v24.18.0-win-x64;%PATH%"
if exist ".tools\cargo\bin\cargo.exe" set "PATH=%CD%\.tools\cargo\bin;%PATH%"
if exist ".tools\cargo" set "CARGO_HOME=%CD%\.tools\cargo"
if exist ".tools\rustup" set "RUSTUP_HOME=%CD%\.tools\rustup"

call bun run tauri build -- --locked
set "build_result=%ERRORLEVEL%"
popd
exit /b %build_result%
