--!strict
--
-- A mesma tela de `examples/gvb/i18n`, com o estado em Luau. O markup traduz o que é
-- texto fixo com `{t("…")}`; o que o script monta (plural com argumento, número, data)
-- usa o global `i18n` e é recalculado em `atualizar()` sempre que algo muda.

local contagem = 1

local function atualizar()
    ctx.ola = i18n.t("ola", { nome = "Ana" })
    ctx.itens = i18n.t("itens", { n = contagem })
    ctx.preco_fmt = i18n.num(1234567.891, 2)
    ctx.quando_fmt = i18n.date("2026-10-31")
end

local function idioma(l: string)
    i18n.set_locale(l)
    atualizar()
end

function init()
    ctx["__locale"] = i18n.locale()
    atualizar()
end

function mais()
    contagem += 1
    atualizar()
end

function menos()
    contagem = math.max(0, contagem - 1)
    atualizar()
end

function idioma_pt() idioma("pt-BR") end
function idioma_en() idioma("en") end
function idioma_ru() idioma("ru") end
function idioma_ar() idioma("ar") end
