#!/usr/bin/env bash
# Fase 0: o MESMO painel (examples/painel/web/index.html) numa janela do Chrome
# com perfil novo — uma aba só, nada de extensão. Soma todos os processos do Chrome
# (browser + GPU + renderer + utilitários): é o custo de "mostrar este painel".
#
#   scripts/medir_chrome.sh [rodadas]
set -euo pipefail
RODADAS="${1:-5}"
PORTA=8098
RAIZ="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$TMP"' EXIT
(cd "$RAIZ/examples/painel" && exec python3 -m http.server "$PORTA" >/dev/null 2>&1) &
sleep 1
mediana() { sort -n | awk '{a[NR]=$1} END {print a[int((NR+1)/2)]}'; }

: >"$TMP/rss"; : >"$TMP/pss"; : >"$TMP/n"
for i in $(seq "$RODADAS"); do
  PERFIL="$TMP/perfil$i"
  google-chrome --user-data-dir="$PERFIL" --no-first-run --no-default-browser-check \
    --disable-extensions --app="http://127.0.0.1:$PORTA/web/index.html" >/dev/null 2>&1 &
  sleep 8
  rss=0; pss=0; n=0
  for pid in $(pgrep -f -- "--user-data-dir=$PERFIL"); do
    r=$(awk '$1=="VmRSS:" {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
    p=$(awk '$1=="Pss:" {print $2}' "/proc/$pid/smaps_rollup" 2>/dev/null || echo 0)
    rss=$((rss + ${r:-0})); pss=$((pss + ${p:-0})); n=$((n + 1))
  done
  echo "$rss" >>"$TMP/rss"; echo "$pss" >>"$TMP/pss"; echo "$n" >>"$TMP/n"
  pkill -f -- "--user-data-dir=$PERFIL" || true
  sleep 1
done
echo "Chrome $(google-chrome --version | awk '{print $3}') | rodadas: $RODADAS (mediana)"
printf 'processos                  %8s\n' "$(mediana <"$TMP/n")"
printf 'soma dos RSS               %8.1f MiB (conta páginas compartilhadas mais de uma vez)\n' "$(echo "$(mediana <"$TMP/rss")/1024" | bc -l)"
printf 'soma dos PSS               %8.1f MiB (a medida honesta)\n' "$(echo "$(mediana <"$TMP/pss")/1024" | bc -l)"
