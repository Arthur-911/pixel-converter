@echo off
setlocal enabledelayedexpansion

set "EXE_PATH=%~dp0target\release\pixelgenrator.exe"

if not exist "!EXE_PATH!" (
    echo Error: pixelgenrator.exe not found. Building release binary...
    cargo build --release
)

:: If double-clicked with no arguments, open native image picker
if "%~1"=="" (
    "!EXE_PATH!"
    exit /b 0
)

:: If files dragged & dropped onto this script, process all of them
:loop
if "%~1"=="" goto done
"!EXE_PATH!" "%~1"
shift
goto loop

:done
exit /b 0
