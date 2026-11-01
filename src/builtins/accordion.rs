/// `Accordion` / `QToolBox`: seções empilhadas que abrem e fecham — animadas
/// desde a 0.90.
///
/// São **os dois modos do mesmo widget**, e a diferença cabe numa linha: o
/// accordion guarda um **conjunto** de seções abertas, o tool box guarda **uma**
/// — e por isso o primeiro precisou do `contains` (0.84) e o segundo não
/// precisa de nada.
///
/// ```gv
/// accordion {
///   accordionitem(title = Rede, value = abertas, open = @abertas, id = rede) {
///     input(:value = host)
///   }
///   accordionitem(title = Disco, value = abertas, open = @abertas, id = disco) {
///     text("Nada por aqui ainda.")
///   }
/// }
/// toolbox {
///   toolboxitem("…", title = Ferramentas, value = secao, open = @secao, id = ferr)
///   toolboxitem("…", title = Camadas, value = secao, open = @secao, id = cam)
/// }
/// ```
///
/// # Por que uma tag por item, e não uma coleção
///
/// Porque o **conteúdo** de cada seção é diferente, e conteúdo é do app: um
/// `items="secoes"` daria os títulos, mas o corpo teria de sair de um
/// `slot` por seção — e os nomes de slot são fixos no template do
/// componente (0.67), então o widget não pode inventar um por item da coleção.
///
/// A saída é a do próprio Qt: `QToolBox::addItem(widget, "Título")` também
/// recebe uma seção por chamada. Aqui a chamada é uma tag, o widget é o que
/// está escrito entre elas, e o `slot` (0.65) é o que torna isso possível.
///
/// # O `Accordion`/`ToolBox` de fora é só a moldura
///
/// Um `Column` com espaçamento e uma borda opcional. Todo o comportamento
/// mora no **item** — que por isso funciona sozinho, sem a moldura, quando o
/// que se quer é uma seção dobrável solta no meio de um formulário.
///
/// # `value` e `open` andam em par
///
/// A terceira vez que este par aparece na biblioteca (`TabBar`, `ListView`,
/// agora aqui), e sempre pelo mesmo motivo: o template precisaria ler o valor
/// da chave cujo *nome* está numa prop — a indireção `{{value}}` que o
/// interpolador não tem. `value` é o **nome** da chave; `open` é o **valor**
/// atual dela.
///
/// # Abre e fecha animado (0.90)
///
/// O corpo desliza para dentro e para fora em 180ms, em vez de aparecer e
/// sumir entre dois quadros. Quem faz isso é a primitiva `Reveal`
/// (ver [`crate::reveal`]), que o item já embrulha sozinho: o corpo está
/// **sempre** na árvore — é de onde a altura encolhe ao fechar — e o widget é
/// que decide quanto dele mostrar a cada quadro.
///
/// A consequência prática é uma só, e vale saber: o conteúdo de uma seção
/// **fechada** continua sendo montado e medido (só não é desenhado nem recebe
/// clique). Para um corpo caro, `virtualize` na lista lá dentro continua sendo
/// a saída (ver `PRIMITIVAS.md`).
///
/// `duration="0"` no item desliga a animação e devolve o comportamento de
/// antes.
///
/// # Props do item
///
/// - `title`    — o rótulo do cabeçalho.
/// - `value`    — **nome** da chave que guarda o que está aberto.
/// - `open`     — o valor atual dessa chave.
/// - `id`       — o identificador desta seção dentro da chave.
/// - `sub`      — segunda linha do cabeçalho, menor. Opcional.
/// - `padding`  — espaço interno do corpo. Default: `12`.
/// - `spacing`  — espaço entre os filhos do corpo. Default: `8`.
/// - `duration` — duração do abre/fecha em ms. Default: `180`; `0` desliga.
///
/// # Aparência
///
/// `.accordion-head` / `.accordion-body` na folha global do template. O
/// indicador é `▾`/`▸` — texto, não ícone: o widget não pode depender de um
/// arquivo `.svg` que o app talvez não tenha.
///
/// # Classes nos nós de dentro
///
/// `class` no uso aplica na **raiz** do item (a coluna que empilha cabeçalho e
/// corpo). Para alcançar um nó lá dentro, uma prop por nó — o padrão da
/// biblioteca, e o que faz um accordion caber num tema do app sem reescrever o
/// widget:
///
/// - `head_class`  — o `Button` do cabeçalho (aceita `:hover`).
/// - `mark_class`  — o glifo `▾`/`▸`.
/// - `title_class` — o título.
/// - `sub_class`   — a segunda linha.
/// - `body_class`  — a coluna do corpo (dentro do `Reveal` que a anima).
///
/// ```gv
/// accordionitem(title = Rede, value = abertas, open = @abertas, id = rede, head_class = sec_cabecalho, body_class = sec_corpo) {
///   input(:value = host)
/// }
/// ```
use crate::component::{Component, Context, Template};

