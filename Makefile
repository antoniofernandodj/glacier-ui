# glacier-ui — tarefas de topo.
#
# Instala as extensões de VS Code a partir da raiz do projeto. Cada extensão
# tem seu próprio Makefile em editors/; aqui só delegamos.

GV  := editors/vscode-gv
GSS := editors/vscode
CLI := crates/glacier-cli

.PHONY: help install-gv install-gss install-extensions reinstall-extensions uninstall-extensions \
        sync-extensions publish-cli clean-extensions \
        deb-cli check-deb install-cli reinstall-cli uninstall-cli clean-deb

help: ## Lista os alvos
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'

install-gv: ## Instala a extensão Glacier View (.gv) no VS Code
	$(MAKE) -C $(GV) install

install-gss: ## Instala a extensão Glacier GSS (.gss) no VS Code
	$(MAKE) -C $(GSS) install

install-extensions: install-gv install-gss ## Instala as duas extensões de VS Code

reinstall-extensions: ## Reempacota e reinstala as duas extensões
	$(MAKE) -C $(GV) reinstall
	$(MAKE) -C $(GSS) reinstall

uninstall-extensions: ## Remove as duas extensões do VS Code
	$(MAKE) -C $(GV) uninstall
	$(MAKE) -C $(GSS) uninstall

# ── CLI (crates/glacier-cli) ────────────────────────────────────────────────
# A CLI embute as duas extensões para instalá-las sem Node/vsce. Num checkout
# ela as lê de `editors/`; num `cargo publish`, não — o Cargo não empacota nada
# de fora do diretório do crate. Este alvo copia a árvore para dentro dele, de
# onde o `include` do Cargo.toml a leva para o .crate (o `.gitignore` mantém a
# cópia fora do git: ela é artefato de publicação, não fonte).

sync-extensions: ## Copia editors/ para dentro do crate da CLI (pré-publicação)
	rm -rf $(CLI)/extensions
	mkdir -p $(CLI)/extensions
	cp -r $(GV)  $(CLI)/extensions/$(notdir $(GV))
	cp -r $(GSS) $(CLI)/extensions/$(notdir $(GSS))
	find $(CLI)/extensions -name '*.vsix' -delete
	@sed -n 's/^version *= *"\(.*\)"/\1/p' Cargo.toml | head -1 > $(CLI)/engine-version.txt
	@echo "extensões copiadas para $(CLI)/extensions"
	@echo "versão do motor gravada em $(CLI)/engine-version.txt: $$(cat $(CLI)/engine-version.txt)"

# `--allow-dirty` é necessário e não é cego. O `sync-extensions` acima cria
# `extensions/`, que o `include` do Cargo.toml empacota mas o `.gitignore`
# mantém fora do git — e o cargo trata "no pacote, não commitado" como árvore
# suja. A trava é a linha anterior: se o `git status` não estiver limpo, o alvo
# para antes de publicar; então os únicos arquivos "sujos" que sobram são
# exatamente os que o sync acabou de gerar.
publish-cli: ## Publica a CLI no crates.io (roda o sync antes)
	@test -z "$$(git status --porcelain)" || \
		{ echo "árvore de git suja — commite antes de publicar"; exit 1; }
	$(MAKE) sync-extensions
	cargo publish -p glacier-cli --allow-dirty
	$(MAKE) clean-extensions

clean-extensions: ## Remove a cópia vendorizada (ela SOMBREIA editors/ na build local)
	rm -rf $(CLI)/extensions $(CLI)/engine-version.txt

# ── .deb da CLI ─────────────────────────────────────────────────────────────
# Para testar o `glacier` como o usuário final o vê: no PATH, longe do target/,
# sem passar pelo crates.io. O pacote é só o binário (~270 KB) e depende de
# libc6 e nada mais — as extensões de VS Code vão EMBUTIDAS nele, e o .vsix é
# montado em tempo de execução (ver crates/glacier-cli/build.rs e src/vsix.rs).
#
# O `dpkg -i` precisa de root, então `install-cli` chama sudo — rode-o de um
# terminal onde você possa digitar a senha.

