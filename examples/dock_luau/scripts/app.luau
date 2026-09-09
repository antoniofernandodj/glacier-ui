-- Onda 12 em Luau — o <dock> do lado do script.
--
-- O <dock> inteiro é markup e motor: o <splitter> acoplado, o <stack>
-- flutuante, os botões `❒`/`✕` do cabeçalho, a aba de restaurar e o arrasto de
-- reancoragem (grip::Alvo::Zona, que comete a borda na soltura). Nenhuma linha
-- aqui desenha ou posiciona.
--
-- O que o script faz: semear, um atalho que escreve `lado`, e a PERSISTÊNCIA.
-- `on_change="salvar_layout"` no <dock> dispara `salvar_layout()` depois de
-- QUALQUER mudança de estado — botão do cabeçalho, arrasto, ou o atalho. O
-- handler grava as quatro chaves com o global `storage`; `init` as lê de volta.
-- Assim o dock lembra onde estava entre execuções.

local CHAVE = "layout"

function init()
    -- Defaults.
    ctx.lado = "left"
    ctx.tam_painel = "220 fill"
    ctx.painel_x = "90"
    ctx.painel_y = "70"

    -- Lembrar: se há um layout salvo, ele vence os defaults.
    local salvo = storage.get(CHAVE)
    if type(salvo) == "table" then
        for _, k in ipairs({ "lado", "tam_painel", "painel_x", "painel_y" }) do
            if salvo[k] ~= nil then
                ctx[k] = tostring(salvo[k])
            end
        end
    end

    ctx.camadas = {
        { id = "fundo", label = "Fundo" },
        { id = "linha", label = "Arte de linha" },
        { id = "sombra", label = "Sombras" },
    }
    ctx.camada = "linha"
    ctx.notas = "Arraste a faixa \"Camadas\" para uma borda e solte.\n`✕` esconde; a aba `▸` traz de volta.\nO layout é lembrado entre execuções (storage)."
    ctx.status = "painel: " .. ctx.lado .. " (lembrado, se havia layout salvo)"
end

-- `on_change` do <dock>: grava o layout inteiro.
function salvar_layout()
    storage.set(CHAVE, {
        lado = ctx.lado,
        tam_painel = ctx.tam_painel,
        painel_x = ctx.painel_x,
        painel_y = ctx.painel_y,
    })
    ctx.status = "layout salvo — painel: " .. ctx.lado
end

-- Atalho da barra: escreve `lado` e persiste (os botões do cabeçalho já
-- persistem sozinhos via on_change).
function mover_dock(destino)
    ctx.lado = destino
    salvar_layout()
end
