VERSION := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
APP     := {{nome_projeto}}

# ══════════════════════════════════════════════════════════════════════════════
# {{nome_projeto}} — app Android com glacier-ui
#
#   make setup      prepara a máquina (Java 17, Android SDK + NDK, alvos Rust, cargo-apk)
#   make doctor     confere o que falta, sem instalar nada
#   make run        desktop, com hot-reload (é aqui que se desenvolve)
#   make build      APK de debug       → target/debug/apk/$(APP).apk
#   make release    APK de release     → target/release/apk/$(APP).apk
#   make install    instala no aparelho/emulador (adb)
#   make launch     compila, instala e abre no aparelho, e segue o logcat
#
# No Windows: fazer.bat (mesmos comandos, sem make).
# O que ainda não funciona no Android: ANDROID_TODO.md.
# ══════════════════════════════════════════════════════════════════════════════

BOLD  := \033[1m
RESET := \033[0m
GREEN := \033[32m
CYAN  := \033[36m
RED   := \033[31m

export CARGO_TERM_COLOR := never

.DEFAULT_GOAL := help

# ── Versões ───────────────────────────────────────────────────────────────────
#
# Todas sobrescrevíveis na linha de comando: `make setup NDK_VERSION=26.3.11579264`.

JAVA_MIN     := 17
JAVA_SDKMAN  := 17.0.13-tem
NDK_VERSION  ?= 25.2.9519653
API_LEVEL    ?= 33
BUILD_TOOLS  ?= 34.0.0
# O `minSdk` do APK (Cargo.toml) é 26; é também o nível do clang que compila o C/C++.
CLANG_API    ?= 26
CMDLINE_URL  ?= https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip

ANDROID_HOME     ?= $(HOME)/android-sdk
ANDROID_NDK_ROOT ?= $(ANDROID_HOME)/ndk/$(NDK_VERSION)
SDKMAN_DIR       ?= $(HOME)/.sdkman

ANDROID_TARGETS := aarch64-linux-android armv7-linux-androideabi \
                   x86_64-linux-android i686-linux-android

# `make build ABI=x86_64` para o emulador; `ABI=armv7` para aparelho de 32 bits.
ABI ?= aarch64
ifeq ($(ABI),aarch64)
  TARGET := aarch64-linux-android
else ifeq ($(ABI),armv7)
  TARGET := armv7-linux-androideabi
else ifeq ($(ABI),x86_64)
  TARGET := x86_64-linux-android
else ifeq ($(ABI),x86)
  TARGET := i686-linux-android
else
  $(error ABI desconhecida: '$(ABI)' — use aarch64, armv7, x86_64 ou x86)
endif

PACKAGE  := com.example.{{nome_crate}}
ACTIVITY := android.app.NativeActivity
APK_DEBUG   := target/debug/apk/$(APP).apk
APK_RELEASE := target/release/apk/$(APP).apk

# ── Ambiente de todas as receitas ────────────────────────────────────────────

export ANDROID_HOME
export ANDROID_NDK_ROOT
export PATH := $(ANDROID_HOME)/cmdline-tools/latest/bin:$(ANDROID_HOME)/platform-tools:$(ANDROID_HOME)/emulator:$(PATH)

# O Java que o `make setup` instala via SDKMAN, se existir. Um JDK do sistema
# (apt install openjdk-17-jdk) já está no PATH e dispensa isto.
ifneq ($(wildcard $(SDKMAN_DIR)/candidates/java/current),)
  JAVA_HOME ?= $(SDKMAN_DIR)/candidates/java/current
  export JAVA_HOME
  export PATH := $(JAVA_HOME)/bin:$(PATH)
endif

# O `cargo-apk` configura o linker, mas o motor compila C/C++ (Luau, ring) pelo
# crate `cc`, que precisa saber qual compilador usar para o alvo Android. Sem
# isto a build para no `luau0-src` com "failed to find tool".
NDK_BIN := $(ANDROID_NDK_ROOT)/toolchains/llvm/prebuilt/linux-x86_64/bin
export CC_aarch64_linux_android      := $(NDK_BIN)/aarch64-linux-android$(CLANG_API)-clang
export CXX_aarch64_linux_android     := $(NDK_BIN)/aarch64-linux-android$(CLANG_API)-clang++
export AR_aarch64_linux_android      := $(NDK_BIN)/llvm-ar
export CC_armv7_linux_androideabi    := $(NDK_BIN)/armv7a-linux-androideabi$(CLANG_API)-clang
export CXX_armv7_linux_androideabi   := $(NDK_BIN)/armv7a-linux-androideabi$(CLANG_API)-clang++
export AR_armv7_linux_androideabi    := $(NDK_BIN)/llvm-ar
export CC_x86_64_linux_android       := $(NDK_BIN)/x86_64-linux-android$(CLANG_API)-clang
export CXX_x86_64_linux_android      := $(NDK_BIN)/x86_64-linux-android$(CLANG_API)-clang++
export AR_x86_64_linux_android       := $(NDK_BIN)/llvm-ar
export CC_i686_linux_android         := $(NDK_BIN)/i686-linux-android$(CLANG_API)-clang
export CXX_i686_linux_android        := $(NDK_BIN)/i686-linux-android$(CLANG_API)-clang++
export AR_i686_linux_android         := $(NDK_BIN)/llvm-ar

# ── Setup da máquina ──────────────────────────────────────────────────────────

.PHONY: setup
setup: setup-host setup-java setup-android-sdk setup-rust keystore ## Instala TUDO que falta para compilar o APK (primeira vez)
	@echo ""
	@echo "$(GREEN)Setup completo.$(RESET) Próximo passo:  make build"
	@echo "Confira com:  make doctor"

.PHONY: setup-host
setup-host: ## Pacotes do sistema (compilador C/C++, curl, unzip) via apt/dnf/pacman
	@echo "$(BOLD)── Pacotes do sistema ──$(RESET)"
	@if command -v cc >/dev/null 2>&1 && command -v curl >/dev/null 2>&1 \
	    && command -v unzip >/dev/null 2>&1 && command -v pkg-config >/dev/null 2>&1; then \
	  echo "  já instalados ✓"; \
	elif command -v apt-get >/dev/null 2>&1; then \
	  sudo apt-get update && sudo apt-get install -y build-essential pkg-config curl unzip zip git; \
	elif command -v dnf >/dev/null 2>&1; then \
	  sudo dnf install -y gcc gcc-c++ make pkgconf-pkg-config curl unzip zip git; \
	elif command -v pacman >/dev/null 2>&1; then \
	  sudo pacman -S --needed --noconfirm base-devel pkgconf curl unzip zip git; \
	else \
	  echo "$(RED)gerenciador de pacotes não reconhecido$(RESET) — instale à mão: compilador C/C++, pkg-config, curl, unzip"; \
	  exit 1; \
	fi

.PHONY: setup-java
setup-java: ## Java 17+ (exigido pelo sdkmanager); instala via SDKMAN se faltar
	@echo "$(BOLD)── Java $(JAVA_MIN)+ ──$(RESET)"
	@major=$$(java -version 2>&1 | sed -n '1s/.*"\([0-9]*\)[."].*/\1/p'); \
	if [ -n "$$major" ] && [ "$$major" -ge $(JAVA_MIN) ] 2>/dev/null; then \
	  echo "  Java $$major já instalado ✓"; \
	else \
	  if [ ! -f "$(SDKMAN_DIR)/bin/sdkman-init.sh" ]; then \
	    echo "  instalando o SDKMAN…"; \
	    curl -fsSL "https://get.sdkman.io" | bash; \
	  fi; \
	  . "$(SDKMAN_DIR)/bin/sdkman-init.sh" && sdk install java $(JAVA_SDKMAN) < /dev/null; \
	  echo "  Java $(JAVA_SDKMAN) instalado ✓"; \
	fi

.PHONY: setup-android-sdk
setup-android-sdk: ## Android SDK: platform-tools, build-tools, plataforma e NDK
	@echo "$(BOLD)── Android SDK / NDK ──$(RESET)"
	@if [ ! -x "$(ANDROID_HOME)/cmdline-tools/latest/bin/sdkmanager" ]; then \
	  echo "  baixando as command-line tools…"; \
	  mkdir -p "$(ANDROID_HOME)/cmdline-tools"; \
	  curl -fL --progress-bar "$(CMDLINE_URL)" -o "$(ANDROID_HOME)/cmdline-tools.zip"; \
	  unzip -q -o "$(ANDROID_HOME)/cmdline-tools.zip" -d "$(ANDROID_HOME)/cmdline-tools"; \
	  rm -rf "$(ANDROID_HOME)/cmdline-tools/latest"; \
	  mv "$(ANDROID_HOME)/cmdline-tools/cmdline-tools" "$(ANDROID_HOME)/cmdline-tools/latest"; \
	  rm -f "$(ANDROID_HOME)/cmdline-tools.zip"; \
	fi
	@yes | sdkmanager --licenses >/dev/null 2>&1 || true
	sdkmanager "platform-tools" "build-tools;$(BUILD_TOOLS)" "platforms;android-$(API_LEVEL)" "ndk;$(NDK_VERSION)"
	@echo "  SDK em $(ANDROID_HOME) ✓"

.PHONY: setup-rust
setup-rust: ## Alvos Android do rustup e o cargo-apk
	@echo "$(BOLD)── Rust ──$(RESET)"
	@command -v rustup >/dev/null 2>&1 || \
	  (echo "$(RED)rustup não encontrado$(RESET) — https://rustup.rs" && exit 1)
	rustup target add $(ANDROID_TARGETS)
	@command -v cargo-apk >/dev/null 2>&1 || \
	  (echo "  instalando o cargo-apk…" && cargo install cargo-apk --locked)
	@echo "  toolchain Rust ✓"

.PHONY: keystore
keystore: ## Cria release.keystore (chave SÓ para testar `make release`)
	@if [ -f release.keystore ]; then \
	  echo "  release.keystore já existe ✓"; \
	else \
	  command -v keytool >/dev/null 2>&1 || \
	    (echo "$(RED)keytool não encontrado$(RESET) — rode 'make setup-java'" && exit 1); \
	  keytool -genkeypair -keystore release.keystore -alias {{nome_crate}} \
	    -keyalg RSA -keysize 2048 -validity 10000 \
	    -storepass {{nome_crate}} -keypass {{nome_crate}} \
	    -dname "CN={{titulo}}, OU=Dev, O=Dev, L=Unknown, ST=Unknown, C=BR"; \
	  echo "  release.keystore criada ✓ (chave de teste — não publique com ela)"; \
	fi

# `doctor` não instala nada: lista o que está e o que falta, e falha se faltar algo.
.PHONY: doctor
doctor: ## Confere o ambiente (Java, SDK, NDK, alvos Rust, cargo-apk, adb)
	@ok=0; \
	check() { if eval "$$2" >/dev/null 2>&1; then printf "  $(GREEN)✔$(RESET) %s\n" "$$1"; else printf "  $(RED)✘$(RESET) %s  → %s\n" "$$1" "$$3"; ok=1; fi; }; \
	check "rustup"            "command -v rustup"                                   "https://rustup.rs"; \
	check "cargo-apk"         "command -v cargo-apk"                                "make setup-rust"; \
	check "alvos Rust Android" "[ \$$(rustup target list --installed | grep -c android) -ge 4 ]" "make setup-rust"; \
	check "Java $(JAVA_MIN)+" "[ \"\$$(java -version 2>&1 | sed -n '1s/.*\"\\([0-9]*\\)[.\"].*/\\1/p')\" -ge $(JAVA_MIN) ]" "make setup-java"; \
	check "sdkmanager"        "command -v sdkmanager"                               "make setup-android-sdk"; \
	check "adb"               "command -v adb"                                      "make setup-android-sdk"; \
	check "NDK $(NDK_VERSION)" "[ -d '$(ANDROID_NDK_ROOT)' ]"                       "make setup-android-sdk"; \
	check "clang do NDK"      "[ -x '$(CC_aarch64_linux_android)' ]"                "make setup-android-sdk"; \
	check "plataforma android-$(API_LEVEL)" "[ -d '$(ANDROID_HOME)/platforms/android-$(API_LEVEL)' ]" "make setup-android-sdk"; \
	check "build-tools $(BUILD_TOOLS)" "[ -d '$(ANDROID_HOME)/build-tools/$(BUILD_TOOLS)' ]" "make setup-android-sdk"; \
	check "release.keystore"  "[ -f release.keystore ]"                             "make keystore"; \
	check "cc / pkg-config"   "command -v cc && command -v pkg-config"              "make setup-host"; \
	echo ""; \
	if [ $$ok = 0 ]; then echo "$(GREEN)Ambiente pronto.$(RESET)"; else echo "$(RED)Falta algo — rode make setup.$(RESET)"; fi; \
	exit $$ok

# ── Desktop: onde se desenvolve ──────────────────────────────────────────────

.PHONY: run
run: ## Roda no desktop (hot-reload de .gv/.gss)
	cargo run

.PHONY: check
check: ## Verifica o código do desktop sem linkar
	cargo check

.PHONY: check-android
check-android: ## Verifica o código para o alvo Android (sem empacotar)
	cargo check --lib --target $(TARGET)

.PHONY: fmt
fmt: ## Formata com rustfmt
	cargo fmt --all

.PHONY: clippy
clippy: ## Clippy com warnings como erro
	cargo clippy --all-targets -- -D warnings

# ── APK ───────────────────────────────────────────────────────────────────────

# `--lib` porque o pacote tem lib E bin: o APK carrega a `cdylib`, e o binário
# do desktop não faz sentido empacotado.

.PHONY: build
build: ## Compila o APK de debug (ABI=aarch64|armv7|x86_64|x86)
	@echo "$(BOLD)── APK debug ($(TARGET)) ──$(RESET)"
	cargo apk build --lib --target $(TARGET)
	@echo ""
	@echo "$(GREEN)APK:$(RESET) $(APK_DEBUG)"
	@ls -lh $(APK_DEBUG)

.PHONY: release
release: keystore ## Compila o APK de release (otimizado, assinado com release.keystore)
	@echo "$(BOLD)── APK release ($(TARGET)) ──$(RESET)"
	cargo apk build --lib --release --target $(TARGET)
	@echo ""
	@echo "$(GREEN)APK:$(RESET) $(APK_RELEASE)"
	@ls -lh $(APK_RELEASE)

# ── Aparelho / emulador ──────────────────────────────────────────────────────

.PHONY: devices
devices: ## Lista aparelhos e emuladores conectados
	adb devices -l

.PHONY: install
install: build ## Instala o APK de debug no aparelho (adb install -r)
	adb install -r $(APK_DEBUG)

.PHONY: launch
launch: install ## Compila, instala, abre o app e segue o logcat
	adb shell am start -n $(PACKAGE)/$(ACTIVITY)
	@$(MAKE) --no-print-directory logcat

.PHONY: logcat
logcat: ## Segue o log do app (Ctrl-C para sair)
	adb logcat -c
	adb logcat -s {{nome_crate}}:V RustStdoutStderr:V AndroidRuntime:E

.PHONY: stop
stop: ## Encerra o app no aparelho
	adb shell am force-stop $(PACKAGE)

.PHONY: uninstall
uninstall: ## Remove o app do aparelho
	adb uninstall $(PACKAGE)

# ── Emulador (opcional) ──────────────────────────────────────────────────────

AVD_NAME  ?= {{nome_crate}}_avd
AVD_IMAGE ?= system-images;android-$(API_LEVEL);google_apis;x86_64

.PHONY: setup-emulator
setup-emulator: ## Baixa a imagem x86_64 e cria um AVD (precisa de KVM)
	@test -e /dev/kvm || echo "$(RED)aviso:$(RESET) sem /dev/kvm o emulador x86_64 é inutilmente lento"
	sdkmanager "emulator" "$(AVD_IMAGE)"
	@echo no | avdmanager create avd -n $(AVD_NAME) -k "$(AVD_IMAGE)" --force
	@echo "$(GREEN)AVD '$(AVD_NAME)' criado.$(RESET) Rode: make emulator, depois: make launch ABI=x86_64"

.PHONY: emulator
emulator: ## Abre o emulador
	emulator -avd $(AVD_NAME) -gpu host

# ── Limpeza / info ───────────────────────────────────────────────────────────

.PHONY: clean
clean: ## Remove artefatos de build
	cargo clean

.PHONY: version
version: ## Exibe a versão atual
	@echo "$(VERSION)"

.PHONY: help
help: ## Lista todos os targets disponíveis
	@echo ""
	@echo "$(BOLD)$(APP) $(VERSION) — Android — targets disponíveis$(RESET)"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ { \
		printf "  $(CYAN)%-19s$(RESET) %s\n", $$1, $$2 \
	}' $(MAKEFILE_LIST)
	@echo ""
	@echo "  $(BOLD)No Windows:$(RESET) use  fazer.bat  (não há make lá)"
	@echo "  $(BOLD)O que ainda não funciona no Android:$(RESET) ANDROID_TODO.md"
	@echo ""
