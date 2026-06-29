# Plano: Formato GlacierView (`.gv`)

## Motivação

O XML é verboso por natureza: tags de abertura e fechamento duplicam cada
elemento, aspas são obrigatórias em todo valor e declarações de recursos como
`<link rel="stylesheet" href="...">` carregam ruído desnecessário.

O KDL (ver `PLANO_KDL.md`) resolve parte disso, mas mantém aspas em quase
tudo e usa `key=value` inline, o que ainda concentra muita informação numa
linha só para elementos com muitos atributos.

O formato **GlacierView** (`.gv`) propõe uma terceira via: inspirado em CSS e
em linguagens de configuração modernas (HCL, Nix), separa a **estrutura**
(hierarquia de elementos) das **propriedades** (configuração de cada elemento)
com blocos recuados dentro de chaves. O resultado é um arquivo que se lê
naturalmente de cima para baixo, sem fechamentos redundantes e sem excesso de
aspas.

---

## Comparativo

**XML (atual — 27 linhas):**
```xml
<link rel="theme"      href="styles/theme.json" />
<link rel="stylesheet" href="styles/estilos.iss" />
<import name="PerfilCard" from="templates/perfil_card.xml" />

<Container class="card">
    <Column class="stack">
        <Text class="title" content="Estilos via .iss" />
        <Text class="subtitle" content="Contador: {valor}" />
        <Row class="actions">
            <Button class="btn btn-danger" text="-" onClick="decrementar" />
            <Button class="btn btn-success" text="+" onClick="incrementar" />
        </Row>
    </Column>
</Container>
```

**GlacierView (`.gv` — 22 linhas):**
```gv
theme  "styles/theme.json"
style  "styles/estilos.iss"
import "PerfilCard" from="templates/perfil_card.xml"

Container {
    class: card

    Column {
        class: stack

        Text "Estilos via .iss" { class: title }
        Text "Contador: {valor}" { class: subtitle }

        Row {
            class: actions
            Button "-" { class: btn btn-danger;  onClick: decrementar }
            Button "+" { class: btn btn-success; onClick: incrementar }
        }
    }
}
```

---

## Sintaxe completa

### Declarações de recurso (topo do arquivo)

Idênticas ao KDL — nó simples com argumento posicional:

```gv
theme  "styles/theme.json"
style  "styles/app.iss"
import "PerfilCard" from="templates/perfil_card.xml"
data   "data/team.json" as="team"
```

### Estrutura de um nó

```
NomeDoElemento ["conteúdo"] [atributos_inline] {
    propriedade: valor
    propriedade: valor
    FilhoA { ... }
    FilhoB "conteúdo" { ... }
}
```

- O **conteúdo** (para `Text`, `Button`, etc.) é o primeiro argumento entre aspas.
- **Atributos inline** (`key=value`) podem ser usados quando há poucos e curtos.
- **Propriedades em bloco** (`key: value`) são para o corpo do nó.
- Um nó sem filhos e com poucas props pode ser escrito em linha: `Text "Olá" { size: 16 }`
- Um nó sem nada pode omitir as chaves: `Rule`

### Valores sem aspas

Aspas são necessárias apenas quando o valor contém espaços ou caracteres
especiais. Nos demais casos são opcionais:

```gv
// Com aspas (necessário: espaço)
padding: 10 20
content: "Olá, {nome}!"

// Sem aspas (simples)
size: 28
bold: true
color: #ECEFF4
align: Center
width: fill
```

### Atributos booleanos abreviados

```gv
Text "Título" {
    size: 32
    bold         // equivale a bold: true
    italic       // equivale a italic: true
}
```

### Controle de fluxo

**Condicional:**
```gv
Column {
    if: "{logado}"

    Text "Bem-vindo, {usuario}!" { color: #A6E3A1 }
    Button "Sair" { onClick: logout }
}
Column {
    else

    Text "Não conectado." { color: #A6ADC8 }
    Button "Entrar" { onClick: login }
}

// Comparação explícita
Text "(dica)" {
    if: "{logado}"
    equals: false
    size: 12
    color: #6C7086
}
```

**Loop:**
```gv
// Wrapper (múltiplos filhos diferentes por item)
for-each items=usuarios var=u {
    CartaoUsuario {
        nome:    "{u.nome}"
        cargo:   "{u.cargo}"
        inicial: "{u.inicial}"
        cor:     "{u.cor}"
    }
}

// Atributo (elemento único repetido)
CartaoUsuario {
    for-each: usuarios
    var:      u
    nome:     "{u.nome}"
    cargo:    "{u.cargo}"
}
```

### Importação de componentes e uso

