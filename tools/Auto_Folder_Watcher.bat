@echo off
setlocal

set "BASE_DIR=%~dp0..\"
set "EXE_PATH=%BASE_DIR%target\release\pixelgenrator.exe"
set "WATCH_DIR=%BASE_DIR%Drop_Images_Here"
set "OUTPUT_DIR=%BASE_DIR%Pixel_Outputs"

if not exist "%WATCH_DIR%" mkdir "%WATCH_DIR%"
if not exist "%OUTPUT_DIR%" mkdir "%OUTPUT_DIR%"

echo =======================================================
echo   🚀 PIXELGEN AUTO FOLDER WATCHER
echo =======================================================
echo   Drop or paste images into: %WATCH_DIR%
echo   Outputs will be saved in : %OUTPUT_DIR%
echo =======================================================
echo   Watching for images... (Press Ctrl+C to stop)
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$watch = '%WATCH_DIR%';" ^
    "$out = '%OUTPUT_DIR%';" ^
    "$exe = '%EXE_PATH%';" ^
    "$fsw = New-Object IO.FileSystemWatcher $watch, '*.*';" ^
    "$fsw.EnableRaisingEvents = $true;" ^
    "$action = {" ^
    "    $path = $Event.SourceEventArgs.FullPath;" ^
    "    $ext = [IO.Path]::GetExtension($path).ToLower();" ^
    "    if ($ext -match '\.(png|jpg|jpeg|webp|bmp)$') {" ^
    "        Start-Sleep -Milliseconds 400;" ^
    "        $stem = [IO.Path]::GetFileNameWithoutExtension($path);" ^
    "        $outPath = Join-Path $out ($stem + '_pixelart.png');" ^
    "        Write-Host \"[+] Converting: $([IO.Path]::GetFileName($path))\";" ^
    "        & $exe $path -o $outPath;" ^
    "    }" ^
    "};" ^
    "Register-ObjectEvent $fsw 'Created' -Action $action | Out-Null;" ^
    "while ($true) { Start-Sleep -Seconds 1 }"
