#!/usr/bin/env bash
# Fase 0 do docs/PLANO_SUBSTITUIBILIDADE.md: mede o painel de referência.
#
#   make medir                 # ou: scripts/medir.sh [rodadas]
#
# Mede, em release: tamanho do binário (com e sem strip), memória residente em
# repouso e sob carga, e o tempo até a primeira tela montada. Imprime a MEDIANA das
# rodadas. Linux apenas (lê /proc). Exige: cargo, python3, strip.
set -euo pipefail

RODADAS="${1:-5}"
PORTA=8098
RAIZ="$(cd "$(dirname "$0")/.." && pwd)"
cd "$RAIZ"
export WGPU_BACKEND="${WGPU_BACKEND:-gl}"   # o Vulkan da iGPU desta máquina é quebrado

cargo build --release --example painel 2>&1 | tail -1
BIN="target/release/examples/painel"
TMP="$(mktemp -d)"
trap 'kill $(jobs -p) 2>/dev/null || true; rm -rf "$TMP"' EXIT

cp "$BIN" "$TMP/painel-strip" && strip "$TMP/painel-strip"
TAM_BRUTO=$(stat -c %s "$BIN")
TAM_STRIP=$(stat -c %s "$TMP/painel-strip")

(cd examples/painel && exec python3 -m http.server "$PORTA" >/dev/null 2>&1) &
sleep 1

# kB de um campo de /proc/PID/<arquivo>
campo() { awk -v k="$2:" '$1==k {print $2}' "/proc/$1/$3"; }

mediana() { sort -n | awk '{a[NR]=$1} END {print a[int((NR+1)/2)]}'; }

: >"$TMP/rss_repouso"; : >"$TMP/rss_carga"; : >"$TMP/pss_repouso"; : >"$TMP/tempo"
for i in $(seq "$RODADAS"); do
  GLACIER_PRIMEIRA_TELA=1 "$BIN" 2>"$TMP/err" &
  PID=$!
  sleep 6                                           # repouso: nada se mexe
  campo "$PID" VmRSS status >>"$TMP/rss_repouso"
  campo "$PID" Pss smaps_rollup >>"$TMP/pss_repouso"
  grep -o '[0-9]*ms' "$TMP/err" | head -1 | tr -d 'ms' >>"$TMP/tempo" || true
  kill "$PID"; wait "$PID" 2>/dev/null || true

  # carga: um quadro por vsync durante 10s (GLACIER_PERF_STRESS)
  GLACIER_PERF_STRESS=1 "$BIN" 2>/dev/null &
  PID=$!
  sleep 10
  campo "$PID" VmRSS status >>"$TMP/rss_carga"
  kill "$PID"; wait "$PID" 2>/dev/null || true
done

echo "máquina: $(uname -srm) | $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs)"
echo "backend: WGPU_BACKEND=$WGPU_BACKEND | rodadas: $RODADAS (mediana)"
printf 'binário release            %8.1f MiB\n' "$(echo "$TAM_BRUTO/1048576" | bc -l)"
printf 'binário release (strip)    %8.1f MiB\n' "$(echo "$TAM_STRIP/1048576" | bc -l)"
printf 'memória residente, repouso %8.1f MiB\n' "$(echo "$(mediana <"$TMP/rss_repouso")/1024" | bc -l)"
printf 'PSS, repouso               %8.1f MiB\n' "$(echo "$(mediana <"$TMP/pss_repouso")/1024" | bc -l)"
printf 'memória residente, carga   %8.1f MiB\n' "$(echo "$(mediana <"$TMP/rss_carga")/1024" | bc -l)"
printf 'primeira tela montada      %8s ms\n' "$(mediana <"$TMP/tempo")"
