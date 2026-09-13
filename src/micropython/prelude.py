# Prelúdio do `<script lang="micropython">` — roda ANTES do corpo do
# usuário em cada componente (ver `MicropythonComponent::build`, que faz o
# equivalente ao `luau.load(PRELUDE)` do lado Lua). Define o global `ctx`
# como um **dotdict**: `ctx.chave` (além de `ctx["chave"]`, que já
# funcionaria de graça se `ctx` fosse só um dict) sobre o dict de dados de
# verdade — que mora num global à parte, `__glacier_ctx__`, o único nome que
# `glacier_mp_ctx_reset`/`_set`/`_visit` (vendor/glacier_mp_shim/glacier_mp_shim.c)
# leem e escrevem por fora (ver o comentário "bridge de ctx" lá).
#
# `ctx.chave` ausente devolve `None` em vez de levantar `AttributeError` —
# mesma leniência do `ctx.chave` em Lua (`nil` sem erro), pensada pra
# `ctx.contador or 0` funcionar sem precisar de `.get(...)`. `ctx["chave"]`
# continua com semântica de dict de verdade (levanta `KeyError` se ausente —
# use `ctx.get("chave", padrao)` quando quiser um default explícito).
# `ctx.chave = None` (e `ctx["chave"] = None`/`del ctx["chave"]`) remove a
# chave — a forma de "apagar", espelhando `ctx.chave = nil` no Lua.
__glacier_ctx__ = {}


class _GlacierCtx:
    def __getattr__(self, chave):
        return __glacier_ctx__.get(chave)

    def __setattr__(self, chave, valor):
        if valor is None:
            __glacier_ctx__.pop(chave, None)
        else:
            __glacier_ctx__[chave] = valor

    def __getitem__(self, chave):
        return __glacier_ctx__[chave]

    def __setitem__(self, chave, valor):
        if valor is None:
            del __glacier_ctx__[chave]
        else:
            __glacier_ctx__[chave] = valor

    def __delitem__(self, chave):
        del __glacier_ctx__[chave]

    def __contains__(self, chave):
        return chave in __glacier_ctx__

    def get(self, chave, padrao=None):
        return __glacier_ctx__.get(chave, padrao)


ctx = _GlacierCtx()