```gv
import "CartaoUsuario" from="templates/cartao_usuario.xml"

Container {
    Column {
        spacing: 20

        Text "Equipe ({total})" { size: 26; bold }

        for-each items=usuarios var=u {
            CartaoUsuario {
                nome:    "{u.nome}"
                cargo:   "{u.cargo}"
            }
        }

        Button "Adicionar" { onClick: adicionar; color: #89B4FA }
    }
}
```

### Ponto e vírgula

Propriedades em linha separadas por `;` (opcional, só para compactar):

```gv
Button "-" { onClick: decrementar; color: #BF616A; padding: 10 20 }
```

### Script (componentes com lógica)

O bloco `script` continua como Rust puro ao final do arquivo:

```gv
Container {
    padding: 20
    ...
}

script {
    fn incrementar(&mut self) {
        self.contador += 1;
    }

    fn decrementar(&mut self) {
        self.contador -= 1;
    }
}
```

---

## Regras de parsing

| Construção | Regra |
|-----------|-------|
| Nó com filhos | `Nome { ... }` |
| Nó folha (sem filhos) | `Nome "conteúdo"` ou só `Nome` |
| Conteúdo | Primeiro string entre aspas após o nome |
| Propriedade | `chave: valor` (newline ou `;` separa) |
| Atributo inline | `chave=valor` após o nome, antes do `{` |
| Booleano bare | Identificador sem `:` dentro do bloco |
| Comentário | `//` linha ou `/* */` bloco |
| Declaração de recurso | Nó no topo sem `{`, com arg posicional |

---

## Plano de implementação

### Fase 1 — Parser

**1.1** Criar `src/gv_parser.rs` com um parser recursivo-descendente escrito à
mão, sem dependências externas. Entrada: `&str`. Saída: `UiNode` — o mesmo tipo
já usado pelo parser XML.

Estágios internos do parser:
1. **Tokenizer** — produz `Token` (Ident, Str, Colon, Equals, LBrace, RBrace,
   Semicolon, Newline, EOF). Simples, sem lookahead além de 1 caractere.
2. **Parser de nó** — consome `Nome ["str"] [k=v]* { props e filhos }` ou
   `Nome ["str"] [k=v]*` (folha).
3. **Parser de declaração** — reconhece `theme`, `style`, `import`, `data` no
   topo do arquivo e os converte nos `NodeType` correspondentes
   (`NodeType::Link`, `NodeType::Import`).
4. **Extração de script** — igual ao `strip_script` do XML: busca o nó
   `script { ... }` textualmente antes de parsear o resto.

**1.2** `pub fn parse_gv(input: &str) -> Result<UiNode, String>` — ponto de
entrada público, paralelo ao `UiNode::parse_xml`.

### Fase 2 — Integração no engine

**2.1** Em `src/lib.rs`, na função que lê templates de arquivo, detectar a
extensão:

```rust
let ast = if path.ends_with(".gv") {
    gv_parser::parse_gv(&markup)?
} else {
    UiNode::parse_xml(&markup)?
};
```

**2.2** O `Template::File` já recebe caminho como `String` — nenhuma mudança
na API pública.

**2.3** Hot-reload: `check_reload` já monitora o arquivo pelo caminho; não
precisa saber a extensão. Funciona automaticamente.

**2.4** Expor `parse_gv` no `pub use` de `lib.rs` (opcional, para testes
externos).

### Fase 3 — Exemplos e templates

**3.1** `templates/contador.gv` — porta do contador XML, valida o parser base.

**3.2** `templates/estilos.gv` — valida `theme`, `style` e classes `.iss`.

**3.3** `templates/condicional.gv` — valida `if`/`else` em bloco e como prop.

**3.4** `templates/lista_usuarios.gv` — valida `for-each` e componentes
importados.

**3.5** `examples/contador_gv.rs` — exemplo mínimo rodável apontando para
`templates/contador.gv`.

### Fase 4 — Testes

**4.1** Em `tests/engine_tests.rs`, adicionar casos cobrindo:
- Nó simples com propriedades em bloco
- Conteúdo posicional (`Text "..."`)
- Atributos inline (`key=value` antes do `{`)
- Booleano bare (`bold`)
- `if`/`else` como propriedade de bloco
- `for-each` como nó wrapper e como propriedade
- Declarações de recurso (`theme`, `style`, `import`, `data`)
- Arquivo `.gv` end-to-end no engine (register → render)

---

## Não está no escopo

- Migração automática dos templates XML existentes para `.gv`
- Deprecação do XML ou do KDL — os três formatos coexistem
- Formatter / linter (pode vir depois, após o parser estabilizar)
- Suporte a `.gv` no proc-macro `#[component]` (fica para fase posterior)

---

## Referências

- Parser XML atual: `src/parser.rs`
- Engine e hot-reload: `src/lib.rs`
- Plano KDL (formato alternativo): `PLANO_KDL.md`
- Templates XML de referência: `templates/`
