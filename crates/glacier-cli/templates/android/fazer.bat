@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul

rem O equivalente do Makefile para quem desenvolve NO Windows, onde nao ha make.
rem
rem     fazer setup       instala Java 17, Android SDK + NDK, alvos Rust e cargo-apk
rem     fazer doctor      confere o que falta, sem instalar nada
rem     fazer run         roda no desktop (hot-reload de .gvb/.gss)
rem     fazer build       APK de debug    -> target\debug\apk\{{nome_projeto}}.apk
rem     fazer release     APK de release  -> target\release\apk\{{nome_projeto}}.apk
rem     fazer install     instala no aparelho/emulador (adb)
rem     fazer launch      compila, instala, abre e segue o logcat
rem     fazer             esta ajuda
rem
rem ABI: `fazer build x86_64` (emulador) | aarch64 (padrao) | armv7 | x86
rem
rem Pre-requisito unico: o Rust (https://rustup.rs) com o MSVC. O resto o
rem `fazer setup` instala. Se ele instalar o Java por winget, feche e reabra o
rem terminal e rode `fazer setup` de novo: o PATH so atualiza em terminal novo.

set "APP={{nome_projeto}}"
set "PACKAGE=com.example.{{nome_crate}}"
set "ACTIVITY=android.app.NativeActivity"
set "TAG={{nome_crate}}"
set "KS_NAME={{nome_crate}}"

if not defined NDK_VERSION  set "NDK_VERSION=25.2.9519653"
if not defined API_LEVEL    set "API_LEVEL=33"
if not defined BUILD_TOOLS  set "BUILD_TOOLS=34.0.0"
if not defined CLANG_API    set "CLANG_API=26"
if not defined ANDROID_HOME set "ANDROID_HOME=%USERPROFILE%\android-sdk"
set "ANDROID_NDK_ROOT=%ANDROID_HOME%\ndk\%NDK_VERSION%"
set "CMDLINE_URL=https://dl.google.com/android/repository/commandlinetools-win-11076708_latest.zip"

set "PATH=%ANDROID_HOME%\cmdline-tools\latest\bin;%ANDROID_HOME%\platform-tools;%ANDROID_HOME%\emulator;%PATH%"

rem O cargo-apk configura o linker, mas o motor compila C/C++ (Luau, ring) pelo
rem crate `cc`, que precisa saber qual compilador usar no alvo Android. Sem isto
rem a build para no luau0-src com "failed to find tool".
set "NDK_BIN=%ANDROID_NDK_ROOT%\toolchains\llvm\prebuilt\windows-x86_64\bin"
set "CC_aarch64_linux_android=%NDK_BIN%\aarch64-linux-android%CLANG_API%-clang.cmd"
set "CXX_aarch64_linux_android=%NDK_BIN%\aarch64-linux-android%CLANG_API%-clang++.cmd"
set "AR_aarch64_linux_android=%NDK_BIN%\llvm-ar.exe"
set "CC_armv7_linux_androideabi=%NDK_BIN%\armv7a-linux-androideabi%CLANG_API%-clang.cmd"
set "CXX_armv7_linux_androideabi=%NDK_BIN%\armv7a-linux-androideabi%CLANG_API%-clang++.cmd"
set "AR_armv7_linux_androideabi=%NDK_BIN%\llvm-ar.exe"
set "CC_x86_64_linux_android=%NDK_BIN%\x86_64-linux-android%CLANG_API%-clang.cmd"
set "CXX_x86_64_linux_android=%NDK_BIN%\x86_64-linux-android%CLANG_API%-clang++.cmd"
set "AR_x86_64_linux_android=%NDK_BIN%\llvm-ar.exe"
set "CC_i686_linux_android=%NDK_BIN%\i686-linux-android%CLANG_API%-clang.cmd"
set "CXX_i686_linux_android=%NDK_BIN%\i686-linux-android%CLANG_API%-clang++.cmd"
set "AR_i686_linux_android=%NDK_BIN%\llvm-ar.exe"

rem ABI -> target do Rust (segundo argumento, opcional)
set "ABI=%~2"
if "%ABI%"=="" set "ABI=aarch64"
set "TARGET="
if /i "%ABI%"=="aarch64" set "TARGET=aarch64-linux-android"
if /i "%ABI%"=="armv7"   set "TARGET=armv7-linux-androideabi"
if /i "%ABI%"=="x86_64"  set "TARGET=x86_64-linux-android"
if /i "%ABI%"=="x86"     set "TARGET=i686-linux-android"
if not defined TARGET (
    echo   ABI desconhecida: %ABI%  ^(use aarch64, armv7, x86_64 ou x86^)
    exit /b 1
)

if "%~1"==""            goto :ajuda
if /i "%~1"=="ajuda"    goto :ajuda
if /i "%~1"=="help"     goto :ajuda
if /i "%~1"=="setup"    goto :setup
if /i "%~1"=="doctor"   goto :doctor
if /i "%~1"=="run"      goto :run
if /i "%~1"=="check"    goto :check
if /i "%~1"=="build"    goto :build
if /i "%~1"=="release"  goto :release
if /i "%~1"=="install"  goto :install
if /i "%~1"=="launch"   goto :launch
if /i "%~1"=="logcat"   goto :logcat
if /i "%~1"=="devices"  goto :devices
if /i "%~1"=="stop"     goto :stop
if /i "%~1"=="limpar"   goto :limpar

echo   Comando desconhecido: %~1
goto :ajuda

:ajuda
echo.
echo   %APP% - Android - o que da para fazer aqui
echo.
echo     fazer setup       instala Java 17, Android SDK + NDK, alvos Rust, cargo-apk
echo     fazer doctor      confere o ambiente, sem instalar nada
echo     fazer run         roda no desktop (hot-reload)
echo     fazer check       cargo check para o alvo Android
echo     fazer build       compila o APK de debug   (ABI opcional: fazer build x86_64)
echo     fazer release     compila o APK de release
echo     fazer install     instala no aparelho/emulador
echo     fazer launch      compila, instala, abre o app e segue o logcat
echo     fazer logcat      segue o log do app
echo     fazer devices     lista aparelhos conectados
echo     fazer stop        encerra o app no aparelho
echo     fazer limpar      cargo clean
echo.
echo   O que ainda nao funciona no Android: ANDROID_TODO.md
echo   No Linux, o equivalente e o Makefile: make help
echo.
exit /b 0

rem -- setup --------------------------------------------------------------------

:setup
echo.
echo   -- Rust --
where rustup >nul 2>&1
if errorlevel 1 (
    echo   ERRO: rustup nao encontrado. Instale em https://rustup.rs ^(com o MSVC^) e rode de novo.
    exit /b 1
)
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android || exit /b 1
where cargo-apk >nul 2>&1
if errorlevel 1 (
    echo   instalando o cargo-apk...
    cargo install cargo-apk --locked || exit /b 1
)

echo.
echo   -- Java 17+ --
call :java_ok
if errorlevel 1 (
    where winget >nul 2>&1
    if errorlevel 1 (
        echo   ERRO: Java 17+ nao encontrado e o winget nao existe.
        echo         Instale o Temurin 17: https://adoptium.net  e rode de novo.
        exit /b 1
    )
    winget install -e --id EclipseAdoptium.Temurin.17.JDK --accept-package-agreements --accept-source-agreements
    echo.
    echo   Java instalado. FECHE e REABRA este terminal ^(o PATH so atualiza em terminal novo^)
    echo   e rode  fazer setup  de novo para continuar.
    exit /b 0
)
echo   Java ok

echo.
echo   -- Android SDK / NDK --
if not exist "%ANDROID_HOME%\cmdline-tools\latest\bin\sdkmanager.bat" (
    echo   baixando as command-line tools...
    if not exist "%ANDROID_HOME%\cmdline-tools" mkdir "%ANDROID_HOME%\cmdline-tools"
    curl -fL -o "%ANDROID_HOME%\cmdline-tools.zip" "%CMDLINE_URL%" || exit /b 1
    rem O tar do Windows 10+ (bsdtar) abre .zip.
    tar -xf "%ANDROID_HOME%\cmdline-tools.zip" -C "%ANDROID_HOME%\cmdline-tools" || exit /b 1
    if exist "%ANDROID_HOME%\cmdline-tools\latest" rmdir /s /q "%ANDROID_HOME%\cmdline-tools\latest"
    move "%ANDROID_HOME%\cmdline-tools\cmdline-tools" "%ANDROID_HOME%\cmdline-tools\latest" >nul || exit /b 1
    del "%ANDROID_HOME%\cmdline-tools.zip"
)
rem Aceita as licencas: o sdkmanager pergunta "y/N" varias vezes.
(for /l %%i in (1,1,40) do @echo y) | call sdkmanager --licenses >nul 2>&1
call sdkmanager "platform-tools" "build-tools;%BUILD_TOOLS%" "platforms;android-%API_LEVEL%" "ndk;%NDK_VERSION%" || exit /b 1
echo   SDK em %ANDROID_HOME%

call :keystore || exit /b 1

echo.
echo   Setup completo. Proximo passo:  fazer build
echo   Confira com:  fazer doctor
exit /b 0

rem Java 17+ no PATH? errorlevel 0 = sim.
:java_ok
set "JMAJOR="
for /f "tokens=3" %%v in ('java -version 2^>^&1 ^| findstr /i "version"') do (
    if not defined JMAJOR set "JMAJOR=%%~v"
)
if not defined JMAJOR exit /b 1
for /f "tokens=1 delims=." %%m in ("!JMAJOR!") do set "JMAJOR=%%m"
if !JMAJOR! LSS 17 exit /b 1
exit /b 0

:keystore
if exist "%~dp0release.keystore" (
    echo   release.keystore ja existe
    exit /b 0
)
where keytool >nul 2>&1
if errorlevel 1 (
    echo   ERRO: keytool nao encontrado ^(Java fora do PATH^). Reabra o terminal.
    exit /b 1
)
keytool -genkeypair -keystore "%~dp0release.keystore" -alias %KS_NAME% -keyalg RSA -keysize 2048 -validity 10000 -storepass %KS_NAME% -keypass %KS_NAME% -dname "CN={{titulo}}, OU=Dev, O=Dev, L=Unknown, ST=Unknown, C=BR" || exit /b 1
echo   release.keystore criada ^(chave de teste - nao publique com ela^)
exit /b 0

rem -- doctor -------------------------------------------------------------------

:doctor
set /a FALTA=0
echo.
where rustup >nul 2>&1 && (call :ok "rustup") || (call :falta "rustup" "https://rustup.rs")
where cargo-apk >nul 2>&1 && (call :ok "cargo-apk") || (call :falta "cargo-apk" "fazer setup")
call :java_ok && (call :ok "Java 17+") || (call :falta "Java 17+" "fazer setup")
where sdkmanager.bat >nul 2>&1 && (call :ok "sdkmanager") || (call :falta "sdkmanager" "fazer setup")
where adb >nul 2>&1 && (call :ok "adb") || (call :falta "adb" "fazer setup")
if exist "%ANDROID_NDK_ROOT%\" (call :ok "NDK %NDK_VERSION%") else (call :falta "NDK %NDK_VERSION%" "fazer setup")
if exist "%ANDROID_HOME%\platforms\android-%API_LEVEL%\" (call :ok "plataforma android-%API_LEVEL%") else (call :falta "plataforma android-%API_LEVEL%" "fazer setup")
if exist "%ANDROID_HOME%\build-tools\%BUILD_TOOLS%\" (call :ok "build-tools %BUILD_TOOLS%") else (call :falta "build-tools %BUILD_TOOLS%" "fazer setup")
if exist "%~dp0release.keystore" (call :ok "release.keystore") else (call :falta "release.keystore" "fazer setup")
echo.
if !FALTA! EQU 0 (
    echo   Ambiente pronto.
    exit /b 0
)
echo   Falta algo - rode  fazer setup.
exit /b 1

:ok
echo   [ok]    %~1
exit /b 0

:falta
echo   [FALTA] %~1  -^> %~2
set /a FALTA+=1
exit /b 0

rem -- desktop ------------------------------------------------------------------

:run
cargo run
exit /b %errorlevel%

:check
cargo check --lib --target %TARGET%
exit /b %errorlevel%

rem -- APK ----------------------------------------------------------------------
rem `--lib` porque o pacote tem lib E bin: o APK carrega a cdylib.

:build
echo.
echo   -- APK debug ^(%TARGET%^) --
cargo apk build --lib --target %TARGET% || exit /b 1
echo.
echo   APK: target\debug\apk\%APP%.apk
exit /b 0

:release
call :keystore || exit /b 1
echo.
echo   -- APK release ^(%TARGET%^) --
cargo apk build --lib --release --target %TARGET% || exit /b 1
echo.
echo   APK: target\release\apk\%APP%.apk
exit /b 0

rem -- aparelho -----------------------------------------------------------------

:devices
adb devices -l
exit /b %errorlevel%

:install
call :build || exit /b 1
adb install -r target\debug\apk\%APP%.apk
exit /b %errorlevel%

:launch
call :install || exit /b 1
adb shell am start -n %PACKAGE%/%ACTIVITY% || exit /b 1
goto :logcat

:logcat
adb logcat -c
adb logcat -s %TAG%:V RustStdoutStderr:V AndroidRuntime:E
exit /b %errorlevel%

:stop
adb shell am force-stop %PACKAGE%
exit /b %errorlevel%

:limpar
cargo clean
exit /b %errorlevel%
