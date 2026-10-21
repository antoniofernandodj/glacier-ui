//! `glacier.jsonc`: os padrões do `glacier serve` e do `glacier test` por
//! projeto, no lugar de repetir `-F web-gpu --port 3000 --watch` a cada
//! chamada.
//!
//! É JSONC (JSON com comentários e vírgula final, ver [`crate::json`]): a
//! extensão `.jsonc` faz o editor aceitar os comentários sem sublinhar erro, e
//! o arquivo gerado por `glacier new` documenta cada campo no próprio campo.
//!
//! ## Precedência
//!
//! flag da linha de comando › configuração escolhida (`-c`) › base do arquivo ›
//! padrão da CLI. Sem o arquivo, nada muda: ele é opcional.
//!
//! ## Campo desconhecido é erro
//!
//! O arquivo é validado inteiro contra [`RAIZ`] antes de qualquer valor ser
//! lido. Um `"prot": 3000` ignorado em silêncio é uma porta que não mudou e
//! ninguém sabe por quê — a mesma família de bug que o `CLAUDE.md` do motor
//! descreve para o `.gss`. Todos os problemas saem de uma vez, não um por
//! execução.

use std::io;
use std::path::Path;

use crate::json::{self, Valor};

pub const NOME: &str = "glacier.jsonc";

/// O tipo esperado de um campo.
enum Campo {
    Bool,
    Texto,
    /// Inteiro de 1 a 65535.
    Porta,
    /// Lista de textos.
    Lista,
    Obj(&'static [(&'static str, Campo)]),
    /// Objeto cujas chaves são livres (nomes de configuração) e cujos valores
    /// seguem todos o mesmo esquema.
    Mapa(&'static [(&'static str, Campo)]),
}

const SERVE_WASM: &[(&str, Campo)] = &[
    ("port", Campo::Porta),
    ("release", Campo::Bool),
    ("watch", Campo::Bool),
    ("features", Campo::Lista),
];

const SERVE_DESKTOP: &[(&str, Campo)] = &[
    ("release", Campo::Bool),
    ("features", Campo::Lista),
    ("args", Campo::Lista),
];

const SERVE: &[(&str, Campo)] = &[
    ("wasm", Campo::Obj(SERVE_WASM)),
    ("desktop", Campo::Obj(SERVE_DESKTOP)),
];

const TEST: &[(&str, Campo)] = &[
    ("release", Campo::Bool),
    ("features", Campo::Lista),
    ("luau_path", Campo::Lista),
];

/// O que uma configuração nomeada pode sobrescrever: tudo, menos
/// `configurations` e `default_configuration` (uma configuração dentro de outra
/// não teria o que dizer).
const CORPO: &[(&str, Campo)] = &[
    ("features", Campo::Lista),
    ("serve", Campo::Obj(SERVE)),
    ("test", Campo::Obj(TEST)),
];

const RAIZ: &[(&str, Campo)] = &[
    ("features", Campo::Lista),
    ("serve", Campo::Obj(SERVE)),
    ("test", Campo::Obj(TEST)),
    ("default_configuration", Campo::Texto),
    ("configurations", Campo::Mapa(CORPO)),
];

/// O `glacier.jsonc` já validado e com a configuração escolhida aplicada por
/// cima da base. Sem arquivo, é vazio e todo getter devolve `None`.
pub struct Config {
    efetiva: Valor,
    /// Resumo para a CLI mostrar ("glacier.jsonc (dev)"); `None` = sem arquivo.
    origem: Option<String>,
}

impl Config {
    pub fn vazia() -> Self {
        Self {
            efetiva: Valor::Obj(Vec::new()),
            origem: None,
        }
    }

    /// Lê `raiz/glacier.jsonc`. `escolhida` é o `-c nome` da linha de comando;
    /// sem ele vale o `default_configuration` do arquivo, se houver.
    pub fn carregar(raiz: &Path, escolhida: Option<&str>) -> io::Result<Self> {
        let caminho = raiz.join(NOME);
        if !caminho.is_file() {
            return match escolhida {
                Some(nome) => Err(invalido(format!(
                    "`-c {nome}`: não há {NOME} em {} — é lá que as configurações são declaradas",
                    raiz.display()
                ))),
                None => Ok(Self::vazia()),
            };
        }
        let texto = std::fs::read_to_string(&caminho)
            .map_err(|e| io::Error::new(e.kind(), format!("lendo {}: {e}", caminho.display())))?;
        Self::de_texto(&texto, escolhida)
    }

