-- Onda 9 em Luau — o ponteiro preso, do lado do script.
--
-- O que este arquivo mostra e a versão Rust não mostra: os nove widgets desta
-- onda **não têm API de script**, e isso é a observação, não uma falta.
--
-- A Onda 8 precisou de quatro globais novos (`prompt`, `progress`,
-- `pick_color`, e o retorno generalizado) porque um diálogo SUSPENDE: o script
-- para e volta com a resposta, e sem um global não haveria como parar. Um
-- arrasto não suspende nada. Ele escreve numa chave, e o Luau já lê e escreve
-- chaves desde sempre — `ctx.painel_h = "260 fill"` é a mesma linha que
-- configuraria um `<slider>` da 0.63.
--
-- Então o que este exemplo demonstra é o contrário do da Onda 8: um habilitador
-- de motor que **não custou nada** à camada de script. As nove tags chegam ao
-- Luau prontas, e o que aparece abaixo são as três coisas que o script de fato
-- faz — semear, receber `on_change` e reagir a um atalho.

function init()
    ctx.status = "Pronto — experimente Ctrl+S"
    ctx.salvamentos = "0"

    -- ── 1. Splitter ────────────────────────────────────────────────────
    --
    -- A chave guarda TRILHAS, no formato do `columns` do `<grid>`: um número é
    -- pixel, `fill` fica com o resto. É a mesma chave que a alça de coluna de
    -- um `<tableheader>` escreve — na Onda 9 os dois viraram o mesmo código.
    ctx.painel_h = "300 fill"
    ctx.painel_v = "150 fill"

    -- ── 2. RangeSlider ─────────────────────────────────────────────────
    --
    -- DUAS chaves, a mesma forma do `<daterangepicker range>` da Onda 3.
    ctx.preco_min = "150"
    ctx.preco_max = "820"

    -- ── 3. Tumbler ─────────────────────────────────────────────────────
    --
    -- A coleção pela convenção `items` de sempre. Do lado Luau uma tabela vira
    -- JSON sozinha, que é o que a torna cômoda aqui e verbosa em Rust.
    ctx.meses = {
        "janeiro", "fevereiro", "março", "abril", "maio", "junho",
        "julho", "agosto", "setembro", "outubro", "novembro", "dezembro",
    }
    ctx.mes = "março"
    ctx.regiao = "sudeste"

    -- ── 4. SwipeView + PageIndicator ───────────────────────────────────
    --
    -- Índice base ZERO — o `currentIndex` do QML. As duas tags leem a mesma
    -- chave, que é o que faz o indicador acompanhar o arrasto sem uma linha de
    -- cola.
    ctx.pagina = "0"

    -- ── 6. RubberBand ──────────────────────────────────────────────────
    ctx.caixas = caixas()
    ctx.marcados = ""

    -- ── 9. ShortcutInput ───────────────────────────────────────────────
    ctx.atalho_salvar = "ctrl+s"
    ctx.atalho_buscar = "ctrl+shift+p"
end

-- Os alvos do laço: `id` mais geometria, em coordenadas do próprio widget.
--
-- A geometria mora no DADO e não no widget, e é o que torna o laço genérico:
-- ele não sabe o que são essas caixas, só quais o retângulo tocou. Aqui a
-- aritmética de posicionar a grade é do script — um markup não faz conta, que é
-- a lição que a Onda 8 aprendeu com o `<wizard>`.
function caixas()
    local nomes = { "api", "banco", "fila", "cdn", "cache", "proxy" }
    local out = {}
    for i, nome in ipairs(nomes) do
        local col = (i - 1) % 3
        local lin = math.floor((i - 1) / 3)
        table.insert(out, {
            id = nome,
            x = 20 + col * 110,
            y = 20 + lin * 76,
            w = 84,
            h = 60,
        })
    end
    return out
end

-- ── Item 8: os dois `<shortcut>` da tela ────────────────────────────────
--
-- São funções comuns. Um atalho não é um tipo novo de evento: é o teclado
-- despachando o que um botão despacharia, e é por isso que o habilitador B não
-- precisou de global nenhum do lado Luau.

function salvar()
    local n = tonumber(ctx.salvamentos) or 0
    n = n + 1
    ctx.salvamentos = tostring(n)
    ctx.status = string.format("Salvo (%dx) — pelo atalho ou pelo botão", n)
end

function buscar()
    ctx.status = "Busca aberta pelo atalho"
end

-- ── Item 5: o `<delaybutton>` ───────────────────────────────────────────
--
-- A ação só chega aqui quando o anel FECHA. Soltar antes não despacha nada —
-- não há o que tratar, e é o ponto todo do widget.
function apagar_tudo()
    ctx.marcados = ""
    ctx.status = "Tudo apagado — o anel fechou"
end

-- ── Item 6: o `<rubberband>` ────────────────────────────────────────────
--
-- O que ele SELECIONOU já está em `ctx.marcados`, escrito pelo próprio widget.
-- O que chega aqui é a geometria do retângulo, para quem quiser fazer outra
-- coisa com ela.
function lacou(retangulo)
    local quantos = 0
    for _ in string.gmatch(ctx.marcados or "", "[^,]+") do
        quantos = quantos + 1
    end
    ctx.status = string.format("Laço %s → %d serviço(s)", retangulo or "", quantos)
end

-- ── Item 4: o `on_change` do `<swipeview>` ──────────────────────────────
--
-- O único dos nove que o markup mandou delegar. Os outros oito gravam a chave
-- sozinhos — o par `on_change` vazio / preenchido de todo widget deste motor.
function virou(indice)
    ctx.pagina = indice
    ctx.status = "Página " .. indice
end
