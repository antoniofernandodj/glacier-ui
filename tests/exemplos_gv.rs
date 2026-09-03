//! Todo `.gv` do repositório parseia, e todo `.gv` está na forma com cabeçalho.
//!
//! Desde a 0.61 a segunda parte é redundante com a primeira — um arquivo sem
//! cabeçalho não parseia mais. O teste mantém as duas assertivas mesmo assim,
//! porque elas falham por motivos diferentes e a mensagem importa: a primeira
//! pega um exemplo quebrado (o erro do parser diz o quê e onde), a segunda diz
//! qual arquivo ficou para trás numa migração.

use glacier_ui::{UiNode, normalize_bare_directives, strip_script};
use std::path::{Path, PathBuf};

fn gvs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            gvs(&p, out);
        } else if p.extension().is_some_and(|e| e == "gv") {
            out.push(p);
        }
    }
}

/// Tira os comentários para "a primeira tag" ser a primeira tag de verdade.
fn sem_comentarios(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut resto = src;
    while let Some(i) = resto.find("<!--") {
        out.push_str(&resto[..i]);
        resto = match resto[i..].find("-->") {
            Some(j) => &resto[i + j + 3..],
            None => "",
        };
    }
    out.push_str(resto);
    out
}

#[test]
fn todo_exemplo_parseia_e_tem_cabecalho() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    gvs(&raiz.join("examples"), &mut arquivos);
    gvs(&raiz.join("templates"), &mut arquivos);
    // Os presets que o `glacier new` materializa. Um `.gv` quebrado ali não
    // aparece em nenhum exemplo — ele só falha na máquina de quem acabou de
    // criar o primeiro projeto, que é o pior lugar possível para falhar.
    gvs(&raiz.join("crates/glacier-cli/templates"), &mut arquivos);
    arquivos.sort();
    assert!(
        arquivos.len() >= 35,
        "só {} .gv encontrados — o caminho está errado?",
        arquivos.len()
    );

    for arquivo in arquivos {
        // Os presets da CLI trazem `{{titulo}}` onde vai o nome do projeto; o
        // parse não se importa com o texto, mas deixar o marcador cru tornaria
        // o teste dependente de ele nunca cair dentro de uma tag.
        let src = std::fs::read_to_string(&arquivo)
            .expect("ler .gv")
            .replace("{{titulo}}", "Exemplo");
        let relativo = arquivo.strip_prefix(raiz).unwrap_or(&arquivo).display();

        let primeira = sem_comentarios(&src)
            .split('<')
            .nth(1)
            .unwrap_or("")
            .trim_start()
            .to_string();
        assert!(
            primeira.starts_with("screen") || primeira.starts_with("component"),
            "{relativo}: começa com <{}…>, e todo .gv começa com <screen> ou <component>",
            primeira.split_whitespace().next().unwrap_or("?")
        );

        // As mesmas duas passadas que o motor faz antes de parsear (ver
        // `parse_markup` em src/lib.rs): recortar o `<script>` por texto e
        // reescrever as diretivas nuas (`else` -> `else=""`). Sem elas o teste
        // seria mais ESTRITO que o motor, e recusaria markup que abre sem
        // problema — um `else` pelado, ou um `<` dentro de um script Luau.
        let (markup, _script) = strip_script(&src);
        let markup = normalize_bare_directives(&markup);
        UiNode::parse_xml_in(&markup, Some(&relativo.to_string()))
            .unwrap_or_else(|e| panic!("{relativo} não parseia: {e}"));
    }
}

/// Todo `<script src="…">` aponta para um arquivo que existe.
///
/// O parse de um `.gv` não resolve o `src` — o bloco de script é recortado por
/// texto antes, e o caminho só é lido quando o motor REGISTRA o componente. Um
/// `src` errado passava por todos os testes e só aparecia ao abrir o app; foi
/// assim que `examples/stream_lua` ficou apontando para um `.luau` enquanto o
/// arquivo no disco era `.lua`.
#[test]
fn todo_script_src_aponta_para_um_arquivo_existente() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    gvs(&raiz.join("examples"), &mut arquivos);
    gvs(&raiz.join("templates"), &mut arquivos);
    gvs(&raiz.join("crates/glacier-cli/templates"), &mut arquivos);

    let mut conferidos = 0;
    for arquivo in arquivos {
        let src = std::fs::read_to_string(&arquivo).expect("ler .gv");
        for capturado in srcs_de_script(&src) {
            // O `src` resolve relativo ao diretório do próprio `.gv` (ver
            // `luau::resolve_script`), não ao diretório de onde o app roda.
            let alvo = arquivo.parent().unwrap_or(raiz).join(&capturado);
            assert!(
                alvo.is_file(),
                "{}: <script src=\"{capturado}\"> não existe ({})",
                arquivo.strip_prefix(raiz).unwrap_or(&arquivo).display(),
                alvo.display()
            );
            conferidos += 1;
        }
    }
    assert!(
        conferidos >= 5,
        "só {conferidos} <script src> conferidos — o caminho está errado?"
    );
}

/// Os valores de `src`/`from` das tags `<script>` de `xml`, na ordem em que
/// aparecem. Varredura por texto, do mesmo jeito que o motor faz.
fn srcs_de_script(xml: &str) -> Vec<String> {
    let minusculo = xml.to_ascii_lowercase();
    let mut saida = Vec::new();
    let mut de = 0;

    while let Some(i) = minusculo[de..].find("<script").map(|i| de + i) {
        let Some(fim) = minusculo[i..].find('>').map(|f| i + f) else {
            break;
        };
        let tag = &xml[i..fim];
        for atributo in ["src=\"", "from=\""] {
            if let Some(a) = tag.find(atributo) {
                let valor = &tag[a + atributo.len()..];
                if let Some(f) = valor.find('"') {
                    saida.push(valor[..f].to_string());
                }
                break;
            }
        }
        de = fim;
    }
    saida
}