    fn de_texto(texto: &str, escolhida: Option<&str>) -> io::Result<Self> {
        let raiz = json::parse(texto).map_err(|e| invalido(format!("{NOME}:{e}")))?;

        let mut problemas = Vec::new();
        if !matches!(raiz, Valor::Obj(_)) {
            problemas.push(format!("a raiz deve ser um objeto, não {}", raiz.tipo()));
        } else {
            validar(&raiz, RAIZ, "", &mut problemas);
        }
        if !problemas.is_empty() {
            return Err(invalido(format!(
                "{NOME} inválido:\n  - {}",
                problemas.join("\n  - ")
            )));
        }

        let nome = escolhida.map(str::to_owned).or_else(|| {
            raiz.get("default_configuration")
                .and_then(Valor::como_str)
                .map(str::to_owned)
        });

        let Some(nome) = nome else {
            return Ok(Self {
                efetiva: raiz,
                origem: Some(NOME.to_string()),
            });
        };

        let Some(sobreposta) = raiz.caminho(&["configurations", &nome]) else {
            let existentes = match raiz.get("configurations") {
                Some(Valor::Obj(pares)) => pares
                    .iter()
                    .map(|(k, _)| k.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            let de_onde = if escolhida.is_some() {
                "pedida com `-c`"
            } else {
                "do `default_configuration`"
            };
            return Err(invalido(format!(
                "a configuração '{nome}' ({de_onde}) não existe em {NOME} (existentes: {})",
                if existentes.is_empty() {
                    "nenhuma"
                } else {
                    &existentes
                }
            )));
        };

        Ok(Self {
            efetiva: mesclar(raiz.clone(), sobreposta.clone()),
            origem: Some(format!("{NOME} ({nome})")),
        })
    }

    pub fn origem(&self) -> Option<&str> {
        self.origem.as_deref()
    }

    // Os getters não precisam checar tipo: `de_texto` já recusou qualquer
    // valor que não seja o do esquema.

    pub fn booleano(&self, caminho: &[&str]) -> Option<bool> {
        self.efetiva.caminho(caminho)?.como_bool()
    }

    pub fn porta(&self, caminho: &[&str]) -> Option<u16> {
        self.efetiva.caminho(caminho)?.como_num().map(|n| n as u16)
    }

    pub fn lista(&self, caminho: &[&str]) -> Option<Vec<String>> {
        let itens = self.efetiva.caminho(caminho)?.como_lista()?;
        Some(
            itens
                .iter()
                .filter_map(|v| v.como_str().map(str::to_owned))
                .collect(),
        )
    }

    /// `features` de uma seção, com a lista de topo como reserva, já na forma
    /// que o `cargo --features` aceita. Lista vazia = sem features.
    pub fn features(&self, secao: &[&str]) -> Option<String> {
        let mut caminho = secao.to_vec();
        caminho.push("features");
        let lista = self.lista(&caminho).or_else(|| self.lista(&["features"]))?;
        (!lista.is_empty()).then(|| lista.join(","))
    }
}

/// Objetos se fundem campo a campo; qualquer outra coisa (inclusive lista) é
/// trocada pela de cima. Uma configuração que diz `"features": []` quer ZERAR
/// as da base, não somar nada a elas.
fn mesclar(base: Valor, cima: Valor) -> Valor {
    match (base, cima) {
        (Valor::Obj(mut pares), Valor::Obj(novos)) => {
            for (chave, valor) in novos {
                match pares.iter_mut().find(|(k, _)| *k == chave) {
                    Some((_, atual)) => {
                        let antigo = std::mem::replace(atual, Valor::Nulo);
                        *atual = mesclar(antigo, valor);
                    }
                    None => pares.push((chave, valor)),
                }
            }
            Valor::Obj(pares)
        }
        (_, cima) => cima,
    }
}

fn validar(valor: &Valor, esquema: &[(&str, Campo)], prefixo: &str, problemas: &mut Vec<String>) {
    let Valor::Obj(pares) = valor else {
        return;
    };
    for (chave, v) in pares {
        let caminho = if prefixo.is_empty() {
            chave.clone()
        } else {
            format!("{prefixo}.{chave}")
        };
        let Some((_, campo)) = esquema.iter().find(|(nome, _)| nome == chave) else {
            let conhecidas: Vec<&str> = esquema.iter().map(|(n, _)| *n).collect();
            problemas.push(format!(
                "campo desconhecido '{caminho}' (os desta seção: {})",
                conhecidas.join(", ")
            ));
            continue;
        };
        match campo {
            Campo::Bool => {
                if v.como_bool().is_none() {
                    problemas.push(esperava(&caminho, "booleano (true/false)", v));
                }
            }
            Campo::Texto => {
                if v.como_str().is_none() {
                    problemas.push(esperava(&caminho, "texto", v));
                }
            }
            Campo::Porta => match v.como_num() {
                Some(n) if n.fract() == 0.0 && (1.0..=65535.0).contains(&n) => {}
                Some(_) => problemas.push(format!(
                    "'{caminho}': a porta deve ser um inteiro de 1 a 65535"
                )),
                None => problemas.push(esperava(&caminho, "número (a porta)", v)),
            },
            Campo::Lista => match v.como_lista() {
                Some(itens) => {
                    for (i, item) in itens.iter().enumerate() {
                        if item.como_str().is_none() {
                            problemas.push(esperava(&format!("{caminho}[{i}]"), "texto", item));
                        }
                    }
                }
                None => problemas.push(esperava(&caminho, "lista de textos", v)),
            },
            Campo::Obj(filhos) => match v {
                Valor::Obj(_) => validar(v, filhos, &caminho, problemas),
                _ => problemas.push(esperava(&caminho, "objeto", v)),
            },
            Campo::Mapa(filhos) => match v {
                Valor::Obj(entradas) => {
                    for (nome, conteudo) in entradas {
                        let sub = format!("{caminho}.{nome}");
                        match conteudo {
                            Valor::Obj(_) => validar(conteudo, filhos, &sub, problemas),
                            outro => problemas.push(esperava(&sub, "objeto", outro)),
                        }
                    }
                }
                _ => problemas.push(esperava(&caminho, "objeto", v)),
            },
        }
    }
}

fn esperava(caminho: &str, tipo: &str, achou: &Valor) -> String {
    format!("'{caminho}': esperava {tipo}, achei {}", achou.tipo())
}

fn invalido(mensagem: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, mensagem)
}

/// Tira `-c <nome>` / `--configuration <nome>` / `--configuration=<nome>` dos
/// argumentos e devolve o nome e o resto. Para de procurar no `--`: o que vem
/// depois é do app (`serve desktop -- -c x`), não da CLI.
pub fn extrair_configuracao(argv: Vec<String>) -> io::Result<(Option<String>, Vec<String>)> {
    let mut nome = None;
    let mut resto = Vec::with_capacity(argv.len());
    let mut it = argv.into_iter();
    while let Some(arg) = it.next() {
        if arg == "--" {
            resto.push(arg);
            resto.extend(it.by_ref());
            break;
        }
        let valor = if arg == "-c" || arg == "--configuration" {
            Some(
                it.next()
                    .ok_or_else(|| invalido(format!("`{arg}` precisa do nome de uma configuração")))?,
            )
        } else {
            arg.strip_prefix("--configuration=").map(str::to_owned)
        };
        match valor {
            Some(v) => nome = Some(v),
            None => resto.push(arg),
        }
    }
    Ok((nome, resto))
}

#[cfg(test)]
mod testes {
    use super::*;

