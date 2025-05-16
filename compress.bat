@echo off
cd /d "%~dp0"

REM Affiche une bannière si disponible
if exist "banner.txt" (
  for /f "delims=" %%L in (banner.txt) do echo %%L
) else (
  echo ====== DEBUT DE LA COMPRESSION ======
)

REM Vérifie les arguments
if "%~1"=="" (
    echo Erreur : Aucun fichier ou dossier specifie.
    pause
    exit /b 1
)
if not exist "%~1" (
    echo Erreur : Le fichier ou dossier "%~1" est introuvable.
    pause
    exit /b 1
)

REM Chemin d'entrée et nom
set "inputPath=%~1"
pushd "%~dp1"
for %%F in ("%~nx1") do set "baseName=%%~nxF"

:FORMAT_MENU
echo.
echo Choisissez le format de compression :
echo   1. FreeArc (.arc)
echo   2. FreeArc avec compression 7z (.7z)
echo.
set /p "fmt=Entrez votre choix (1-2) : "

if "%fmt%"=="1" (
    set "ext=arc"
    goto ARC_LEVEL_MENU
) else if "%fmt%"=="2" (
    set "ext=7z"
    goto SEVENZ_LEVEL_MENU
) else (
    echo Choix invalide.
    goto FORMAT_MENU
)

:ARC_LEVEL_MENU
echo.
echo Choisissez le niveau de compression FreeArc :
echo   1. m1 (tres rapide)
echo   2. m2 (rapide)
echo   3. m3 (equilibre)
echo   4. m4 (bon compromis)
echo   5. m5 (meilleure compression)
echo   6. m6 (ultra)
echo   7. m7 (maximum)
echo   8. m8 (extreme)
echo   9. m9 (insane)
echo.
set /p "level=Votre choix (1-9) : "

if "%level%"=="1" set "mode=m1"
if "%level%"=="2" set "mode=m2"
if "%level%"=="3" set "mode=m3"
if "%level%"=="4" set "mode=m4"
if "%level%"=="5" set "mode=m5"
if "%level%"=="6" set "mode=m6"
if "%level%"=="7" set "mode=m7"
if "%level%"=="8" set "mode=m8"
if "%level%"=="9" set "mode=m9"

if not defined mode (
    echo Niveau invalide.
    goto ARC_LEVEL_MENU
)

echo.
echo Compression avec FreeArc (%mode%) vers .arc...
"C:\ProgramData\stelarc\FreeArc\arc.exe" a -%mode% "%baseName%.arc" "%baseName%" || goto ERR
goto SUCCESS

:SEVENZ_LEVEL_MENU
echo.
echo Choisissez le niveau de compression 7z (via FreeArc) :
echo   1. -mx1  (tres rapide)
echo   2. -mx3  (equilibre)
echo   3. -mx5  (meilleure compression)
echo   4. -mx7  (ultra)
echo   5. -mx9  (maximum)
echo.
set /p "mx=Votre choix (1-5) : "

if "%mx%"=="1" set "method=-m7z -mx1"
if "%mx%"=="2" set "method=-m7z -mx3"
if "%mx%"=="3" set "method=-m7z -mx5"
if "%mx%"=="4" set "method=-m7z -mx7"
if "%mx%"=="5" set "method=-m7z -mx -mc:lzma/lzma:max:512mb"

if not defined method (
    echo Choix invalide.
    goto SEVENZ_LEVEL_MENU
)

echo.
echo Compression avec FreeArc vers .7z avec %method%...
"C:\ProgramData\stelarc\FreeArc\arc.exe" a -dp %method% "%baseName%.7z" "%baseName%" || goto ERR
goto SUCCESS

:SUCCESS
echo.
echo Compression terminee avec succes en .%ext%
popd
pause
exit /b 0

:ERR
echo.
echo Erreur : La compression a echoue.
popd
pause
exit /b 1
