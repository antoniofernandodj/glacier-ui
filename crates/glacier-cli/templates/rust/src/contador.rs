//! O componente: UI (o template), comportamento (`update`) e estado (os campos)
//! no mesmo lugar.

use glacier_ui::{Component, Context, Template};

/// Estado tipado. É a diferença prática para o caminho Luau: aqui `passo` é um
/// `i32`, não uma string do contexto, e trocar o tipo quebra a compilação em
/// vez de virar um `nil` em runtime.
pub struct Contador {
    valor: i32,
    passo: i32,
}

impl Contador {
    pub fn new() -> Self {
        Self { valor: 0, passo: 1 }
    }

    /// Copia o estado para o contexto — é o que os `{marcadores}` do template
    /// leem. O contexto guarda strings; a conversão é sempre aqui, num lugar só.
    fn publicar(&self, ctx: &mut Context) {
        ctx.set("contador", self.valor.to_string());
        ctx.set("passo", self.passo.to_string());
    }
}

impl Component for Contador {
    /// O nome com que o componente é registrado e roteado (`set_initial_screen`).
    fn name(&self) -> &str {
        "contador"
    }

    /// O caminho é relativo ao diretório de onde o app roda — a raiz do projeto,
    /// se for `cargo run`. `Template::File` mantém o hot-reload: salvar o `.gv`
    /// redesenha sem recompilar.
    fn template(&self) -> Template {
        Template::File("views/contador.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        self.publicar(ctx);
    }

    /// Todo clique do template chega aqui. `value` traz o texto de um
    /// `on_change`/`on_toggle` (o passo, no caso) e é `None` num clique simples.
    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            "somar" => self.valor += self.passo,
            "subtrair" => self.valor -= self.passo,
            "zerar" => self.valor = 0,
            "mudar_passo" => {
                // Um campo de texto pode conter qualquer coisa enquanto é
                // digitado: um passo ilegível mantém o anterior em vez de zerar
                // o comportamento do app no meio da digitação.
                let Some(texto) = value else { return };
                if let Ok(n) = texto.trim().parse::<i32>() {
                    self.passo = n;
                }
                ctx.set("passo_texto", texto);
            }
            // Uma ação desconhecida não é erro: ela pode pertencer a outro
            // componente da árvore, e o motor já cuidou do roteamento.
            _ => return,
        }
        self.publicar(ctx);
    }
}
