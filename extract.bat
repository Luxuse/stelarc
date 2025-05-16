@echo off
chcp 65001 > nul

cls

REM === Affiche la bannière ASCII si banner.txt existe ===
if exist "%~dp0banner.txt" (
    echo(
    type "%~dp0banner.txt"
    echo(
) else (
    echo ====== DEBUT DE L’EXTRACTION ======
)

REM === Vérifie l’argument ===
if "%~1"=="" (
    echo Erreur : Aucun fichier .arc spécifié.
    pause
    exit /b 1
)

REM === Vérifie l’existence du fichier ===
if not exist "%~1" (
    echo Erreur : Le fichier "%~1" est introuvable.
    pause
    exit /b 1
)

REM === Normalise le chemin du fichier à extraire et récupère sa taille ===
set "inputPath=%~1"
for %%F in ("%inputPath%") do (
    set "normalizedPath=%%~fF"
    set "arcSize=%%~zF"
)

REM === Affiche la taille de l’archive ===
set /a arcSizeMB=%arcSize% / 1048576
echo(
echo Fichier détecté : %normalizedPath%
echo Taille de l’archive : %arcSizeMB% Mo
echo(
echo Début de l’extraction, veuillez patienter...

REM === Exécution de l’extraction ===
"C:\ProgramData\stelarc\FreeArc\arc.exe" x "%normalizedPath%" -o+ || (
    echo Erreur : L’extraction a échoué.
    pause
    exit /b 1
)

echo(
echo Extraction terminée avec succès.
pause