pub struct Accordion;
pub struct AccordionItem;
pub struct ToolBox;
pub struct ToolBoxItem;

/// O corpo comum das duas molduras: uma `Column` que só empilha o que
/// escreverem dentro dela. `op` distingue o que o item faz no clique.
fn moldura(largura_padrao: &str) -> Template {
    Template::Inline(
        r##"
column(spacing = "{spacing|4}", width = "{width|%LARGURA%}") {
  slot
}"##
            .replace("%LARGURA%", largura_padrao),
    )
}

/// O item: um cabeçalho clicável e um corpo que **abre e fecha animado**.
///
/// `op` é a operação que o `update` executa — `toggle` (conjunto) ou `only`
/// (um por vez) —, e `teste` é a diretiva que decide o estado: `contains`
/// para o conjunto, `equals` para o único.
///
/// # Por que os dois braços repetem o corpo inteiro
///
/// Porque o `Reveal` precisa de um `open` **booleano**, e derivá-lo de um
/// `contains`/`equals` é exatamente a indireção que o interpolador não tem
/// (a mesma razão de `value`/`open` andarem em par, lá em cima). Então quem
/// decide continua sendo o `template(if)`, e cada braço escreve o literal
/// que lhe cabe: `open="true"` num, `open="false"` no outro.
///
/// O que **não** muda entre os dois braços é a FORMA da árvore — cabeçalho e
/// `Reveal`, nessa ordem, nos dois. É disso que a animação depende: o
/// `iced` casa o estado da árvore por posição, então o `Reveal` do braço
/// aberto e o do braço fechado são o MESMO widget para ele, e a transição de
/// `open` é o que o `diff()` dele enxerga (ver `ANIMACOES.md`). Um braço com
/// um filho a mais que o outro desalinharia as posições e a animação sumiria
/// — o corpo voltaria a piscar.
fn item(op: &str, teste: &str) -> Template {
    let corpo = r##"
reveal(open = %ABERTO%, duration = "{duration|180}", width = fill) {
  column(
    class = "accordion-body {body_class}",
    spacing = "{spacing|8}",
    padding = "{padding|12}",
    width = fill,
  ) {
    slot
  }
}"##;

    Template::Inline(
        r##"
column(spacing = 0, width = "{width|fill}") {
  style """
    .accordion-head {
      color: #8080801f;
      text_color: #cdd6f4;
      border_width: 0;
      border_radius: 5;
    }
    .accordion-head:hover { background: #8080803d; }
    .accordion-mark { color: #80868d; }
    .accordion-sub  { color: #80868d; }
  """

  template(if = @open, %TESTE%) {
    %ABERTO%
    %CORPO_ABERTO%
  }

  template(else) {
    %FECHADO%
    %CORPO_FECHADO%
  }
}"##
        .replace("%TESTE%", teste)
        .replace("%ABERTO%", &cabecalho("▾").replace("%OP%", op))
        .replace("%FECHADO%", &cabecalho("▸").replace("%OP%", op))
        .replace("%CORPO_ABERTO%", &corpo.replace("%ABERTO%", "true"))
        .replace("%CORPO_FECHADO%", &corpo.replace("%ABERTO%", "false")),
    )
}

