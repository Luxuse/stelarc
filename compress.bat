@echo off
setlocal enabledelayedexpansion

:: ----- Argument check -----
if "%~1"=="" (
  echo Error: No file or folder specified.
  echo Usage: %~nx0 ^<path\file_or_folder^>
  pause
  exit /b 1
)

:: ----- Save current directory -----
set "originalDir=%cd%"
pushd "%~dp1" || (
  echo Error: Cannot access "%~dp1".
  pause
  exit /b 1
)

:: ----- Input info -----
set "inputNameExt=%~nx1"
set "baseName=%~n1"

:FORMAT_MENU
cls
:: ───────── Bannière ─────────

if exist "%~dp0banner.txt" (

echo(

type "%~dp0banner.txt" 
echo.
echo Select compression format:
echo   1. FreeArc classic (.arc)
echo   2. 7-Zip classic   (.7z)
echo   3. Sharky          (.stel)
echo   4. Pixel speed     (.pixel)
echo.
set "fmt="
set /p "fmt=Choice (1-4): "
if "%fmt%"=="1" (
  set "ext=arc"
  goto ENCRYPTION_MENU
) else if "%fmt%"=="2" (
  set "ext=7z"
  goto SEVENZ_LEVEL_MENU
) else if "%fmt%"=="3" (
  set "ext=stel"
  goto SHARKY_LEVEL_MENU
) else if "%fmt%"=="4" (
  set "ext=pixel"
  goto PIXEL_LEVEL_MENU
) else (
  echo Invalid choice.
  pause
  goto FORMAT_MENU
)

:ENCRYPTION_MENU
cls
echo.
echo Use encryption?
echo   1. Yes
echo   2. No
echo.
set "enc_choice="
set /p "enc_choice=Choice (1-2): "
if "%enc_choice%"=="1" (
  goto ENCRYPTION_METHOD_MENU
) else if "%enc_choice%"=="2" (
  goto ARC_LEVEL_MENU
) else (
  echo Invalid choice.
  pause
  goto ENCRYPTION_MENU
)

:ENCRYPTION_METHOD_MENU
cls
echo.
echo Select encryption method:
echo   1. AES-256
echo   2. Blowfish
echo   3. Twofish
echo   4. Serpent
echo   5. Custom
echo.
set "enc_method="
set /p "choice=Choice (1-6): "
if "%choice%"=="1" set "enc_method=aes-256"
if "%choice%"=="2" set "enc_method=blowfish"
if "%choice%"=="3" set "enc_method=twofish"
if "%choice%"=="4" set "enc_method=serpent"
if "%choice%"=="5" (
  set /p "enc_method=Enter full method (e.g. aes+serpent/cfb+twofish): "
)
if not defined enc_method (
  echo Invalid choice.
  pause
  goto ENCRYPTION_METHOD_MENU
)
set /p "password=Enter password: "
goto ARC_LEVEL_MENU

:ARC_LEVEL_MENU
cls
echo.
echo FreeArc - select compression level:
echo   1. m1 (fast)
echo   2. m3 (balanced)
echo   3. m5 (good)
echo   4. m7 (better)
echo   5. m9 (ultra)
echo   6. Berserk (experimental)
echo.
set "mode="
set /p "lvl=Choice (1-6): "
if "%lvl%"=="1" set "mode=m1d"
if "%lvl%"=="2" set "mode=m3d"
if "%lvl%"=="3" set "mode=m5d"
if "%lvl%"=="4" set "mode=m7d"
if "%lvl%"=="5" set "mode=m9d"
if "%lvl%"=="6" set "mode=Casca-alpha"
if not defined mode (
  echo Invalid level.
  pause
  goto ARC_LEVEL_MENU
)
echo.
echo Compressing with FreeArc (%mode%) to %baseName%.%ext%...
if defined enc_method (
  set "enc_cmd=--encryption=%enc_method% --password="!password!""
) else (
  set "enc_cmd="
)
"C:\ProgramData\stelarc\FreeArc\arc.exe" a -%mode% %enc_cmd% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

:SEVENZ_LEVEL_MENU
cls
echo.
echo 7-Zip - select compression level:
echo   1. -mx1 (fast)
echo   2. -mx3 (balanced)
echo   3. -mx5 (better)
echo   4. -mx7 (ultra)
echo   5. -mx9 (max)
echo.
set "mx="
set /p "lvl7=Choice (1-5): "
if "%lvl7%"=="1" set "mx=-mx1"
if "%lvl7%"=="2" set "mx=-mx3"
if "%lvl7%"=="3" set "mx=-mx5"
if "%lvl7%"=="4" set "mx=-mx7"
if "%lvl7%"=="5" set "mx=-mx9"
if not defined mx (
  echo Invalid choice.
  pause
  goto SEVENZ_LEVEL_MENU
)
echo.
echo Compressing with 7-Zip (%mx%) to %baseName%.%ext%...
"C:\ProgramData\stelarc\FreeArc\7z.exe" a %mx% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

:SHARKY_LEVEL_MENU
cls
echo.
echo Sharky - select compression level:
echo   1. Fast      (XZ=1, Zstd=2)
echo   2. Balanced  (XZ=4, Zstd=7)
echo   3. Mid       (XZ=6, Zstd=13)
echo   4. Better    (XZ=9, Zstd=18)
echo   5. Insane    (XZ=9, Zstd=22)
echo.
set "xz_preset="
set "zstd_level="
set /p "lvl=Choice (1-5): "
if "%lvl%"=="1" ( set "xz_preset=1" & set "zstd_level=2" )
if "%lvl%"=="2" ( set "xz_preset=4" & set "zstd_level=7" )
if "%lvl%"=="3" ( set "xz_preset=6" & set "zstd_level=13" )
if "%lvl%"=="4" ( set "xz_preset=9" & set "zstd_level=18" )
if "%lvl%"=="5" ( set "xz_preset=9" & set "zstd_level=22" )
if not defined xz_preset (
  echo Invalid choice.
  pause
  goto SHARKY_LEVEL_MENU
)
echo.
echo Compressing !inputNameExt! to !baseName!.!ext!...
"C:\ProgramData\stelarc\sharky\sharky.exe" ^
  -c -x !xz_preset! -z !zstd_level! -i "!inputNameExt!" -o "!baseName!.!ext!" || goto ERR
goto SUCCESS

:PIXEL_LEVEL_MENU
cls
echo.
echo Pixel - select compression level:
echo   1. lz4    (very fast)
echo   2. zstd   (fast)
echo   3. M3     (Medium+)
echo   4. Razor  (very slow)
echo.
set "pixel_opts="
set /p "lvl=Choice (1-4): "
if "%lvl%"=="1" set "pixel_opts=-mlz4"
if "%lvl%"=="2" set "pixel_opts=-mzstd:5"
if "%lvl%"=="3" set "pixel_opts=-m3d -s; -mc:rep/maxsrep -mc$default,$obj:+precompj"
if "%lvl%"=="4" set "pixel_opts=-mrazor"
if not defined pixel_opts (
  echo Invalid choice.
  pause
  goto PIXEL_LEVEL_MENU
)
echo.
echo Compressing with Pixel (%pixel_opts%)...
"C:\ProgramData\stelarc\FreeArc\arc.exe" a %pixel_opts% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

:SUCCESS
echo.
echo Compression complete.
popd
pause
exit /b 0

:ERR
echo.
echo Compression failed!
popd
pause
exit /b 1
