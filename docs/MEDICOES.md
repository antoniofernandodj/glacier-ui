# Medições (Fase 0 do `PLANO_SUBSTITUIBILIDADE.md`)

O painel de referência tem uma lista, um formulário, um `fetch` e um gráfico de
barras: `examples/gvb/painel` (Glacier) e `examples/gvb/painel/web/index.html` (o
equivalente em HTML, sem framework). Reproduzir:

```
make medir                     # Glacier: build release + 5 rodadas
scripts/medir_chrome.sh 5      # o mesmo painel numa janela do Chrome, perfil novo
```

Para só **abrir** o painel (sem medir): `WGPU_BACKEND=gl cargo run --example painel`. O `main.rs`
sobe sozinho um servidor local para o `dados.json` (uma thread que morre com o processo; se a
porta 8098 já estiver ocupada — o `make medir` sobe o dele —, usa o que está lá). A versão web
(`web/index.html`) ainda pede um servidor de arquivos: `python3 -m http.server 8098` na pasta do painel.

## Resultado (2026-10-30, mediana de 5 rodadas)

Máquina: Linux 7.0, Intel i5-3470 (Ivy Bridge, iGPU), `WGPU_BACKEND=gl`
(o Vulkan desta iGPU é quebrado). Glacier 0.143.0, Chrome 155.

| | Glacier 0.143.0 (30/10) | Glacier 0.146.1, com i18n (31/10) | Chrome (janela `--app`, perfil novo) |
|---|---|---|---|
| Processos | 1 | 1 | 11 |
| Memória residente (RSS) em repouso | 89,7 MiB | 90,6 MiB | 1252 MiB (soma, conta compartilhado em dobro) |
| **PSS** em repouso | **38,5 MiB** | **35,2 MiB** | **385,9 MiB** |
| RSS sob carga (um quadro por vsync, 10 s) | 89,9 MiB | 90,5 MiB | não medido |
| Binário release | 40,7 MiB | 41,5 MiB | n/a |
| Binário release, `strip` | 32,0 MiB | 32,7 MiB | n/a |
| Primeira tela montada | 130 ms | 110 ms | não medido |

A coluna do meio é o painel **com i18n** (botões Português/English, textos por `{t('…')}`),
medido com `make medir` em 2026-10-31. A do Chrome **não foi refeita** (é de 2026-10-30, e o
HTML mudou só pelos botões e traduções). O que mudou no Glacier:

- **Binário: +0,8 MiB** (com e sem `strip`) — é o motor da 0.146 (i18n, ponte de acessibilidade
  e o foco de teclado do botão), não só o painel. As features `audio` e `a11y` não entram neste build.
- **RSS: +0,9 MiB** em repouso e +0,6 MiB sob carga — dentro do que uma rodada varia.
- **PSS e primeira tela vieram MENORES** (35,2 contra 38,5 MiB; 110 contra 130 ms). **Não leia
  isso como ganho**: ninguém otimizou nada nesta rodada, e são medianas de 5 execuções numa máquina
  só, com a mesma variação entre rodadas que já havia. O que se pode dizer é que o i18n **não
  deixou o painel mais pesado de forma mensurável**; não que o deixou mais leve.

**Como ler.** O RSS conta cada biblioteca compartilhada (Mesa/GL, libc…) inteira,
em cada processo que a mapeia; por isso a soma de 11 processos do Chrome é inflada.
**PSS** divide as páginas compartilhadas entre quem as usa — é a comparação justa, e
ela dá ~10× a favor do Glacier neste painel. O RSS do Glacier (90 MiB) é bem maior
que o PSS dele (38 MiB) pelo mesmo motivo: a maior parte é biblioteca do sistema.

## O painel ganhou i18n depois da primeira medição (2026-10-31)

O painel ganhou **i18n** (Fase 7): os textos vêm de `locales/*.json` por `{t('…')}` e há dois
botões, Português e English, que trocam o idioma ao vivo. O `web/index.html` ganhou os mesmos
botões e as mesmas traduções, para a tela continuar sendo a mesma dos dois lados. A coluna do
Glacier acima foi **refeita** com essa versão; a do Chrome (`scripts/medir_chrome.sh`) **não**.

## Correção (2026-10-30): a primeira medição era de um painel quebrado

A primeira versão deste documento (PSS 35,4 MiB, 120 ms) foi medida sobre um painel que
**não carregava os dados**: o `fetch` do `init` nunca rodava (defeito do motor, corrigido
em `take_init_tasks`) e a lista de serviços saía vazia (`foreach(items = @servicos)` no
lugar de `items = servicos`, erro do próprio painel). Ou seja, medi uma tela sem gráfico e
sem lista — e ninguém tinha olhado a janela. Os números acima são da nova medição, com o
painel completo (lista, formulário, gráfico vindo do `fetch`, "ok (200)"). A diferença foi
pequena (+3 MiB de PSS, +10 ms), mas a primeira medição não valia como medição do painel
descrito.

## O que estas medições NÃO dizem

- **Não é um benchmark de aplicação real.** É um painel pequeno. Uma página com
  centenas de nós ou um framework de verdade muda os dois lados.
- **Tauri não foi medido.** Exige `webkit2gtk-4.1` (dev), que não está instalado
  nesta máquina; instalar pede `sudo`. O plano pedia essa terceira coluna e ela
  continua em aberto.
- **"Primeira tela montada" não é "primeiro pixel".** Conta do início do processo até
  o motor montar a árvore do primeiro `view` (`GLACIER_PRIMEIRA_TELA=1`); não inclui
  o layout e o primeiro desenho do `iced`/`wgpu`. Para o Chrome não há número
  comparável, então a célula está vazia em vez de inventada.
- **Não conferi que a janela do Chrome carregou o painel** (o status de `fetch` na
  tela): o script mede os processos e não olha a página. O servidor local estava de
  pé e o HTML é o mesmo, mas isso é premissa, não verificação.
- **Sob carga o Chrome não foi medido**, e o Glacier ficou igual ao repouso (o painel
  não cresce): não prova nada sobre telas maiores.
- Uma máquina, um backend GL, uma GPU antiga. Os números absolutos mudam em outra.
