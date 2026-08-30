# glacier-ui — tarefas de topo.
#
# Instala as extensões de VS Code a partir da raiz do projeto. Cada extensão
# tem seu próprio Makefile em editors/; aqui só delegamos.

GV  := editors/vscode-gv
GSS := editors/vscode
CLI := crates/glacier-cli

.PHONY: help install-gv install-gss install-extensions reinstall-extensions uninstall-extensions \
        sync-extensions publish-cli clean-extensions

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
	@echo "extensões copiadas para $(CLI)/extensions"

publish-cli: sync-extensions ## Publica a CLI no crates.io (roda o sync antes)
	cargo publish -p glacier-cli
	$(MAKE) clean-extensions

clean-extensions: ## Remove a cópia vendorizada (ela SOMBREIA editors/ na build local)
	rm -rf $(CLI)/extensions
