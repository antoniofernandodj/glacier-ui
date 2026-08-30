--!strict
-- app.luau — ponto de entrada do <script src> de views/app.gv.
--
-- Não define lógica própria: só carrega cada pacote de `handlers/`, para que as
-- funções globais deles (as que os on_click dos templates nomeiam) estejam
-- registradas antes de qualquer clique acontecer. Cada handler cuida de UM
-- domínio; o estado mutável compartilhado vive em `state.luau`.
--
-- Convenção de `require` (resolve relativo ao ARQUIVO que chama, como em Node):
--   · irmão no mesmo diretório  -> nome nu:      require("dados")
--   · pacote pai                -> prefixo ../:  require("../state")
--   · a partir DESTE arquivo    -> caminho:      require("handlers/dados")

local Dados = require("handlers/dados")
require("handlers/nav")

-- `init` é chamado pelo motor quando a tela entra. É onde o contexto ganha os
-- valores que o markup já lê no primeiro quadro.
function init(): ()
    ctx.view = "home"
    ctx.status = ""
    ctx.ip = "-"
    Dados.publicar()
end