/// O cabeçalho, com o glifo do estado. Repetido nos dois braços porque a
/// alternativa — um `slot` interno — não existe: o slot é o conteúdo de
/// quem USA o widget, não um mecanismo de reuso dentro do template.
fn cabecalho(marca: &str) -> String {
    r##"
button(
  class = "accordion-head {head_class}",
  on_click = "%OP%:{value}|{id}",
  padding = "{head_padding|9 12}",
  width = fill,
) {
  row(spacing = 8, align_y = center, width = fill) {
    text(class = "accordion-mark {mark_class}", content = "%MARCA%", size = 12)
    column(spacing = 1, width = fill) {
      text(class = "{title_class}", content = "{title}", size = "{size|13}", bold = true)
      template(if = @sub, not_equals = "") {
        text(class = "accordion-sub {sub_class}", content = "{sub}", size = 11)
      }
    }
  }
}"##
    .replace("%MARCA%", marca)
}

impl Component for Accordion {
    fn name(&self) -> &str {
        "Accordion"
    }
    fn template(&self) -> Template {
        moldura("fill")
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {
        // Moldura pura: o comportamento é do item.
    }
}

impl Component for ToolBox {
    fn name(&self) -> &str {
        "ToolBox"
    }
    fn template(&self) -> Template {
        moldura("fill")
    }
    fn update(&mut self, _a: &str, _v: Option<&str>, _c: &mut Context) {}
}

impl Component for AccordionItem {
    fn name(&self) -> &str {
        "AccordionItem"
    }

    fn template(&self) -> Template {
        // `contains`: a chave guarda o CONJUNTO das seções abertas, e é aqui
        // que o habilitador da 0.84 paga o aluguel — sem ele, cada seção
        // precisaria da sua própria chave, que é exatamente o "estado por
        // instância" que o plano dizia faltar.
        item("toggle", r#"contains = "{id}""#)
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `toggle:abertas|rede` — a chave que o app nomeou e o id desta seção.
        let Some(("toggle", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let (chave, id) = (chave.trim(), id.trim());
        if chave.is_empty() || id.is_empty() {
            return;
        }
        let atual = ctx.get(chave).cloned().unwrap_or_default();
        ctx.set(chave, super::list_view::alterna_no_conjunto(&atual, id));
    }
}

impl Component for ToolBoxItem {
    fn name(&self) -> &str {
        "ToolBoxItem"
    }

    fn template(&self) -> Template {
        // `equals`: a chave guarda UMA seção. É o `TabBar` na vertical, e por
        // isso nunca precisou do `contains` — nem esteve bloqueado por nada.
        item("only", r#"equals = "{id}""#)
    }

    fn update(&mut self, action: &str, _value: Option<&str>, ctx: &mut Context) {
        // `only:secao|ferr` — abre esta e fecha as outras. Clicar na que já
        // está aberta **fecha** (a chave vira vazia): é o `QToolBox` sendo
        // gentil, e é o que a pessoa espera de um cabeçalho clicável.
        let Some(("only", payload)) = action.split_once(':') else {
            return;
        };
        let Some((chave, id)) = payload.split_once('|') else {
            return;
        };
        let (chave, id) = (chave.trim(), id.trim());
        if chave.is_empty() || id.is_empty() {
            return;
        }
        let ja_aberta = ctx.get(chave).is_some_and(|v| v.trim() == id);
        ctx.set(
            chave,
            if ja_aberta {
                String::new()
            } else {
                id.to_string()
            },
        );
    }
}
