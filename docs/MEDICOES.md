# Medições (Fase 0 do `PLANO_SUBSTITUIBILIDADE.md`)

O painel de referência tem uma lista, um formulário, um `fetch` e um gráfico de
barras: `examples/gvb/painel` (Glacier) e `examples/gvb/painel/web/index.html` (o
equivalente em HTML, sem framework). Reproduzir:

```
make medir                     # Glacier: build release + 5 rodadas
scripts/medir_chrome.sh 5      # o mesmo painel numa janela do Chrome, perfil novo
```

## Resultado (2026-10-30, mediana de 5 rodadas)

Máquina: Linux 7.0, Intel i5-3470 (Ivy Bridge, iGPU), `WGPU_BACKEND=gl`
(o Vulkan desta iGPU é quebrado). Glacier 0.142.0, Chrome 155.

| | Glacier | Chrome (janela `--app`, perfil novo) |
|---|---|---|
| Processos | 1 | 11 |
| Memória residente (RSS) em repouso | 89,3 MiB | 1252 MiB (soma, conta compartilhado em dobro) |
| **PSS** em repouso | **35,4 MiB** | **385,9 MiB** |
| RSS sob carga (um quadro por vsync, 10 s) | 89,2 MiB | não medido |
| Binário release | 40,5 MiB | n/a |
| Binário release, `strip` | 31,9 MiB | n/a |
| Primeira tela montada | 120 ms | não medido |

**Como ler.** O RSS conta cada biblioteca compartilhada (Mesa/GL, libc…) inteira,
em cada processo que a mapeia; por isso a soma de 11 processos do Chrome é inflada.
**PSS** divide as páginas compartilhadas entre quem as usa — é a comparação justa, e
ela dá ~11× a favor do Glacier neste painel. O RSS do Glacier (89 MiB) é bem maior
que o PSS dele (35 MiB) pelo mesmo motivo: a maior parte é biblioteca do sistema.

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
