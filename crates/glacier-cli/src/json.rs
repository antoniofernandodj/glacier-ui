//! Um parser de JSON de verdade, em `std` puro (a CLI não tem dependências).
//!
//! Aceita **JSONC** — o dialeto do `tsconfig.json` e do `settings.json` do VS
//! Code — e nada além disso:
//!
//! - `// linha` e `/* bloco */` onde caberia um espaço;
//! - vírgula final em objeto e array (`[1, 2,]`).
//!
//! Chaves sem aspas, aspas simples, hexadecimal e `Infinity` (o resto do JSON5)
//! ficam de fora: custam parser e não ajudam a documentar um campo.
//!
//! Dois usos: o `glacier.jsonc` do projeto ([`crate::config`]) e o
//! `package.json` das extensões ([`crate::vsix`]). O segundo já foi uma
//! varredura de bytes atrás de uma chave, que lia um `"name"` escrito dentro de
//! um comentário ou de uma string como se fosse o campo; aqui o texto é
//! percorrido uma vez, com a estrutura, e a pergunta "onde está a chave" deixa
//! de existir.

use std::fmt;

/// Aninhamento máximo. Um `[[[[…` de milhares de níveis estouraria a pilha
/// antes de virar erro; 64 é muito além de qualquer arquivo de configuração.
const PROFUNDIDADE_MAXIMA: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    Nulo,
    Bool(bool),
    Num(f64),
    Str(String),
    Lista(Vec<Valor>),
    /// Pares na ordem em que foram escritos (sem chave repetida: ver [`parse`]).
    Obj(Vec<(String, Valor)>),
}

