--!strict
--
-- A Onda 5 inteira em Luau. O `main.rs` ao lado não tem `impl Component`: ele
-- registra o `.gv` e sai de cena.
--
-- O que o `examples/onda5` faz em Rust — `init` e três ações — mora aqui, e a
-- comparação lado a lado é o ponto do exemplo. Duas diferenças aparecem logo:
--
--  1. **Não há `self`.** O `Onda5` de Rust é um struct; aqui o estado que
--     sobrevive entre ações mora no `ctx` (o que a tela precisa ver) ou num
--     `local` do módulo (o que ela não precisa).
--  2. **`ctx` guarda texto.** Atribuir uma tabela a uma chave a codifica em
--     JSON — que é exatamente o formato que o `items` de um `<autocomplete>`
--     (e de qualquer `for-each`) lê.
--
-- E uma terceira, que é a que este exemplo em particular torna visível:
--
--  3. **`date.today()` é uma linha.** A versão Rust precisa de vinte linhas de
--     `civil_from_days` para saber que dia é hoje, porque o motor não tem uma
--     crate de data — de propósito (ver a §4 do PLANO_WIDGETS.md). O prelúdio
--     Luau tem. É o mesmo achado que o exemplo `onda3` registrou: o realce de
--     hoje é PROP, não relógio, e é o script que fecha esse buraco.
--
-- O que NÃO muda entre as duas versões é o mais importante: nenhum dos seis
-- widgets desta onda pede uma linha de script. Nenhuma função aqui abre ou
-- fecha um painel, troca uma página de aba ou filtra uma lista de sugestões —
-- pressionar o gatilho abre, clicar fora fecha, Esc fecha, ▲▼ navegam. O que
-- está aqui é só o que é do APP.

-- As cidades que o `<autocomplete>` filtra. Uma lista literal, para o exemplo
-- não depender de rede — e propositalmente com acento, que é o que o recorte
-- sem acento do widget existe para resolver: "sao paulo" acha "São Paulo".
local CIDADES: { string } = {
    "São Paulo",
    "Rio de Janeiro",
    "Belo Horizonte",
    "Brasília",
    "Salvador",
    "Fortaleza",
    "Curitiba",
    "Recife",
    "Porto Alegre",
    "Manaus",
    "Belém",
    "Goiânia",
    "São Luís",
    "Maceió",
    "Campo Grande",
    "Florianópolis",
    "Vitória",
    "Natal",
    "João Pessoa",
    "Teresina",
}

-- Os serviços, com `id` separado do rótulo — é o par que o `onSelect` do
-- `<autocomplete>` entrega: o widget grava o RÓTULO no campo (é o que a pessoa
-- vê) e manda o `id` para quem pediu (é o que liga a escolha a um registro).
local SERVICOS: { { id: string, label: string } } = {
    { id = "api", label = "API pública" },
    { id = "db", label = "Banco primário" },
    { id = "cache", label = "Redis" },
    { id = "fila", label = "Fila de eventos" },
    { id = "cdn", label = "CDN" },
    { id = "auth", label = "Autenticação" },
    { id = "mail", label = "Disparo de e-mail" },
    { id = "logs", label = "Coletor de logs" },
}

function init()
    -- As abas do `<tabs>`: a MESMA chave que antes alimentava a `<tabbar>`. O
    -- que mudou é que a lista não aparece mais uma segunda vez numa escada de
    -- `se`/`senão` — as páginas são `slot="…"` no markup.
    ctx.abas = {
        { id = "paineis", label = "Popover + Popup" },
        { id = "busca", label = "Autocomplete + calendarPopup" },
        { id = "gaveta", label = "Drawer" },
    }
    ctx.aba = "paineis"

    -- Uma chave por painel. Nenhuma delas é escrita por este arquivo depois
    -- daqui: quem as abre e fecha é o widget.
    ctx.menu_usuario = ""
    ctx.filtros = ""
    ctx.lateral = ""
    ctx.atalhos = ""
    ctx.gaveta = ""

    -- O conteúdo do painel de filtros. Semeá-las é o que faz a tela abrir
    -- coerente com o que o app considera o estado inicial — o widget só LÊ
    -- estas chaves; quem as escreve são as três funções lá embaixo.
    ctx.so_no_ar = "true"
    ctx.arquivados = "false"
    ctx.uso_max = "80"

    -- O `<autocomplete>`: a lista inteira numa chave, o texto noutra. Quem
    -- recorta é o widget — é a razão de ser dele.
    ctx.cidades = CIDADES
    ctx.cidade = ""
    ctx.servicos = SERVICOS
    ctx.servico_busca = ""
    ctx.servico_id = "—"

    -- As duas datas do `calendarPopup`. `hoje` é PROP, não relógio: o motor não
    -- lê a hora do sistema em lugar nenhum — e é aqui que a versão Luau fica
    -- mais curta que a Rust, porque o prelúdio já sabe a data.
    ctx.hoje = date.today()
    ctx.entrada = date.today()
    ctx.saida = ""

    ctx.escolha = "—"
    ctx.status = "Pronto"
end

-- Os botões da gaveta trocam de aba. `ir:paineis` chega aqui como
-- `ir("paineis")` — a convenção `nome:sufixo` do motor, que é o mesmo fatiar de
-- string que o `update` da versão Rust faz à mão.
--
-- Repare que a gaveta continua aberta: fechá-la ao navegar seria uma decisão de
-- app, e este exemplo prefere deixar visível que ela empurra em vez de cobrir.
function ir(destino: string)
    ctx.aba = destino
    ctx.status = "Aba: " .. destino
end

-- Os itens do popover de usuário. Zerar `menu_usuario` é o que fecha o painel
-- ao escolher — uma linha, e é o app decidindo: o widget sozinho só fecha por
-- clique fora e por Esc, porque um clique DENTRO do painel é do botão.
function escolher(item: string)
    ctx.escolha = item
    ctx.menu_usuario = ""
    ctx.status = "Menu: " .. item
end

-- As três chaves do painel de filtros.
--
-- `<checkbox>` e `<slider>` são primitivas da Onda 1: elas **disparam a ação** e
-- quem grava a chave é o app — ao contrário do `<popover>` que as envolve, que
-- abre e fecha sozinho. É a mesma divisão que o exemplo `onda1` existe para
-- mostrar, e a razão de estas três funções existirem: sem elas, a caixa não
-- marca e o cursor do slider não anda.
--
-- Do lado Rust isto é um braço só (`chave if value.is_some()` no `update`),
-- porque lá a ação é uma string que se compara. Aqui a ação é o NOME de uma
-- função global, então cada chave precisa da sua.
function so_no_ar()
    ctx.so_no_ar = value
end

function arquivados()
    ctx.arquivados = value
end

function uso_max()
    ctx.uso_max = value
end

-- O `<autocomplete>` com `onSelect` DELEGA: ele entrega o `id`, e quem decide o
-- que fazer com ele é o app. Sem `onSelect`, o widget grava o rótulo na chave e
-- ninguém passa por aqui — que é o caso do campo "Cidade" logo acima na tela.
function escolher_servico()
    ctx.servico_id = value
    ctx.status = "Serviço: " .. tostring(value)
end
