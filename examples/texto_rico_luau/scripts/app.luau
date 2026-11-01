--!strict
--
-- O estado do exemplo `texto_rico`, em Luau: o mesmo que o `impl Component` da versão
-- Rust faz em `init` e `update`. As chaves `usuario` e `cliques` alimentam o
-- `{usuario}` e o `{cliques}` do prose; `contar` é a ação do link "cliques".

function init()
    ctx.usuario = "Ana"
    ctx.cliques = 0
end

function contar()
    ctx.cliques = ctx.cliques + 1
end
