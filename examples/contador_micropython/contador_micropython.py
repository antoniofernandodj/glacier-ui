# Comportamento do contador em MicroPython — a segunda linguagem de
# `<script>`, ao lado do Luau (ver `examples/contador_externo` pro mesmo
# exemplo em Lua). O template referencia este arquivo via
# `<script lang="micropython" src="contador_micropython.py">`.
#
# O contexto do motor é o **dotdict** global `ctx` (ver
# src/micropython/prelude.py): `ctx.contador` lê/escreve exatamente como o
# `ctx.contador` do Lua — ausente vira `None`, sem erro, o que deixa
# `ctx.contador or 0` funcionar sem precisar de `.get(...)`. `ctx["contador"]`
# também funciona (semântica de dict de verdade: levanta `KeyError` se
# ausente) — os dois lêem/escrevem o mesmo dado por baixo, use o que ler
# melhor em cada linha. Valores sempre chegam como STRING; quem quiser
# aritmética faz `int(...)` no próprio script, como abaixo.


def init():
    ctx.contador = int(ctx.contador or 0)  # type: ignore
    ctx.passo = int(ctx.passo or 1)  # type: ignore


def incrementar():
    ctx.contador = int(ctx.contador) + int(ctx.passo)  # type: ignore


def decrementar():
    ctx.contador = int(ctx.contador) - int(ctx.passo)  # type: ignore


def zerar():
    ctx.contador = 0  # type: ignore


# onChange do input: define de quanto em quanto o contador anda.
def definir_passo(v: str):
    try:
        ctx.passo = int(v)  # type: ignore
    except ValueError:
        ctx.passo = 1  # type: ignore
