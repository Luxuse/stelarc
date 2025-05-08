@echo off
setlocal EnableDelayedExpansion
hcp 65001 > nul
cls



if "%~1"=="" (
    echo Erreur : Aucun fichier specifie
    pause
    exit /b 1
)

if not exist "%~1" (
    echo Erreur : Le fichier "%~1" n'existe pas
    pause
    exit /b 1
)

:menu
cls
echo.
echo Fichier : %~1
echo.

REM === Affiche la bannière ASCII si banner.txt existe ===
if exist "%~dp0banner.txt" (
    echo(
    type "%~dp0banner.txt"
    echo(
) 

echo Choisissez le type de hash :
echo.
echo [1] CRC32
echo [2] BLAKE3
echo [3] MD5
echo [4] SHA-256
echo [5] SHA3-256
echo [6] Tout calculer
echo [0] Quitter
echo.

choice /C 1234560 /N /M "Votre choix (0-6) : "

set HASH_TYPE=
if !errorlevel! == 7 exit /b 0
if !errorlevel! == 6 (
    set "HASH_TYPE=all"
) else if !errorlevel! == 5 (
    set "HASH_TYPE=sha3"
) else if !errorlevel! == 4 (
    set "HASH_TYPE=sha256"
) else if !errorlevel! == 3 (
    set "HASH_TYPE=md5"
) else if !errorlevel! == 2 (
    set "HASH_TYPE=blake3"
) else if !errorlevel! == 1 (
    set "HASH_TYPE=crc32"
)

echo.
echo ----------------------------------------
"C:\ProgramData\stelarc\stelarc.exe" --hash "%~1" --type !HASH_TYPE!
echo ----------------------------------------
pause
goto menu