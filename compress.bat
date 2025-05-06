@echo off
REM -------------------------------------------------
REM Affiche une bannière ASCII depuis banner.txt
REM -------------------------------------------------
cd /d "%~dp0"
if exist "banner.txt" (
  for /f "delims=" %%L in (banner.txt) do echo %%L
) else (
  echo ====== DEBUT DE LA COMPRESSION ======
)

REM Vérifie les arguments et l'existence du fichier/dossier
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

REM Normalise le chemin et obtient le nom de base
set "inputPath=%~1"
pushd "%~dp1"
for %%F in ("%~nx1") do (
    set "baseName=%%~nxF"
)

REM Lancement de la compression
echo.
echo Compression demandee pour : %baseName%
"C:\ProgramData\stelarc\FreeArc\bin\arc.exe" a "%baseName%.arc" "%baseName%" || (
    echo Erreur : La compression a echoue.
    popd
    pause
    exit /b 1
)

popd
echo.
echo Compression terminee avec succes.
pause
