-- Onda 12 em Luau — o <dock> do lado do script.
--
-- O <dock> inteiro é markup e motor: o <splitter> acoplado, o <stack>
-- flutuante, os botões `❒`/`—` do cabeçalho e o arrasto de reancoragem
-- (grip::Alvo::Zona, que comete a borda na soltura). Nenhuma linha aqui
-- desenha ou posiciona.
--
-- O que sobra: semear as chaves e um atalho que escreve `lado`. É o mesmo
-- binding de sempre — `ctx.lado = "right"` e o render_dock seguinte monta o
-- pai certo.

function init()
    -- Onde o painel está.
    ctx.lado = "left"
    -- Trilha do <splitter> quando acoplado: `<medida> fill`, no eixo do dock.
    ctx.tam_painel = "220 fill"
    -- Posição do painel flutuante.
    ctx.painel_x = "90"
    ctx.painel_y = "70"

    ctx.camadas = {
        { id = "fundo", label = "Fundo" },
        { id = "linha", label = "Arte de linha" },
        { id = "sombra", label = "Sombras" },
    }
    ctx.camada = "linha"
    ctx.notas = "Arraste a faixa \"Camadas\" para uma borda e solte.\nUm movimento pequeno conta como clique."
    ctx.status = "painel à esquerda"
end

-- Atalho da barra: `on_click="mover_dock:left"` chama `mover_dock("left")` — a
-- convenção `nome:sufixo` do dispatcher. Escreve a MESMA chave que os botões
-- do cabeçalho do <dock> e o arrasto de reancoragem escrevem.
function mover_dock(destino)
    ctx.lado = destino
    ctx.status = "painel: " .. destino
end
