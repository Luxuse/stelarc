@echo off
rem Correction de "lexperimantale compresse tout larborecense"

cd /d "%~dp0"

REM ───────────── Bannière ────────────────
if exist "banner.txt" (
for /f "delims=" %%L in (banner.txt) do echo %%L
) else (
echo ====== DEBUT DE LA COMPRESSION ======
)

REM ───────────── Vérification des arguments ─────────────
if "%~1"=="" (
echo Erreur : Aucun fichier ou dossier specifie.
echo Utilisation : %~nx0 ^<fichier_ou_dossier^>
pause & exit /b 1
)

REM --- Stocker le chemin d'origine avant pushd ---
set "originalDir=%cd%"

REM --- Naviguer vers le répertoire de l'entrée ---
pushd "%~dp1" || (
    echo Erreur : Impossible d'acceder au repertoire "%~dp1".
    pause & exit /b 1
)

REM --- Variables chemin/nom ---
set "inputPath=%~1"      REM Chemin complet original
set "inputNameExt=%~nx1" REM Nom et extension de l'entrée
set "baseName=%~n1"      REM Nom de base sans extension

REM ───────────── Menu de format ─────────────
:FORMAT_MENU
echo.
echo Choisissez le format de compression :
echo   1. FreeArc classique (.arc)
echo   2. 7-Zip classique (.7z)
echo   3. FreeArc avec compression experimentale (.stel)
echo.
set "fmt=" REM Réinitialise la variable pour éviter l'utilisation d'une valeur précédente
set /p "fmt=Entrez votre choix (1-3) : "

if "%fmt%"=="1" (
    set "ext=arc"
    goto ARC_LEVEL_MENU
) else if "%fmt%"=="2" (
    set "ext=7z"
    goto SEVENZ_LEVEL_MENU
) else if "%fmt%"=="3" (
    set "ext=stel"
    goto EXPERIMENTAL_ARC_MENU
) else (
    echo Choix invalide.
    goto FORMAT_MENU
)

REM ───────────── Menu FreeArc classique ─────────────
:ARC_LEVEL_MENU
echo.
echo Choisissez le niveau de compression FreeArc classique :
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
set "mode=" REM Réinitialise la variable
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

REM ───────────── Menu 7-Zip classique ─────────────
:SEVENZ_LEVEL_MENU
echo.
echo Choisissez le niveau de compression 7z (7-Zip classique) :
echo   1. -mx1  (tres rapide)
echo   2. -mx3  (equilibre)
echo   3. -mx5  (meilleure compression)
echo   4. -mx7  (ultra)
echo   5. -mx9  (maximum)
echo.
set "mx=" REM Réinitialise la variable
set /p "level7z=Votre choix (1-5) : "

if "%level7z%"=="1" set "mx=-mx1"
if "%level7z%"=="2" set "mx=-mx3"
if "%level7z%"=="3" set "mx=-mx5"
if "%level7z%"=="4" set "mx=-mx7"
if "%level7z%"=="5" set "mx=-mx9"

if not defined mx (
    echo Choix invalide.
    goto SEVENZ_LEVEL_MENU
)

echo.
echo Compression avec 7-Zip %mx% vers .%ext%...
REM Modifier le chemin de 7z.exe si besoin
REM L'entrée à compresser est %inputNameExt% car pushd est actif
"C:\ProgramData\stelarc\FreeArc\7z.exe" a %mx% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

REM ───────────── Menu FreeArc expérimental (ZSTD) ─────────────
:EXPERIMENTAL_ARC_MENU
echo.
echo Choisissez le niveau de compression experimental FreeArc + ZSTD :
echo   1. Niveau 1 (Rapide - ZSTD:6)
echo   2. Niveau 2 (Equilibre - ZSTD:14)
echo   3. Niveau 3 (Bon compromis - ZSTD:17)
echo   4. Niveau 4 (Meilleure compression - ZSTD:20)
echo   5. Niveau 5 (Ultra - ZSTD:22)
echo.
set "method=" REM Réinitialise la variable
set /p "level_exp=Votre choix (1-5) : "

REM Adjusted methods based on common FreeArc usage; max_dict can be specified but often inferred
if "%level_exp%"=="1" set "method=m1 -mc\:lzma2/lzma2\:d4m -mzstd:6"
if "%level_exp%"=="2" set "method=m3 -s; -mc\:lzma2/lzma2\:d64m -mzstd:14"
if "%level_exp%"=="3" set "method=m5 -s; -mc\:lzma2/lzma2\:d192m -mzstd:17"
if "%level_exp%"=="4" set "method=m7 -s; -mc\:lzma2/lzma2\:d384m -mzstd:20"
if "%level_exp%"=="5" set "method=m9 -s; -mc\:lzma2/lzma2\:d768m -mzstd:22"

if not defined method (
    echo Choix invalide.
    goto EXPERIMENTAL_ARC_MENU
)

echo.
echo Compression experimentale FreeArc + ZSTD avec %method% vers .%ext%...
REM L'entrée à compresser est %inputNameExt% car pushd est actif
"C:\ProgramData\stelarc\FreeArc\arc.exe" a -m%method% "%baseName%.%ext%" "%inputNameExt%" || goto ERR
goto SUCCESS

REM ───────────── Fin ─────────────
:SUCCESS
echo.
echo Compression terminee avec succes en .%ext%
popd
pause
exit /b 0

:ERR
echo.
echo Erreur : la compression a echoue.
popd
pause
exit /b 1