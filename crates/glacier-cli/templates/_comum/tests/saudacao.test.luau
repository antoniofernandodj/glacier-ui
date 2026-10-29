--!strict
-- Testa um service: um módulo Luau puro, que mora em `services/` e é achado pelo
-- `require` porque a pasta está em `test.luau_path`, no `glacier.jsonc`.

local Saudacao = require("saudacao")

--- O nome entra na saudação.
function test_saudacao_com_nome()
    assert_eq(Saudacao.para("Ana"), "Olá, Ana!")
end

--- Espaços nas pontas somem; nome vazio não deixa vírgula sobrando.
function test_saudacao_sem_nome()
    assert_eq(Saudacao.para("  Ana "), "Olá, Ana!")
    assert_eq(Saudacao.para("   "), "Olá!")
end
