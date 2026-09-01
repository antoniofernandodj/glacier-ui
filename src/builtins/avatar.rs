/// O avatar: a foto de uma pessoa num círculo, com as iniciais como reserva
/// quando não há foto.
///
/// ```xml
/// <Avatar src="fotos/ana.png" />
/// <Avatar initials="AF" bg="#89B4FA" />
/// ```
///
/// A reserva não é enfeite: numa lista de usuários, a foto que falta é o caso
/// comum (conta nova, upload que falhou, contato externo), e um buraco vazio
/// quebra o alinhamento da linha inteira. Por isso o widget sempre ocupa o
/// mesmo espaço — com foto ou sem.
///
/// # Props
///
/// - `src`      — caminho/URL da imagem. Presente, vence as iniciais.
/// - `initials` — 1–2 letras para a reserva. Default: `?`.
/// - `size`     — diâmetro em px. Default: `40`.
/// - `bg`       — fundo do círculo de iniciais. Default: `#8080803d`.
/// - `fg`       — cor das iniciais. Default: `#cdd6f4`.
///
/// # Por que este builtin não tem folha `<style>`
///
/// Os outros recipientes da lib deixam as cores numa classe global, para o app
/// repintar por `.gss`. Aqui as cores são **props**, e as duas coisas não cabem
/// no mesmo atributo: o eval resolve um campo com
/// `inline.map(process_tpl).or_else(classe)`, então um `background="{bg}"`
/// escrito no template vence a classe **mesmo quando a prop não vem e ele
/// resolve para vazio** — o círculo sairia sem fundo nenhum. Como avatar é o
/// caso em que a cor tende a variar por instância (uma por usuário), a prop
/// ganhou, e o default virou literal.
///
/// # Limites conhecidos
///
/// - **O corte circular é da imagem, não do widget.** A foto usa o
///   `clip="Circle"` do `<Image>`; o círculo das iniciais é um `border_radius`
///   igual à metade do lado. Um `size` ímpar pode deixar meio pixel de
///   diferença entre as duas formas.
/// - **`initials` não é derivado do nome.** Passar `initials="AF"` é trabalho
///   de quem chama — o widget não recebe o nome completo, e cortar iniciais de
///   um nome ("Maria da Silva" → "MS"? "MD"?) é decisão de produto, não de
///   layout.
/// - **Sem indicador de presença.** O pontinho verde de "online" no canto
///   pediria sobreposição (`Stack`) dentro de um builtin; hoje se faz por fora,
///   com o `<Badge>` ao lado.
use crate::component::{Component, Context, Template};

pub struct Avatar;

impl Component for Avatar {
    fn name(&self) -> &str {
        "Avatar"
    }

    fn template(&self) -> Template {
        // `border_radius="{size|40}"` no braço das iniciais: um raio >= metade
        // do lado já arredonda até o círculo, então passar o lado inteiro
        // dispensa uma prop `radius` e continua certo para qualquer `size`.
        //
        // `align_x`/`align_y` centram a letra no círculo — sem eles ela encosta
        // no canto superior esquerdo, que é o default do container.
        Template::Inline(
            r##"<Container>
                    <template if="{src}" notEquals="">
                        <Image
                            source="{src}"
                            width="{size|40}"
                            height="{size|40}"
                            clip="Circle"
                        />
                    </template>

                    <template else>
                        <Container
                            background="{bg|#8080803d}"
                            width="{size|40}"
                            height="{size|40}"
                            border_radius="{size|40}"
                            align_x="center"
                            align_y="center"
                        >
                            <Text content="{initials|?}" color="{fg|#cdd6f4}" bold="true" />
                        </Container>
                    </template>
                </Container>"##
                .to_string(),
        )
    }

    fn update(&mut self, _action: &str, _value: Option<&str>, _ctx: &mut Context) {
        // Apresentacional: sem estado, sem comportamento.
    }
}
