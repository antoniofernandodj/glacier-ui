use glacier_ui::{Component, Context, Template};

pub struct Contador {
    valor: i32,
    passo: i32,
}

impl Contador {
    pub fn new() -> Self {
        Self { valor: 0, passo: 1 }
    }

    fn publicar(&self, ctx: &mut Context) {
        ctx.set("contador", self.valor.to_string());
        ctx.set("passo", self.passo.to_string());
    }
}

impl Component for Contador {
    fn name(&self) -> &str {
        "contador"
    }

    fn template(&self) -> Template {
        Template::File("views/contador.gv".into())
    }

    fn init(&mut self, ctx: &mut Context) {
        self.publicar(ctx);
    }

    fn update(&mut self, action: &str, value: Option<&str>, ctx: &mut Context) {
        match action {
            "somar" => self.valor += self.passo,
            "subtrair" => self.valor -= self.passo,
            "zerar" => self.valor = 0,
            "mudar_passo" => {
                let Some(texto) = value else { return };
                if let Ok(n) = texto.trim().parse::<i32>() {
                    self.passo = n;
                }
                ctx.set("passo_texto", texto);
            }
            _ => return,
        }
        self.publicar(ctx);
    }
}
