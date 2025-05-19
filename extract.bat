@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
cls

:: ───────── Bannière ─────────
if exist "%~dp0banner.txt" (
    echo(
    type "%~dp0banner.txt"
    echo(
) else (
    echo ====== DEBUT DE L’EXTRACTION ======
)

:: ───────── Vérifications ─────────
if "%~1"=="" (
    echo Erreur : aucun fichier spécifié.
    pause & exit /b 1
)
if not exist "%~1" (
    echo Erreur : le fichier "%~1" est introuvable.
    pause & exit /b 1
)

:: Infos fichier
for %%F in ("%~1") do (
    set "normalizedPath=%%~fF"
    set "arcSize=%%~zF"
    set "fileExt=%%~xF"
    set "fileDir=%%~dpF"
)

set /a arcSizeMB=arcSize/1048576
echo(
echo Fichier : !normalizedPath!
echo Taille  : !arcSizeMB! Mo
echo(
echo Début de l’extraction…
echo(

:: ───────── Extraction suivant l’extension ─────────
if /i "!fileExt!"==".7z" (
    echo Extraction avec 7-Zip…
    "C:\ProgramData\stelarc\FreeArc\7z.exe" x "!normalizedPath!" -o"!fileDir!" -y || goto :ERR
) else if /i "!fileExt!"==".zip" (
    echo Extraction avec 7-Zip…
    "C:\ProgramData\stelarc\FreeArc\7z.exe" x "!normalizedPath!" -o"!fileDir!" -y || goto :ERR
) else if /i "!fileExt!"==".stel" (
    echo Extraction avec Sharky…
    rem --- retirer le \ final ---
    set "outDir=!fileDir:~0,-1!"
    "C:\ProgramData\stelarc\sharky\sharky.exe" -d -i "!normalizedPath!" -o "!outDir!" || goto :ERR
) else (

    echo Extraction avec FreeArc…
    "C:\ProgramData\stelarc\FreeArc\arc.exe" x "!normalizedPath!" -o+ || goto :ERR
)

:: ───────── Fin ─────────
echo(
echo Extraction terminee avec succes.
pause
exit /b 0

:ERR
echo(
echo Erreur : l’extraction a echoue.
pause
exit /b 1
