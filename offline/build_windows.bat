@echo off
setlocal
echo === Compilando Vilas-Boas Entertainment v20.08 (Windows .exe) ===
cd /d "%~dp0"

for /f "delims=" %%i in ('rustc --print sysroot 2^>nul') do set "SYSROOT=%%i"
if defined SYSROOT (
    if exist "%SYSROOT%\lib\rustlib\x86_64-pc-windows-gnu\bin\llvm-ar.exe" (
        if not exist "%SYSROOT%\bin\llvm-dlltool.exe" (
            copy /Y "%SYSROOT%\lib\rustlib\x86_64-pc-windows-gnu\bin\llvm-ar.exe" "%SYSROOT%\bin\llvm-dlltool.exe" >nul
        )
        set "PATH=%SYSROOT%\bin;%SYSROOT%\lib\rustlib\x86_64-pc-windows-gnu\bin;%PATH%"
        set "RUSTFLAGS=-C dlltool=llvm-dlltool.exe"
    )
)

cargo build --release
if errorlevel 1 (
    echo [ERRO] Falha na compilacao do executavel.
    exit /b 1
)

copy /Y "target\release\vilas_boas.exe" "vilas_boas.exe" >nul
if not exist "dist" mkdir "dist"
copy /Y "target\release\vilas_boas.exe" "dist\vilas_boas_windows.exe" >nul

echo === Concluido com sucesso! ===
echo Executaveis disponiveis em:
echo   - offline\vilas_boas.exe
echo   - offline\target\release\vilas_boas.exe
echo   - offline\dist\vilas_boas_windows.exe
endlocal

