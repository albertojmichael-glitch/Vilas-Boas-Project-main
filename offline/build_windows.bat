@echo off
setlocal
pushd "%~dp0"
if errorlevel 1 exit /b 1

cargo build --release --locked
if errorlevel 1 goto :fail

if not exist "dist" mkdir "dist"
if errorlevel 1 goto :fail
copy /Y "target\release\vilas_boas.exe" "dist\Vilas-Boas.exe" >nul
if errorlevel 1 goto :fail
echo Executavel disponivel em offline\dist\Vilas-Boas.exe
popd
exit /b 0

:fail
popd
exit /b 1
