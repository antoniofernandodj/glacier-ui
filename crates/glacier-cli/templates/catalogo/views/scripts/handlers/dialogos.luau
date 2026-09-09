--!strict
--!nolint FunctionUnused
-- §2.10 — os modais. `<dialog name>` é markup (em dialogos.gv); estes aqui são
-- as CHAMADAS suspensivas da camada Luau.

-- Aceite do `<dialog name="cat_form">`: no aceite, o rascunho `__dialog.*` ainda
-- está no contexto (o motor o apaga depois). Leia aqui.
function dlg_salvar(): ()
  local nome = ctx["__dialog.nome"] or ""
  local rep = ctx["__dialog.replicas"] or "1"
  ctx.dlg_ultimo = `{nome} · {rep} réplica(s)`
end

function dlg_confirmar(): ()
  local ok = confirm({
    title = "Remover serviço?",
    message = "A ação não pode ser desfeita.",
    confirm_label = "Remover",
    destructive = true,
  })
  ctx.dlg_resposta = if ok then "confirmou" else "cancelou"
end

function dlg_prompt(): ()
  local v = prompt({ title = "Renomear", label = "Novo nome", value = "api-gateway" })
  ctx.dlg_resposta = if v ~= nil then `prompt = "{v}"` else "prompt: desistiu"
end

function dlg_cor(): ()
  local c = pick_color({ title = "Cor do tema", value = ctx.dlg_cor_val })
  if c ~= nil then
    ctx.dlg_cor_val = c
    ctx.dlg_resposta = `cor = {c}`
  end
end

function dlg_arquivo(): ()
  local caminho = open_file({
    title = "Escolha um arquivo",
    filters = { { name = "Todos", extensions = { "*" } } },
  })
  ctx.dlg_resposta = if caminho ~= nil then caminho else "arquivo: cancelou"
end

function dlg_progresso(): ()
  progress({ title = "Processando", label = "trabalhando…", max = 100 })
  local feito = 0
  local function passo(): ()
    feito += 25
    progress_set(feito, `{feito}%`)
    if feito >= 100 then
      progress_close()
      ctx.dlg_resposta = "progresso concluído"
    else
      after(350, passo)
    end
  end
  after(350, passo)
end

return {}
