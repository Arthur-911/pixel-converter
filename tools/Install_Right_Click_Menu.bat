@echo off
setlocal

set "EXE_PATH=%~dp0..\target\release\pixelgenrator.exe"

echo Adding "Convert to Pixel Art" to Right-Click Menu...
reg add "HKCU\Software\Classes\SystemFileAssociations\image\shell\PixelArt" /ve /d "Convert to Pixel Art" /f >nul
reg add "HKCU\Software\Classes\SystemFileAssociations\image\shell\PixelArt" /v "Icon" /d "%EXE_PATH%" /f >nul
reg add "HKCU\Software\Classes\SystemFileAssociations\image\shell\PixelArt\command" /ve /d "\"%EXE_PATH%\" \"%%1\"" /f >nul

if %ERRORLEVEL% EQU 0 (
    echo [SUCCESS] Right-click menu installed! You can right-click any image to convert it.
) else (
    echo [ERROR] Failed to update registry.
)
pause
