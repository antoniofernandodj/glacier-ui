#!/bin/sh
# Instalado como /usr/bin/{{nome_projeto}} pelo .deb.
#
# O programa de verdade fica em /usr/share/{{nome_projeto}}/, junto do views/,
# porque o app resolve os templates contra o DIRETÓRIO DE TRABALHO — é o que dá
# o hot-reload em dev. Sem este `cd`, rodar `{{nome_projeto}}` de qualquer pasta
# abriria uma janela vazia, sem mensagem de erro nenhuma.
cd /usr/share/{{nome_projeto}} || exit 1
exec ./{{nome_projeto}} "$@"