impl Valor {
    /// O valor da chave `chave`, se `self` for um objeto que a tenha.
    pub fn get(&self, chave: &str) -> Option<&Valor> {
        match self {
            Valor::Obj(pares) => pares.iter().find(|(k, _)| k == chave).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Desce por `caminho` (`["engines", "vscode"]`).
    pub fn caminho(&self, caminho: &[&str]) -> Option<&Valor> {
        caminho.iter().try_fold(self, |v, chave| v.get(chave))
    }

    pub fn como_str(&self) -> Option<&str> {
        match self {
            Valor::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn como_bool(&self) -> Option<bool> {
        match self {
            Valor::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn como_num(&self) -> Option<f64> {
        match self {
            Valor::Num(n) => Some(*n),
            _ => None,
        }
    }

    pub fn como_lista(&self) -> Option<&[Valor]> {
        match self {
            Valor::Lista(v) => Some(v),
            _ => None,
        }
    }

    /// Nome do tipo, para mensagem de erro ("esperava texto, achei número").
    pub fn tipo(&self) -> &'static str {
        match self {
            Valor::Nulo => "null",
            Valor::Bool(_) => "booleano",
            Valor::Num(_) => "número",
            Valor::Str(_) => "texto",
            Valor::Lista(_) => "lista",
            Valor::Obj(_) => "objeto",
        }
    }
}

/// Onde e por que o texto não é JSON. `linha` e `coluna` contam de 1, e a
/// coluna em caracteres (não bytes), como o editor mostra.
#[derive(Debug, Clone, PartialEq)]
pub struct ErroJson {
    pub linha: usize,
    pub coluna: usize,
    pub mensagem: String,
}

impl fmt::Display for ErroJson {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.linha, self.coluna, self.mensagem)
    }
}

impl std::error::Error for ErroJson {}

/// Lê um documento JSONC inteiro. Texto sobrando depois do valor é erro, e uma
/// **chave repetida num objeto também**: num arquivo de configuração, a segunda
/// valer em silêncio é exatamente o tipo de coisa que esconde um engano.
pub fn parse(texto: &str) -> Result<Valor, ErroJson> {
    let texto = texto.strip_prefix('\u{feff}').unwrap_or(texto);
    let mut p = Parser {
        bytes: texto.as_bytes(),
        texto,
        i: 0,
    };
    p.espacos()?;
    let valor = p.valor(0)?;
    p.espacos()?;
    if p.i < p.bytes.len() {
        return Err(p.erro("texto depois do fim do documento"));
    }
    Ok(valor)
}

struct Parser<'a> {
    bytes: &'a [u8],
    texto: &'a str,
    i: usize,
}

impl Parser<'_> {
    /// Erro na posição atual.
    fn erro(&self, mensagem: &str) -> ErroJson {
        self.erro_em(self.i, mensagem)
    }

    fn erro_em(&self, pos: usize, mensagem: &str) -> ErroJson {
        let pos = pos.min(self.bytes.len());
        // `pos` cai sempre numa fronteira de caractere (só avançamos byte a byte
        // por ASCII, ou por caracteres inteiros dentro de string), mas o `get`
        // evita um pânico se um dia deixar de cair.
        let antes = self.texto.get(..pos).unwrap_or("");
        let linha = antes.matches('\n').count() + 1;
        let coluna = antes.rsplit('\n').next().map_or(0, |l| l.chars().count()) + 1;
        ErroJson {
            linha,
            coluna,
            mensagem: mensagem.to_string(),
        }
    }

    fn olhar(&self) -> Option<u8> {
        self.bytes.get(self.i).copied()
    }

    /// Espaço em branco e comentários.
    fn espacos(&mut self) -> Result<(), ErroJson> {
        loop {
            match self.olhar() {
                Some(b' ' | b'\t' | b'\n' | b'\r') => self.i += 1,
                Some(b'/') => match self.bytes.get(self.i + 1) {
                    Some(b'/') => {
                        while self.olhar().is_some_and(|b| b != b'\n') {
                            self.i += 1;
                        }
                    }
                    Some(b'*') => {
                        let inicio = self.i;
                        self.i += 2;
                        loop {
                            match self.olhar() {
                                Some(b'*') if self.bytes.get(self.i + 1) == Some(&b'/') => {
                                    self.i += 2;
                                    break;
                                }
                                Some(_) => self.i += 1,
                                None => {
                                    return Err(
                                        self.erro_em(inicio, "comentário `/*` nunca fechado")
                                    );
                                }
                            }
                        }
                    }
                    _ => return Err(self.erro("`/` solto (comentário é `//` ou `/* */`)")),
                },
                _ => return Ok(()),
            }
        }
    }

    fn valor(&mut self, profundidade: usize) -> Result<Valor, ErroJson> {
        if profundidade > PROFUNDIDADE_MAXIMA {
            return Err(self.erro("aninhamento fundo demais"));
        }
        match self.olhar() {
            Some(b'{') => self.objeto(profundidade),
            Some(b'[') => self.lista(profundidade),
            Some(b'"') => self.string().map(Valor::Str),
            Some(b't') => self.palavra("true", Valor::Bool(true)),
            Some(b'f') => self.palavra("false", Valor::Bool(false)),
            Some(b'n') => self.palavra("null", Valor::Nulo),
            Some(b'-' | b'0'..=b'9') => self.numero(),
            Some(_) => Err(self.erro("esperava um valor")),
            None => Err(self.erro("o documento acabou onde se esperava um valor")),
        }
    }

    fn palavra(&mut self, palavra: &str, valor: Valor) -> Result<Valor, ErroJson> {
        if self.bytes[self.i..].starts_with(palavra.as_bytes()) {
            self.i += palavra.len();
            Ok(valor)
        } else {
            Err(self.erro(&format!("esperava `{palavra}`")))
        }
    }

    fn numero(&mut self) -> Result<Valor, ErroJson> {
        let inicio = self.i;
        while self
            .olhar()
            .is_some_and(|b| matches!(b, b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9'))
        {
            self.i += 1;
        }
        let trecho = &self.texto[inicio..self.i];
        // `f64::from_str` aceita `1.`, `.5` e `+1`; JSON não. Barra o que o
        // Rust perdoa para que o arquivo continue sendo JSON de verdade.
        let digitos = trecho.strip_prefix('-').unwrap_or(trecho);
        let valido = !digitos.is_empty()
            && !digitos.starts_with('+')
            && !digitos.starts_with('.')
            && !digitos.contains(".e")
            && !digitos.contains(".E")
            && !digitos.ends_with('.')
            && (digitos == "0" || !digitos.starts_with('0') || digitos[1..].starts_with(['.', 'e', 'E']));
        match trecho.parse::<f64>() {
            Ok(n) if valido && n.is_finite() => Ok(Valor::Num(n)),
            _ => Err(self.erro_em(inicio, &format!("número inválido: `{trecho}`"))),
        }
    }

    fn string(&mut self) -> Result<String, ErroJson> {
        let abertura = self.i;
        self.i += 1; // a aspa
        let mut saida = String::new();
        loop {
            let Some(b) = self.olhar() else {
                return Err(self.erro_em(abertura, "texto nunca fechado (falta a aspa final)"));
            };
            match b {
                b'"' => {
                    self.i += 1;
                    return Ok(saida);
                }
                b'\n' => {
                    return Err(self.erro_em(abertura, "texto com quebra de linha sem fechar"));
                }
                b'\\' => {
                    let Some(&c) = self.bytes.get(self.i + 1) else {
                        return Err(self.erro_em(abertura, "texto nunca fechado"));
                    };
                    self.i += 2;
                    match c {
                        b'"' => saida.push('"'),
                        b'\\' => saida.push('\\'),
                        b'/' => saida.push('/'),
                        b'b' => saida.push('\u{8}'),
                        b'f' => saida.push('\u{c}'),
                        b'n' => saida.push('\n'),
                        b'r' => saida.push('\r'),
                        b't' => saida.push('\t'),
                        b'u' => saida.push(self.escape_unicode()?),
                        _ => return Err(self.erro_em(self.i - 2, "escape desconhecido")),
                    }
                }
                _ => {
                    // Um caractere inteiro (pode ter vários bytes em UTF-8).
                    let c = self.texto[self.i..]
                        .chars()
                        .next()
                        .expect("self.i está numa fronteira de caractere");
                    saida.push(c);
                    self.i += c.len_utf8();
                }
            }
        }
    }

    /// `\uXXXX`, com o par substituto (`😀`) montado num só caractere.
    fn escape_unicode(&mut self) -> Result<char, ErroJson> {
        let alto = self.quatro_hex()?;
        let ponto = if (0xD800..0xDC00).contains(&alto) {
            if self.bytes[self.i..].starts_with(b"\\u") {
                self.i += 2;
                let baixo = self.quatro_hex()?;
                if !(0xDC00..0xE000).contains(&baixo) {
                    return Err(self.erro("par substituto UTF-16 inválido"));
                }
                0x10000 + ((alto - 0xD800) << 10) + (baixo - 0xDC00)
            } else {
                return Err(self.erro("par substituto UTF-16 incompleto"));
            }
        } else {
            alto
        };
        char::from_u32(ponto).ok_or_else(|| self.erro("`\\u` não é um caractere Unicode válido"))
    }

    fn quatro_hex(&mut self) -> Result<u32, ErroJson> {
        let fim = self.i + 4;
        let trecho = self
            .texto
            .get(self.i..fim)
            .filter(|t| t.bytes().all(|b| b.is_ascii_hexdigit()))
            .ok_or_else(|| self.erro("`\\u` precisa de quatro dígitos hexadecimais"))?;
        self.i = fim;
        Ok(u32::from_str_radix(trecho, 16).expect("validado como hexadecimal"))
    }

    fn lista(&mut self, profundidade: usize) -> Result<Valor, ErroJson> {
        self.i += 1; // `[`
        let mut itens = Vec::new();
        loop {
            self.espacos()?;
            match self.olhar() {
                Some(b']') => {
                    self.i += 1;
                    return Ok(Valor::Lista(itens));
                }
                None => return Err(self.erro("lista nunca fechada (falta `]`)")),
                _ => {}
            }
            itens.push(self.valor(profundidade + 1)?);
            self.espacos()?;
            match self.olhar() {
                Some(b',') => self.i += 1,
                Some(b']') => {}
                _ => return Err(self.erro("esperava `,` ou `]`")),
            }
        }
    }

    fn objeto(&mut self, profundidade: usize) -> Result<Valor, ErroJson> {
        self.i += 1; // `{`
        let mut pares: Vec<(String, Valor)> = Vec::new();
        loop {
            self.espacos()?;
            match self.olhar() {
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Valor::Obj(pares));
                }
                Some(b'"') => {}
                None => return Err(self.erro("objeto nunca fechado (falta `}`)")),
                Some(_) => return Err(self.erro("esperava o nome de um campo entre aspas")),
            }
            let pos_chave = self.i;
            let chave = self.string()?;
            if pares.iter().any(|(k, _)| *k == chave) {
                return Err(self.erro_em(pos_chave, &format!("campo `{chave}` repetido")));
            }
            self.espacos()?;
            if self.olhar() != Some(b':') {
                return Err(self.erro("esperava `:` depois do nome do campo"));
            }
            self.i += 1;
            self.espacos()?;
            pares.push((chave, self.valor(profundidade + 1)?));
            self.espacos()?;
            match self.olhar() {
                Some(b',') => self.i += 1,
                Some(b'}') => {}
                _ => return Err(self.erro("esperava `,` ou `}`")),
            }
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn ok(texto: &str) -> Valor {
        parse(texto).unwrap_or_else(|e| panic!("{texto:?} deveria ser válido: {e}"))
    }

    fn erro(texto: &str) -> ErroJson {
        parse(texto).expect_err(&format!("{texto:?} deveria ser recusado"))
    }

    #[test]
    fn le_os_tipos_basicos() {
        let v = ok(r#"{"a": 1, "b": [true, false, null], "c": {"d": "x"}, "e": -2.5e1}"#);
        assert_eq!(v.get("a").and_then(Valor::como_num), Some(1.0));
        assert_eq!(v.get("e").and_then(Valor::como_num), Some(-25.0));
        assert_eq!(
            v.get("b"),
            Some(&Valor::Lista(vec![
                Valor::Bool(true),
                Valor::Bool(false),
                Valor::Nulo
            ]))
        );
        assert_eq!(
            v.caminho(&["c", "d"]).and_then(Valor::como_str),
            Some("x")
        );
        assert_eq!(v.caminho(&["c", "zzz"]), None);
    }

    #[test]
    fn comentarios_de_linha_e_de_bloco_sao_ignorados() {
        let v = ok("// topo\n{ /* a */ \"a\": 1, // depois\n  /* b\n multilinha */ \"b\": 2 }\n// fim");
        assert_eq!(v.get("a").and_then(Valor::como_num), Some(1.0));
        assert_eq!(v.get("b").and_then(Valor::como_num), Some(2.0));
    }

    /// O bug da varredura antiga: um campo escrito dentro de um comentário (ou
    /// dentro de uma string) valia como se fosse o campo de verdade.
    #[test]
    fn campo_dentro_de_comentario_ou_de_texto_nao_conta() {
        let v = ok(
            r#"{
              // "version": "9.9.9"
              /* "version": "8.8.8" */
              "description": "tem \"version\": \"7.7.7\" dentro",
              "version": "1.0.0"
            }"#,
        );
        assert_eq!(v.get("version").and_then(Valor::como_str), Some("1.0.0"));
        assert!(
            v.get("description")
                .and_then(Valor::como_str)
                .unwrap()
                .contains("7.7.7")
        );
    }

    #[test]
    fn barras_dentro_de_texto_nao_sao_comentario() {
        let v = ok(r#"{"url": "https://exemplo.com/a//b", "g": "/* nao */"}"#);
        assert_eq!(
            v.get("url").and_then(Valor::como_str),
            Some("https://exemplo.com/a//b")
        );
        assert_eq!(v.get("g").and_then(Valor::como_str), Some("/* nao */"));
    }

    #[test]
    fn virgula_final_e_aceita() {
        let v = ok(r#"{"a": [1, 2,], "b": 3,}"#);
        assert_eq!(v.get("a").and_then(Valor::como_lista).map(<[_]>::len), Some(2));
        assert!(erro("[,]").mensagem.contains("esperava um valor"));
        assert!(parse("[1,,2]").is_err());
        assert!(parse(r#"{,}"#).is_err());
    }

    #[test]
    fn escapes_e_unicode() {
        let v = ok(r#""a\nb\t\"c\\ é 😀 ção""#);
        assert_eq!(v.como_str(), Some("a\nb\t\"c\\ é 😀 ção"));
        assert!(parse(r#""\ud83d""#).is_err(), "substituto sem par");
        assert!(parse(r#""\x""#).is_err());
        assert!(parse(r#""\u12""#).is_err());
    }

    #[test]
    fn numeros_que_o_rust_perdoa_mas_o_json_nao() {
        for ruim in ["01", "+1", ".5", "1.", "1.e2", "-", "--1", "1e", "1e999"] {
            assert!(parse(ruim).is_err(), "{ruim:?} não é um número JSON");
        }
        for bom in ["0", "-0", "10", "0.5", "1e3", "1E-3", "-12.50"] {
            assert!(parse(bom).is_ok(), "{bom:?} é um número JSON");
        }
    }

    #[test]
    fn chave_repetida_e_erro() {
        let e = erro(r#"{"a": 1, "a": 2}"#);
        assert!(e.mensagem.contains("`a` repetido"), "{e}");
    }

    #[test]
    fn erro_traz_linha_e_coluna() {
        let e = erro("{\n  \"a\": 1,\n  \"b\": ?\n}");
        assert_eq!((e.linha, e.coluna), (3, 8), "{e}");
        // A coluna conta caracteres: o `é` ocupa dois bytes.
        let e = erro("{\"é\": ?}");
        assert_eq!((e.linha, e.coluna), (1, 7), "{e}");
    }

    #[test]
    fn estruturas_mal_fechadas_apontam_o_que_falta() {
        assert!(erro("{\"a\": 1").mensagem.contains('}'));
        assert!(erro("[1, 2").mensagem.contains(']'));
        assert!(erro("\"abc").mensagem.contains("fechado"));
        assert!(erro("/* sem fim").mensagem.contains("nunca fechado"));
        assert!(erro("{\"a\" 1}").mensagem.contains(':'));
        assert!(erro("{} lixo").mensagem.contains("depois do fim"));
        assert!(erro("/ x").mensagem.contains("solto"));
        assert!(parse("").is_err());
    }

    #[test]
    fn aninhamento_fundo_demais_vira_erro_e_nao_estouro_de_pilha() {
        let fundo = "[".repeat(10_000);
        assert!(erro(&fundo).mensagem.contains("fundo demais"));
    }

    #[test]
    fn bom_no_inicio_e_ignorado() {
        assert!(parse("\u{feff}{\"a\": 1}").is_ok());
    }
}