/// Nenhum exemplo — nem preset da CLI — pinta a janela inteira.
///
/// A regra de estilo que mais paga (`AGENTS.md`, "Não pinte a janela inteira"):
/// o tema já pinta o fundo, então um `background` num nó `width: fill; height:
/// fill` é uma camada redobrada em **cada pixel da tela** — invisível e cara.
/// Numa GPU integrada foi o maior ganho isolado de um app real, e apareceu lá
/// nove vezes.
///
/// O teste existe porque o erro é fácil de reintroduzir: escrever
/// `background="#1E1E2E"` na raiz é o reflexo de quem quer um app escuro, e
/// funciona — só custa. A saída certa é o `theme.json`, que é o que todo
/// exemplo daqui usa.
///
/// Ele varre as duas formas de escrever a mesma coisa: o atributo no nó e a
/// classe GSS (inline num `<style>` ou num `.gss` ao lado).
#[test]
fn nenhum_exemplo_pinta_a_janela_inteira() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut arquivos = Vec::new();
    gvs(&raiz.join("examples"), &mut arquivos);
    // Os presets do `glacier new` entram junto: um preset que pinta a janela
    // ensina o hábito a todo projeto novo, e o AGENTS.md que ele mesmo
    // materializa diz o contrário — os três primeiros nasceram assim.
    gvs(&raiz.join("crates/glacier-cli/templates"), &mut arquivos);
    arquivos.sort();

    let mut culpados = Vec::new();

    for arquivo in &arquivos {
        let src = std::fs::read_to_string(arquivo).expect("ler .gv");
        let relativo = arquivo.strip_prefix(raiz).unwrap_or(arquivo).display();

        // Forma 1: os atributos na própria tag.
        for tag in tags_de(&src) {
            let baixo = tag.to_ascii_lowercase();
            if baixo.contains("background=")
                && baixo.contains("width=\"fill\"")
                && baixo.contains("height=\"fill\"")
            {
                culpados.push(format!(
                    "{relativo}: <{}> com background num nó fill/fill",
                    baixo.split_whitespace().next().unwrap_or("?")
                ));
            }
        }

        // Forma 2: uma classe que declara os três juntos, num `<style>` inline.
        for regra in regras_gss(&src) {
            if regra.contains("background")
                && regra.contains("width: fill")
                && regra.contains("height: fill")
            {
                culpados.push(format!("{relativo}: classe GSS com background e fill/fill"));
            }
        }
    }

    // E as folhas `.gss` ao lado dos exemplos.
    let mut folhas = Vec::new();
    gsss(&raiz.join("examples"), &mut folhas);
    gsss(&raiz.join("crates/glacier-cli/templates"), &mut folhas);
    folhas.sort();
    for folha in &folhas {
        let src = std::fs::read_to_string(folha).expect("ler .gss");
        let relativo = folha.strip_prefix(raiz).unwrap_or(folha).display();
        for regra in regras_gss(&src) {
            if regra.contains("background")
                && regra.contains("width: fill")
                && regra.contains("height: fill")
            {
                culpados.push(format!("{relativo}: classe GSS com background e fill/fill"));
            }
        }
    }

    assert!(
        culpados.is_empty(),
        "estes pintam a janela inteira — o fundo é do tema (`<link rel=\"theme\">`), \
         não de um `background` na raiz:\n  {}",
        culpados.join("\n  ")
    );
    assert!(
        arquivos.len() >= 45,
        "só {} .gv varridos — o caminho está errado?",
        arquivos.len()
    );
    assert!(
        folhas.len() >= 6,
        "só {} .gss varridos — o caminho está errado?",
        folhas.len()
    );
}

/// Os `.gss` de um diretório, recursivamente. Gêmeo de [`gvs`].
fn gsss(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            gsss(&p, out);
        } else if p.extension().is_some_and(|e| e == "gss") {
            out.push(p);
        }
    }
}

/// O texto de cada tag de abertura (`<container padding="20" …`), sem os `<`/`>`
/// e sem comentários. Varredura por texto, como o resto deste arquivo.
fn tags_de(src: &str) -> Vec<String> {
    let sem = sem_comentarios(src);
    let mut out = Vec::new();
    let mut resto = sem.as_str();
    while let Some(i) = resto.find('<') {
        resto = &resto[i + 1..];
        let Some(f) = resto.find('>') else { break };
        let tag = &resto[..f];
        if !tag.starts_with('/') && !tag.starts_with('!') {
            out.push(tag.to_string());
        }
        resto = &resto[f + 1..];
    }
    out
}

/// O corpo de cada bloco `{ … }` de GSS encontrado no texto — de um `<style>`
/// inline ou de uma folha inteira. Normaliza o espaço para os testes de
/// `contains` não dependerem de formatação.
fn regras_gss(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut resto = src;
    while let Some(a) = resto.find('{') {
        let Some(b) = resto[a..].find('}') else { break };
        let corpo: String = resto[a + 1..a + b]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(':', ": ")
            .replace("  ", " ");
        out.push(corpo);
        resto = &resto[a + b + 1..];
    }
    out
}