    fn cfg(texto: &str, escolhida: Option<&str>) -> Config {
        Config::de_texto(texto, escolhida).unwrap_or_else(|e| panic!("{e}"))
    }

    fn erro(texto: &str, escolhida: Option<&str>) -> String {
        match Config::de_texto(texto, escolhida) {
            Ok(_) => panic!("deveria ser recusado: {texto}"),
            Err(e) => e.to_string(),
        }
    }

    const BASE: &str = r#"{
      // comentário no meio
      "features": ["web-gpu"],
      "serve": { "wasm": { "port": 3000, "watch": true } },
      "test": { "luau_path": ["lib"], },
      "default_configuration": "dev",
      "configurations": {
        "dev": { "serve": { "wasm": { "port": 4000 } } },
        "prod": { "features": [], "serve": { "wasm": { "watch": false } } },
      },
    }"#;

    #[test]
    fn sem_arquivo_tudo_e_none() {
        let c = Config::vazia();
        assert_eq!(c.porta(&["serve", "wasm", "port"]), None);
        assert_eq!(c.features(&["test"]), None);
        assert!(c.origem().is_none());
    }

    #[test]
    fn a_base_vale_quando_nao_ha_configuracao() {
        let c = cfg(
            r#"{ "serve": { "wasm": { "port": 3000 } } }"#,
            None,
        );
        assert_eq!(c.porta(&["serve", "wasm", "port"]), Some(3000));
        assert_eq!(c.origem(), Some("glacier.jsonc"));
    }

    #[test]
    fn default_configuration_e_aplicada_por_cima_da_base() {
        let c = cfg(BASE, None);
        assert_eq!(c.porta(&["serve", "wasm", "port"]), Some(4000));
        // O que a configuração não diz vem da base.
        assert_eq!(c.booleano(&["serve", "wasm", "watch"]), Some(true));
        assert_eq!(c.origem(), Some("glacier.jsonc (dev)"));
    }

    #[test]
    fn menos_c_vence_o_default_configuration() {
        let c = cfg(BASE, Some("prod"));
        assert_eq!(c.porta(&["serve", "wasm", "port"]), Some(3000));
        assert_eq!(c.booleano(&["serve", "wasm", "watch"]), Some(false));
    }

