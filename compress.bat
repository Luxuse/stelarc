@echo off
setlocal enabledelayedexpansion

REM ───────────── Bannière ─────────────
if exist "%~dp0banner.txt" (
  for /f "delims=" %%L in ("%~dp0banner.txt") do echo %%L
) else (
  echo ====== DEBUT DE LA COMPRESSION ======
)

REM ───────────── Vérification des arguments ─────────────
if "%~1"=="" (
  echo Erreur : Aucun fichier ou dossier specifiw.
  echo Usage : %~nx0 ^<chemin\fichier_ou_dossier^>
  pause
  exit /b 1
)

REM --- On conserve le dossier d'origine et on se deplace ---
set "originalDir=%cd%"
pushd "%~dp1" || (
  echo Erreur : Impossible d'acceder à "%~dp1".
  pause
  exit /b 1
)

REM --- Variables d’entree ---
set "inputNameExt=%~nx1"
set "baseName=%~n1"

:FORMAT_MENU
cls
echo.
echo Choisissez le format de compression :
echo   1. FreeArc classique (.arc)
echo   2. 7-Zip classique    (.7z)
echo   3. Sharky (.stel)
echo.
set "fmt="
set /p "fmt=Entrez votre choix (1-4) : "

if "%fmt%"=="1" (
  set "ext=arc"
  goto ARC_LEVEL_MENU
) else if "%fmt%"=="2" (
  set "ext=7z"
  goto SEVENZ_LEVEL_MENU
) else if "%fmt%"=="3" (
  set "ext=stel"
  goto SHARKY_LEVEL_MENU
) else (
  echo Choix invalide.
  pause
  goto FORMAT_MENU
)

:ARC_LEVEL_MENU
cls
echo.
echo FreeArc classique - choisissez le niveau :
echo   1. m1 (tres rapide)
echo   2. m3 (equilibre)
echo   3. m5 (bon compromis)
echo   4. m7 (meilleure compression)
echo   5. m9 (ultra)
echo.
set "mode="
set /p "lvl=Votre choix (1-5) : "
if "%lvl%"=="1" set "mode=m1d"
if "%lvl%"=="2" set "mode=m3d"
if "%lvl%"=="3" set "mode=m5d"
if "%lvl%"=="4" set "mode=m7d"
if "%lvl%"=="5" set "mode=m9d"
if not defined mode (
  echo Niveau invalide.
  pause
  goto ARC_LEVEL_MENU
)
echo.
echo Compression FreeArc %mode% vers %baseName%.%ext%...
"C:\ProgramData\stelarc\FreeArc\arc.exe" a -%mode% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

:SEVENZ_LEVEL_MENU
cls
echo.
echo 7-Zip classique - choisissez le niveau :
echo   1. -mx1 (rapide)
echo   2. -mx3 (equilibre)
echo   3. -mx5 (meilleure)
echo   4. -mx7 (ultra)
echo   5. -mx9 (maximum)
echo.
set "mx="
set /p "lvl7=Votre choix (1-5) : "
if "%lvl7%"=="1" set "mx=-mx1"
if "%lvl7%"=="2" set "mx=-mx3"
if "%lvl7%"=="3" set "mx=-mx5"
if "%lvl7%"=="4" set "mx=-mx7"
if "%lvl7%"=="5" set "mx=-mx9"
if not defined mx (
  echo Choix invalide.
  pause
  goto SEVENZ_LEVEL_MENU
)
echo.
echo Compression 7z %mx% vers %baseName%.%ext%...
"C:\ProgramData\stelarc\FreeArc\7z.exe" a %mx% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

:SHARKY_LEVEL_MENU
cls
echo.
echo Sharky - choisissez le niveau de compression :
echo   1. Rapide      (XZ preset=1,  Zstd=2)
echo   2. Equilibre   (XZ preset=4,  Zstd=7)
echo   3. Compromis   (XZ preset=6,  Zstd=13)
echo   4. Meilleure   (XZ preset=9,  Zstd=18)
echo   5. Insane      (XZ preset=9,  Zstd=22)
echo.
set "xz_preset="
set "zstd_level="
set /p "lvl=Votre choix (1-5) : "

if "%lvl%"=="1" (
    set "xz_preset=1"
    set "zstd_level=2"
) else if "%lvl%"=="2" (
    set "xz_preset=4"
    set "zstd_level=7"
) else if "%lvl%"=="3" (
    set "xz_preset=6"
    set "zstd_level=13"
) else if "%lvl%"=="4" (
    set "xz_preset=9"
    set "zstd_level=18"
) else if "%lvl%"=="5" (
    set "xz_preset=9"
    set "zstd_level=22"
)

if not defined xz_preset (
    echo Choix invalide.
    pause
    goto SHARKY_LEVEL_MENU
)

echo.
echo Vous avez choisi : XZ preset=!xz_preset!, Zstd level=!zstd_level!.
echo Compression de !inputNameExt! vers !baseName!.!ext!...
"C:\ProgramData\stelarc\sharky\sharky.exe" ^
    -c ^
    -x !xz_preset! ^
    -z !zstd_level! ^
    -i "!inputNameExt!" ^
    -o "!baseName!.!ext!" || goto ERR


:SUCCESS
echo.
echo Compression terminee avec succes : %baseName%.%ext%
popd
pause
exit /b 0

:ERR
echo.
echo Erreur lors de la compression.
popd
pause
exit /b 1
