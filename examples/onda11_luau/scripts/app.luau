-- Onda 11 em Luau — o que fica por cima, do lado do script.
--
-- O que este arquivo mostra e a versão Rust não mostra: quase nada precisou
-- de função Luau nenhuma.
--
-- `<stack>`, `x=`/`y=`/`anchor=`, `<mdiarea>`/`<mdisubwindow>` inteiros — são
-- ATRIBUTOS DE MARKUP e um arrasto que escreve direto nas chaves que o
-- `<mdisubwindow>` nomeia. Nenhum dos dois pediu uma linha aqui: o habilitador
-- A (`<stack>`) é resolvido no parser/render, e o habilitador B
-- (`grip::Alvo::Ponto`) escreve as chaves do motor pra baixo, do mesmo jeito
-- que o `<slider>` já escrevia desde a 0.63. É o MESMO terceiro regime de
-- custo que a Onda 9 já tinha medido em Luau: um habilitador de motor que
-- rende meia dúzia de widgets e ainda assim custa zero na camada de cima.
--
-- O `on_change="qr_texto"` do campo de texto é o mesmo caso: sem uma função
-- chamada `qr_texto`, o `dispatch` do motor cai no binding legado
-- `ctx.qr_texto = value` sozinho — nenhuma linha aqui resolve isso, porque
-- não tem o que resolver.
--
-- O que sobra para o script fazer, então, são exatamente quatro coisas, e
-- nenhuma delas é nova: semear (`init`), alternar um booleano, reagir a um
-- clique com payload (`remover_tag:<tag>`) e reagir a um clique sem payload.

function init()
    ctx.status = "Pronto"

    ctx.abas = {
        { id = "empilhamento", label = "Stack + NotificationDot + Splash" },
        { id = "mdi", label = "MdiArea" },
        { id = "troco", label = "QrCode + Chip + Skeleton + CommandLink" },
    }
    ctx.aba = "empilhamento"

    -- ── NotificationDot ──────────────────────────────────────────────
    ctx.tem_notificacao = "true"

    -- ── SplashScreen ─────────────────────────────────────────────────
    ctx.carregando = "true"

    -- ── MdiArea ──────────────────────────────────────────────────────
    --
    -- Cada janela nomeia QUATRO chaves. Sem seed nenhum elas cairiam no
    -- cascade (20 + índice×24) — seedamos duas para mostrar as duas formas,
    -- e deixamos a terceira sem seed de propósito.
    ctx.editor_x = "40"
    ctx.editor_y = "40"
    ctx.editor_w = "360"
    ctx.editor_h = "220"
    ctx.console_x = "260"
    ctx.console_y = "160"
    -- console_w/console_h sem seed: nasce no default_w/default_h do markup.

    -- ── Chip ─────────────────────────────────────────────────────────
    ctx.tags = { "produção", "staging", "us-east-1" }

    -- ── QrCode ───────────────────────────────────────────────────────
    ctx.qr_texto = "https://glacier-ui.dev"
end

-- ── O "×" de um Chip ─────────────────────────────────────────────────────
--
-- `on_remove="remover_tag:{tag}"` chama `remover_tag(tag)` — a convenção
-- `nome:sufixo` do dispatcher (ver `src/luau/mod.rs::run_inner`), a mesma
-- que um componente Rust faria fatiando a string à mão em `action.split_once`.
--
-- `ctx.tags` chega aqui como a STRING JSON (o `ctx` sempre espelha o contexto
-- do motor como string — quem grava uma tabela é o Luau, na volta), então
-- `json.decode` é o caminho de volta para uma tabela filtrável.
function remover_tag(alvo)
    local tags = json.decode(ctx.tags or "[]")
    local restantes = {}
    for _, t in ipairs(tags) do
        if t ~= alvo then
            table.insert(restantes, t)
        end
    end
    ctx.tags = restantes
    ctx.status = "Removida: " .. alvo
end

-- ── SplashScreen ─────────────────────────────────────────────────────────

function fechar_splash()
    ctx.carregando = nil
    ctx.status = "Splash fechada — o conteúdo estava por baixo o tempo todo"
end

function abrir_splash()
    ctx.carregando = "true"
end

-- ── NotificationDot ──────────────────────────────────────────────────────

function alternar_notificacao()
    local ligado = ctx.tem_notificacao ~= nil and ctx.tem_notificacao ~= "" and ctx.tem_notificacao ~= "false"
    ctx.tem_notificacao = ligado and nil or "true"
end

-- ── CommandLink / RoundButton ────────────────────────────────────────────

function instalar_tipica()
    ctx.status = "CommandLink: instalação típica escolhida"
end

function instalar_custom()
    ctx.status = "CommandLink: instalação customizada escolhida"
end

function novo_item()
    ctx.status = "RoundButton: novo item"
end
