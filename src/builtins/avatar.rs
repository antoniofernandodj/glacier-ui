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
/// - `bg`       — fundo do círculo de iniciais. Default: `.avatar-fallback`.
/// - `fg`       — cor das iniciais. Default: `.avatar-initials`.
///
/// # Classes nos nós de dentro
///
/// `class` no uso aplica na raiz; para alcançar o que está dentro dela há três
/// props, uma por nó — e aqui elas valem mais do que no resto da biblioteca,
/// porque as cores deste widget são props e uma classe é o único jeito de
/// pintar por `@media` ou por pseudo-estado:
///
/// - `image_class`    — a `<Image>` do braço com foto.
/// - `fallback_class` — o círculo das iniciais.
/// - `initials_class` — o `<Text>` com as letras.
///
/// ```xml
/// <Avatar initials="AF" fallback_class="avatar_reserva" />
/// ```
///
/// # Prop e classe convivem no mesmo campo — desde a 0.89
///
/// Este widget foi por um tempo o único da lib **sem** folha `<style>`, e a
/// causa era do motor: um campo resolvia por `inline.or_else(classe)`, então um
/// `background="{bg}"` escrito no template vencia a classe **mesmo quando a
/// prop não vinha e ele resolvia para vazio** — o círculo sairia sem fundo
/// nenhum. O default teve de virar literal (`{bg|#8080803d}`), e um literal
/// vence sempre: nenhuma classe pintava o avatar.
///
/// O `resolve` do eval hoje descarta o vazio antes de consultar a classe, e a
/// escada documentada volta a valer para os dois: **prop → classe injetada →
/// default da lib**. Por isso as cores saíram para `.avatar-fallback` /
/// `.avatar-initials` e as props perderam o default inline.
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
                    <style>
                        .avatar-fallback { background: #8080803d; }
                        .avatar-initials { color: #cdd6f4; }
                    </style>

                    <template if="{src}" notEquals="">
                        <Image
                            class="{image_class}"
                            source="{src}"
                            width="{size|40}"
                            height="{size|40}"
                            clip="Circle"
                        />
                    </template>

                    <template else>
                        <Container
                            class="avatar-fallback {fallback_class}"
                            background="{bg}"
                            width="{size|40}"
                            height="{size|40}"
                            border_radius="{size|40}"
                            align_x="center"
                            align_y="center"
                        >
                            <Text
                                class="avatar-initials {initials_class}"
                                content="{initials|?}"
                                color="{fg}"
                                bold="true"
                            />
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