CARGO_DEB := $(shell command -v cargo-deb 2> /dev/null)

deb-cli: ## Constrói o .deb da CLI em target/debian/ (e confere as dependências)
ifndef CARGO_DEB
	@echo "cargo-deb não encontrado. Instale com: cargo install cargo-deb"
	@exit 1
endif
	cargo deb -p glacier-cli
	@echo
	@$(MAKE) --no-print-directory check-deb

# O `Depends` do pacote sai do `dpkg-shlibdeps` (`depends = "$$auto"`), que
# declara o MÍNIMO: ele omite o que já vem por transitividade. É por isso que
# `libgcc-s1` não aparece lá mesmo sendo um DT_NEEDED do binário — `libc6`
# depende dele. Ou seja, a linha `Depends` não é a lista do que o binário usa,
# e conferir só ela deixaria passar uma biblioteca nova de verdade.
#
# Este alvo confere a fonte: o DT_NEEDED do ELF empacotado. Tudo que a
# lista abaixo permite é glibc (libc6) ou vem por ela; qualquer outra coisa é
# dependência nova, e aí o `Depends` precisa declará-la à mão.
DEB_LIBS_OK := libc.so.6 libm.so.6 libdl.so.2 libpthread.so.0 librt.so.1 libgcc_s.so.1

check-deb: ## Falha se o binário do .deb ganhou dependência nativa nova
	@deb="$$(ls -t target/debian/glacier-cli_*.deb 2>/dev/null | head -1)"; \
	if [ -z "$$deb" ]; then echo "nenhum .deb em target/debian — rode 'make deb-cli'"; exit 1; fi; \
	tmp="$$(mktemp -d)"; \
	dpkg-deb -x "$$deb" "$$tmp"; \
	libs="$$(readelf -d "$$tmp/usr/bin/glacier" | sed -n 's/.*Shared library: \[\(.*\)\]/\1/p')"; \
	rm -rf "$$tmp"; \
	novas=""; \
	for lib in $$libs; do \
		case "$$lib" in ld-linux*) continue ;; esac; \
		echo "$(DEB_LIBS_OK)" | tr " " "\n" | grep -qxF "$$lib" || novas="$$novas $$lib"; \
	done; \
	echo "  DT_NEEDED :$$(echo $$libs | sed 's/^/ /')"; \
	echo "  Depends   : $$(dpkg -I "$$deb" | sed -n 's/^ Depends: //p')"; \
	if [ -n "$$novas" ]; then \
		echo; \
		echo "  ERRO: dependência nativa nova, fora da glibc:$$novas"; \
		echo "  Declare-a em [package.metadata.deb] depends, em crates/glacier-cli/Cargo.toml,"; \
		echo "  e acrescente-a a DEB_LIBS_OK aqui se ela for mesmo esperada."; \
		exit 1; \
	fi; \
	echo "  ok: nada além da glibc (libgcc-s1 vem por libc6)"

# `ls -t | head -1` em vez do nome montado à mão: o arquivo carrega versão e
# arquitetura no nome, e um bump no Cargo.toml não deve quebrar este alvo.
install-cli: deb-cli ## Constrói e INSTALA o .deb (usa sudo)
	sudo dpkg -i "$$(ls -t target/debian/glacier-cli_*.deb | head -1)"
	@echo
	@command -v glacier >/dev/null && glacier --version

reinstall-cli: uninstall-cli install-cli ## Reinstala (remove + constrói + instala)

uninstall-cli: ## Remove o pacote glacier-cli do sistema (usa sudo)
	-sudo dpkg -r glacier-cli

clean-deb: ## Apaga os .deb construídos
	rm -rf target/debian