    /// Lista da configuração TROCA a da base; não soma. `[]` é como se zera.
    #[test]
    fn lista_vazia_numa_configuracao_zera_as_features_da_base() {
        assert_eq!(cfg(BASE, Some("prod")).features(&["serve", "wasm"]), None);
        assert_eq!(
            cfg(BASE, None).features(&["serve", "wasm"]).as_deref(),
            Some("web-gpu")
        );
    }

    #[test]
    fn features_da_secao_vencem_as_de_topo() {
        let c = cfg(
            r#"{ "features": ["a"], "test": { "features": ["b", "c"] } }"#,
            None,
        );
        assert_eq!(c.features(&["test"]).as_deref(), Some("b,c"));
        assert_eq!(c.features(&["serve", "desktop"]).as_deref(), Some("a"));
    }

    #[test]
    fn configuracao_inexistente_lista_as_que_existem() {
        let e = erro(BASE, Some("staging"));
        assert!(e.contains("'staging'") && e.contains("dev, prod"), "{e}");
        // E a do `default_configuration` também é checada.
        let e = erro(
            r#"{ "default_configuration": "x", "configurations": {} }"#,
            None,
        );
        assert!(e.contains("default_configuration") && e.contains("nenhuma"), "{e}");
    }

    #[test]
    fn campo_desconhecido_e_erro_e_diz_quais_existem() {
        let e = erro(r#"{ "serve": { "wasm": { "prot": 3000 } } }"#, None);
        assert!(e.contains("serve.wasm.prot"), "{e}");
        assert!(e.contains("port, release, watch, features"), "{e}");
    }

    #[test]
    fn tipo_errado_e_erro_e_todos_saem_de_uma_vez() {
        let e = erro(
            r#"{
              "serve": { "wasm": { "port": "3000", "watch": "sim" } },
              "test": { "luau_path": ["ok", 1] }
            }"#,
            None,
        );
        assert!(e.contains("serve.wasm.port"), "{e}");
        assert!(e.contains("serve.wasm.watch"), "{e}");
        assert!(e.contains("test.luau_path[1]"), "{e}");
    }

    #[test]
    fn porta_fora_da_faixa_e_recusada() {
        for ruim in ["0", "65536", "80.5", "-1"] {
            let e = erro(&format!(r#"{{ "serve": {{ "wasm": {{ "port": {ruim} }} }} }}"#), None);
            assert!(e.contains("1 a 65535"), "{ruim}: {e}");
        }
    }

    #[test]
    fn campo_desconhecido_dentro_de_uma_configuracao_tambem_e_erro() {
        let e = erro(
            r#"{ "configurations": { "dev": { "default_configuration": "x" } } }"#,
            None,
        );
        assert!(e.contains("configurations.dev.default_configuration"), "{e}");
    }

    #[test]
    fn json_malformado_aponta_linha_e_coluna() {
        let e = erro("{\n  \"serve\": ,\n}", None);
        assert!(e.starts_with("glacier.jsonc:2:"), "{e}");
    }

    #[test]
    fn raiz_que_nao_e_objeto_e_recusada() {
        assert!(erro("[1]", None).contains("raiz"));
    }

    #[test]
    fn extrai_a_configuracao_em_qualquer_posicao_e_respeita_o_duplo_traco() {
        let v = |xs: &[&str]| xs.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        let (nome, resto) = extrair_configuracao(v(&["--watch", "-c", "dev", "-F", "x"])).unwrap();
        assert_eq!(nome.as_deref(), Some("dev"));
        assert_eq!(resto, v(&["--watch", "-F", "x"]));

        let (nome, _) = extrair_configuracao(v(&["--configuration=prod"])).unwrap();
        assert_eq!(nome.as_deref(), Some("prod"));

        let (nome, resto) = extrair_configuracao(v(&["--release", "--", "-c", "do-app"])).unwrap();
        assert_eq!(nome, None);
        assert_eq!(resto, v(&["--release", "--", "-c", "do-app"]));

        assert!(extrair_configuracao(v(&["-c"])).is_err());
    }

    #[test]
    fn carregar_de_disco_sem_arquivo_e_vazio_mas_c_exige_o_arquivo() {
        let dir = std::env::temp_dir().join(format!("glacier-config-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert!(Config::carregar(&dir, None).unwrap().origem().is_none());
        let e = Config::carregar(&dir, Some("dev")).err().expect("deveria falhar");
        assert!(e.to_string().contains("glacier.jsonc"), "{e}");

        std::fs::write(dir.join(NOME), r#"{ "test": { "release": true } }"#).unwrap();
        let c = Config::carregar(&dir, None).unwrap();
        assert_eq!(c.booleano(&["test", "release"]), Some(true));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
