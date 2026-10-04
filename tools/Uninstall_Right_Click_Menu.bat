@echo off
setlocal

echo Removing "Convert to Pixel Art" from Right-Click Menu...
reg delete "HKCU\Software\Classes\SystemFileAssociations\image\shell\PixelArt" /f >nul 2>&1

echo [SUCCESS] Removed successfully.
pause
