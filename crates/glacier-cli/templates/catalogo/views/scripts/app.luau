--!strict
-- O motor chama `init` e os handlers pelo nome; o luau-lsp não vê essas
-- chamadas e os reporta como código morto. É o único aviso falso de um projeto
-- novo, desligado aqui para a saída de `make luau` ser só o que importa.
--!nolint FunctionUnused

require("handlers/demo")
require("handlers/dialogos")

-- Um catálogo não tem "estado" de verdade: cada tela é uma vitrine. O `init`
-- só semeia TODA chave que algum demo lê — um widget sem semente aparece vazio
-- (ver AGENTS.md, "Como um widget lê os seus dados").
function init(): ()
  ctx.view = "botoes"
  ctx.menu = "true"  -- a gaveta da barra lateral começa aberta; o ☰ fecha/abre

  -- §2.1 Botões
  ctx.planos = json.encode(json.array({
    { id = "free", label = "Grátis" },
    { id = "pro", label = "Pro" },
    { id = "ent", label = "Enterprise" },
  }))
  ctx.plano, ctx.plano2 = "mensal", "pro"
  ctx.cb_cache, ctx.cb_tri, ctx.tg_escuro = "true", "mixed", "false"

  -- §2.2 Texto
  ctx.ti_nome, ctx.ti_senha, ctx.ta_notas = "", "", ""
  ctx.mi_cpf, ctx.mi_tel, ctx.si_atalho = "", "", ""
  ctx.ce_host = "https://api.exemplo.com"
  ctx.ce_hosts = json.encode(json.array({ "https://api.exemplo.com", "https://api.staging.exemplo.com" }))
  ctx.ac_cidade = ""
  ctx.ac_cidades = json.encode(json.array({ "São Paulo", "Rio de Janeiro", "Belo Horizonte", "Salvador", "Recife", "Belém" }))

  -- §2.3 Numéricas
  ctx.sb_qtd, ctx.sb_preco, ctx.sb_zoom = "3", "19.90", "100"
  ctx.sl_volume, ctx.sl_brilho = "60", "0.5"
  ctx.rs_min, ctx.rs_max = "150", "780"
  ctx.di_ganho, ctx.tu_mes, ctx.ra_nota = "7", "mar", "3"
  ctx.meses = json.encode(json.array({ "jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez" }))
  ctx.pb_uso, ctx.ga_cpu, ctx.lcd_relogio = "64", "73", "12:34"
  ctx.ga_bands = json.encode(json.array({
    { to = 60, color = "#A6E3A1" },
    { to = 85, color = "#F9E2AF" },
    { to = 100, color = "#F38BA8" },
  }))

  -- §2.4 Seleção
  ctx.se_regiao = "sa-east-1"
  ctx.se_regioes = json.encode(json.array({
    { label = "São Paulo", value = "sa-east-1" },
    { label = "N. Virginia", value = "us-east-1" },
    { label = "Frankfurt", value = "eu-central-1" },
  }))
  ctx.lv_servico, ctx.lv_marcados = "api", "api,fila"
  ctx.lv_servicos = json.encode(json.array({
    { id = "api", label = "API de borda", sub = "porta 8080" },
    { id = "fila", label = "Fila de trabalhos", sub = "3 workers" },
    { id = "frio", label = "Réplica fria", sub = "standby" },
  }))
  ctx.tv_sel, ctx.tv_ordem, ctx.tv_larguras = "1", "", ""
  ctx.tv_linhas = json.encode(json.array({
    { id = "1", servico = "api", estado = "ok", latencia = 42 },
    { id = "2", servico = "cache", estado = "alerta", latencia = 310 },
    { id = "3", servico = "fila", estado = "ok", latencia = 88 },
  }))
  ctx.tv_colunas = json.encode(json.array({
    { key = "servico", label = "Serviço" },
    { key = "estado", label = "Estado", width = "110" },
    { key = "latencia", label = "ms", width = "70", align = "right" },
  }))
  ctx.tr_no, ctx.tr_abertos, ctx.cv_caminho = "raiz/src", "raiz,raiz/src", "raiz/src"
  -- Árvore aninhada: literal JSON (uma string crua) em vez de tabelas — o
  -- inferidor do Luau exigiria `items` em TODO nó se fossem tabelas.
  ctx.tr_arvore = [[
    [{"id":"raiz","label":"projeto","items":[
      {"id":"src","label":"src","items":[
        {"id":"main.rs","label":"main.rs"},
        {"id":"lib.rs","label":"lib.rs"}]},
      {"id":"views","label":"views","items":[{"id":"app.gv","label":"app.gv"}]},
      {"id":"Cargo.toml","label":"Cargo.toml"}]}]
  ]]
  ctx.pg_pagina, ctx.pi_pagina, ctx.ft_fonte = "5", "1", "mono"

  -- §2.5 Data e hora
  ctx.hoje = date.today()
  ctx.dh_dia, ctx.dh_ano = ctx.hoje, ctx.hoje
  ctx.dh_mes = string.sub(ctx.hoje, 1, 7)
  ctx.dh_de, ctx.dh_ate = ctx.hoje, date.add(ctx.hoje, { days = 5 })
  ctx.dh_data = ctx.hoje
  ctx.dh_hora, ctx.dh_quando = "09:30", ctx.hoje .. " 09:30"

  -- §2.6 Displays
  ctx.di_tags = json.encode(json.array({ "web", "api", "urgente" }))
  ctx.di_qr = "https://glacier-ui.dev"

  -- §2.7 Containers
  ctx.ct_proxy, ctx.ct_split = "false", "240 fill"
  ctx.ct_ferramenta, ctx.ct_abertas = "medidas", "rede"
  ctx.ct_ed_x, ctx.ct_ed_y, ctx.ct_ed_w, ctx.ct_ed_h = "12", "12", "220", "120"
  ctx.ct_co_x, ctx.ct_co_y, ctx.ct_co_w, ctx.ct_co_h = "150", "70", "200", "110"
  ctx.ct_mode, ctx.ct_size, ctx.ct_fx, ctx.ct_fy = "left", "180", "40", "40"

  -- §2.8 Navegação
  ctx.nv_aba, ctx.nv_aba2 = "geral", "geral"
  ctx.nv_abas = json.encode(json.array({
    { id = "geral", label = "Geral" },
    { id = "rede", label = "Rede" },
    { id = "avancado", label = "Avançado" },
  }))
  ctx.nv_painel, ctx.nv_passo, ctx.nv_passo_valido = "resumo", "plano", "1"
  ctx.nv_pagina, ctx.nv_gaveta = "0", "false"

  -- §2.9 Barras
  ctx.bar_status = "ocioso"

  -- §2.12 Overlays
  ctx.ov_menu, ctx.ov_pop, ctx.ov_notif, ctx.ov_marcados = "false", "false", "true", ""
  ctx.ov_caixas = json.encode(json.array({
    { id = "a", x = 16, y = 16, w = 80, h = 54 },
    { id = "b", x = 110, y = 16, w = 80, h = 54 },
    { id = "c", x = 204, y = 16, w = 80, h = 54 },
    { id = "d", x = 16, y = 88, w = 80, h = 54 },
    { id = "e", x = 110, y = 88, w = 80, h = 54 },
  }))
  ctx.ov_tarefas = json.encode(json.array({
    { id = "1", nome = "desenhar o wireframe" },
    { id = "2", nome = "revisar o texto" },
    { id = "3", nome = "publicar" },
  }))

  -- §2.13 Gráficos
  ctx.gr_serie = json.encode(json.array({
    { label = "seg", value = 12 }, { label = "ter", value = 31 },
    { label = "qua", value = 24 }, { label = "qui", value = 40 },
    { label = "sex", value = 33 },
  }))
  ctx.gr_carga = json.encode(json.array({
    { name = "API", points = json.array({ 12, 19, 7, 22, 15 }) },
    { name = "DB", points = json.array({ 20, 15, 22, 18, 26 }), color = "#89B4FA" },
  }))
  ctx.gr_barras = json.encode(json.array({
    { label = "web", value = 42 }, { label = "api", value = 31 },
    { label = "db", value = 58 }, { label = "cdn", value = 19 },
  }))
  ctx.gr_fatias = json.encode(json.array({
    { label = "sucesso", value = 72 }, { label = "alerta", value = 18 },
    { label = "erro", value = 10 },
  }))
  ctx.gr_lat = json.encode(json.array({ 8, 12, 9, 15, 11, 18, 10, 7, 13, 9 }))

  -- §2.10 Diálogos
  ctx.dlg_ultimo, ctx.dlg_resposta, ctx.dlg_cor_val = "—", "—", "#89B4FA"
end
