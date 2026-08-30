//! Empacota uma extensão de VS Code num `.vsix`, sem `vsce` e sem Node.
//!
//! Um `.vsix` é um zip com três coisas: os arquivos da extensão sob
//! `extension/`, um `extension.vsixmanifest` (derivado do `package.json`) e um
//! `[Content_Types].xml`. É o formato que `code --install-extension` aceita —
//! apontar o CLI para uma pasta não funciona.
//!
//! O zip é escrito com o método *stored* (sem compressão): a alternativa seria
//! carregar um deflate, e o ganho não paga a dependência para ~40 KB de JSON e
//! JavaScript que o editor descompacta uma vez e nunca mais.

use std::io;

/// Um arquivo dentro do zip: caminho (sempre com `/`) e conteúdo.
type Entrada<'a> = (String, &'a [u8]);

/// Monta os bytes do `.vsix` de uma extensão a partir dos arquivos dela.
///
/// `arquivos` são os caminhos relativos à raiz da extensão (`package.json`,
/// `syntaxes/x.json`, …), como o `build.rs` os embutiu.
pub fn empacotar(manifesto: &Manifesto, arquivos: &[(&str, &[u8])]) -> Vec<u8> {
    let nomes: Vec<String> = arquivos
        .iter()
        .map(|(rel, _)| format!("extension/{rel}"))
        .collect();

    let content_types = content_types(&nomes);
    let vsixmanifest = manifesto.para_xml();

    let mut entradas: Vec<Entrada> = vec![
        ("[Content_Types].xml".to_string(), content_types.as_bytes()),
        (
            "extension.vsixmanifest".to_string(),
            vsixmanifest.as_bytes(),
        ),
    ];
    for ((_, bytes), nome) in arquivos.iter().zip(&nomes) {
        entradas.push((nome.clone(), bytes));
    }

    zip(&entradas)
}

// ── Manifesto ────────────────────────────────────────────────────────────────

/// Os campos do `package.json` que o `extension.vsixmanifest` precisa.
pub struct Manifesto {
    pub publisher: String,
    pub nome: String,
    pub versao: String,
    pub display_name: String,
    pub descricao: String,
    pub engine: String,
    pub categorias: String,
}

impl Manifesto {
    /// Lê o `package.json` da extensão. Falha alto se faltar algum campo
    /// obrigatório: um `.vsix` com `Identity` incompleta é recusado pelo editor
    /// com uma mensagem bem menos clara do que esta.
    pub fn ler(package_json: &[u8]) -> io::Result<Self> {
        let json =
            std::str::from_utf8(package_json).map_err(|_| erro("package.json não é UTF-8"))?;

        let obrigatorio = |chave: &str| {
            json_str(json, &[chave])
                .ok_or_else(|| erro(&format!("package.json sem o campo '{chave}'")))
        };

        Ok(Self {
            publisher: obrigatorio("publisher")?,
            nome: obrigatorio("name")?,
            versao: obrigatorio("version")?,
            display_name: json_str(json, &["displayName"]).unwrap_or_default(),
            descricao: json_str(json, &["description"]).unwrap_or_default(),
            engine: json_str(json, &["engines", "vscode"]).unwrap_or_else(|| "^1.75.0".into()),
            categorias: json_lista(json, "categories").join(","),
        })
    }

    /// `publisher.name`, a identidade com que o editor registra a extensão (e o
    /// que `code --uninstall-extension` espera).
    pub fn id(&self) -> String {
        format!("{}.{}", self.publisher, self.nome)
    }

    fn para_xml(&self) -> String {
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011" xmlns:d="http://schemas.microsoft.com/developer/vsx-schema-design/2011">
  <Metadata>
    <Identity Language="en-US" Id="{nome}" Version="{versao}" Publisher="{publisher}" />
    <DisplayName>{display}</DisplayName>
    <Description xml:space="preserve">{descricao}</Description>
    <Tags></Tags>
    <Categories>{categorias}</Categories>
    <GalleryFlags>Public</GalleryFlags>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="{engine}" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionDependencies" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionPack" Value="" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionKind" Value="workspace" />
    </Properties>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code" />
  </Installation>
  <Dependencies/>
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
  </Assets>
</PackageManifest>
"#,
            nome = xml(&self.nome),
            versao = xml(&self.versao),
            publisher = xml(&self.publisher),
            display = xml(&self.display_name),
            descricao = xml(&self.descricao),
            categorias = xml(&self.categorias),
            engine = xml(&self.engine),
        )
    }
}

fn erro(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, msg.to_string())
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `[Content_Types].xml` com um `Default` por extensão de arquivo presente no
/// pacote. A especificação OPC recusa uma parte cujo tipo não esteja declarado,
/// então a lista sai do conteúdo real em vez de ser fixa — e um arquivo SEM
/// extensão (o `LICENSE` de uma das extensões) não tem `Default` que o cubra,
/// só um `Override` pelo nome inteiro.
fn content_types(nomes: &[String]) -> String {
    let mut exts: Vec<&str> = vec!["vsixmanifest", "json", "xml"];
    let mut sem_extensao: Vec<&str> = Vec::new();

    for nome in nomes {
        let arquivo = nome.rsplit('/').next().unwrap_or(nome);
        match arquivo.rsplit_once('.') {
            Some((_, ext)) if !ext.is_empty() => {
                if !exts.contains(&ext) {
                    exts.push(ext);
                }
            }
            _ => sem_extensao.push(nome),
        }
    }

    let mut s = String::from(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n",
    );
    for ext in exts {
        s.push_str(&format!(
            "  <Default Extension=\".{ext}\" ContentType=\"{}\"/>\n",
            content_type(ext)
        ));
    }
    for nome in sem_extensao {
        s.push_str(&format!(
            "  <Override PartName=\"/{}\" ContentType=\"text/plain\"/>\n",
            xml(nome)
        ));
    }
    s.push_str("</Types>\n");
    s
}

fn content_type(ext: &str) -> &'static str {
    match ext {
        "json" => "application/json",
        "js" | "mjs" | "cjs" => "application/javascript",
        "xml" | "vsixmanifest" => "text/xml",
        "md" | "txt" => "text/plain",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "ttf" => "font/ttf",
        "woff" | "woff2" => "font/woff",
        _ => "application/octet-stream",
    }
}

// ── JSON: leitura dos poucos campos que interessam ───────────────────────────

/// Valor string em `caminho` (`["engines", "vscode"]`), varrendo o JSON com
/// controle de aspas/escape e profundidade. É deliberadamente parcial — não
/// existe aqui um parser de JSON, só o suficiente para ler um `package.json`
/// de extensão sem arrastar `serde_json` para uma CLI sem dependências.
fn json_str(json: &str, caminho: &[&str]) -> Option<String> {
    let bytes = json.as_bytes();
    let mut i = 0usize;
    let mut profundidade = 0usize;
    // Índice, em `caminho`, do próximo segmento a casar.
    let mut nivel = 0usize;

    while i < bytes.len() {
        match bytes[i] {
            b'{' | b'[' => {
                profundidade += 1;
                i += 1;
            }
            b'}' | b']' => {
                profundidade = profundidade.saturating_sub(1);
                // Saímos do objeto onde o segmento já casado vivia: desfaz o
                // casamento, senão uma chave homônima mais adiante casaria.
                if nivel > 0 && profundidade < nivel {
                    nivel = profundidade;
                }
                i += 1;
            }
            b'"' => {
                let (texto, fim) = ler_string(bytes, i)?;
                let apos = pular_espacos(bytes, fim);
                // Só é chave se vier `:` depois, e só interessa no nível certo.
                if apos < bytes.len() && bytes[apos] == b':' && profundidade == nivel + 1 {
                    if texto == caminho[nivel] {
                        let v = pular_espacos(bytes, apos + 1);
                        if nivel + 1 == caminho.len() {
                            return ler_string(bytes, v).map(|(s, _)| s);
                        }
                        nivel += 1;
                    }
                    i = apos + 1;
                } else {
                    i = fim;
                }
            }
            _ => i += 1,
        }
    }
    None
}

/// Itens string de um array no nível de topo (`"categories": ["a", "b"]`).
fn json_lista(json: &str, chave: &str) -> Vec<String> {
    let bytes = json.as_bytes();
    let alvo = format!("\"{chave}\"");
    let Some(inicio) = json.find(&alvo) else {
        return Vec::new();
    };
    let mut i = pular_espacos(bytes, inicio + alvo.len());
    if i >= bytes.len() || bytes[i] != b':' {
        return Vec::new();
    }
    i = pular_espacos(bytes, i + 1);
    if i >= bytes.len() || bytes[i] != b'[' {
        return Vec::new();
    }
    i += 1;

    let mut itens = Vec::new();
    while i < bytes.len() && bytes[i] != b']' {
        if bytes[i] == b'"' {
            let Some((texto, fim)) = ler_string(bytes, i) else {
                break;
            };
            itens.push(texto);
            i = fim;
        } else {
            i += 1;
        }
    }
    itens
}

/// Lê a string JSON que começa em `inicio` (que aponta para a aspa de abertura).
/// Devolve o conteúdo e o índice logo após a aspa de fechamento.
fn ler_string(bytes: &[u8], inicio: usize) -> Option<(String, usize)> {
    if bytes.get(inicio) != Some(&b'"') {
        return None;
    }
    let mut texto = Vec::new();
    let mut i = inicio + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                // Escapes: só os que aparecem num package.json (`\"`, `\\`, `\/`,
                // `\n`). `\uXXXX` fica literal — nenhum campo lido aqui o usa.
                let proximo = *bytes.get(i + 1)?;
                texto.push(match proximo {
                    b'n' => b'\n',
                    b't' => b'\t',
                    outro => outro,
                });
                i += 2;
            }
            b'"' => return Some((String::from_utf8_lossy(&texto).into_owned(), i + 1)),
            outro => {
                texto.push(outro);
                i += 1;
            }
        }
    }
    None
}

fn pular_espacos(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

// ── Zip (stored) ─────────────────────────────────────────────────────────────

fn zip(entradas: &[Entrada]) -> Vec<u8> {
    let mut saida = Vec::new();
    let mut central = Vec::new();

    for (nome, dados) in entradas {
        let offset = saida.len() as u32;
        let crc = crc32(dados);
        let nome_b = nome.as_bytes();

        saida.extend_from_slice(&0x0403_4b50u32.to_le_bytes()); // assinatura local
        saida.extend_from_slice(&20u16.to_le_bytes()); // versão necessária
        saida.extend_from_slice(&0u16.to_le_bytes()); // flags
        saida.extend_from_slice(&0u16.to_le_bytes()); // método: stored
        saida.extend_from_slice(&0u16.to_le_bytes()); // hora (fixa: sem relógio)
        saida.extend_from_slice(&0x0021u16.to_le_bytes()); // data: 1980-01-01
        saida.extend_from_slice(&crc.to_le_bytes());
        saida.extend_from_slice(&(dados.len() as u32).to_le_bytes());
        saida.extend_from_slice(&(dados.len() as u32).to_le_bytes());
        saida.extend_from_slice(&(nome_b.len() as u16).to_le_bytes());
        saida.extend_from_slice(&0u16.to_le_bytes()); // extra
        saida.extend_from_slice(nome_b);
        saida.extend_from_slice(dados);

        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes()); // assinatura central
        central.extend_from_slice(&20u16.to_le_bytes()); // versão de origem
        central.extend_from_slice(&20u16.to_le_bytes()); // versão necessária
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0x0021u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(dados.len() as u32).to_le_bytes());
        central.extend_from_slice(&(dados.len() as u32).to_le_bytes());
        central.extend_from_slice(&(nome_b.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes()); // extra
        central.extend_from_slice(&0u16.to_le_bytes()); // comentário
        central.extend_from_slice(&0u16.to_le_bytes()); // disco
        central.extend_from_slice(&0u16.to_le_bytes()); // atributos internos
        central.extend_from_slice(&0u32.to_le_bytes()); // atributos externos
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(nome_b);
    }

    let inicio_central = saida.len() as u32;
    let tamanho_central = central.len() as u32;
    saida.extend_from_slice(&central);

    saida.extend_from_slice(&0x0605_4b50u32.to_le_bytes()); // fim do diretório
    saida.extend_from_slice(&0u16.to_le_bytes()); // disco
    saida.extend_from_slice(&0u16.to_le_bytes()); // disco do início
    saida.extend_from_slice(&(entradas.len() as u16).to_le_bytes());
    saida.extend_from_slice(&(entradas.len() as u16).to_le_bytes());
    saida.extend_from_slice(&tamanho_central.to_le_bytes());
    saida.extend_from_slice(&inicio_central.to_le_bytes());
    saida.extend_from_slice(&0u16.to_le_bytes()); // comentário
    saida
}

fn crc32(dados: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in dados {
        crc ^= b as u32;
        for _ in 0..8 {
            let mascara = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mascara);
        }
    }
    !crc
}

#[cfg(test)]
mod testes {
    use super::*;

    const PACOTE: &str = r#"{
      "name": "glacier-view",
      "displayName": "Glacier View",
      "description": "Suporte a .gv com \"aspas\" dentro",
      "version": "0.3.0",
      "publisher": "antoniofernandodj",
      "engines": { "vscode": "^1.75.0" },
      "categories": ["Programming Languages"],
      "contributes": { "languages": [{ "id": "glacier-view", "version": "ignorar" }] }
    }"#;

    #[test]
    fn le_os_campos_do_package_json() {
        let m = Manifesto::ler(PACOTE.as_bytes()).unwrap();
        assert_eq!(m.id(), "antoniofernandodj.glacier-view");
        assert_eq!(m.versao, "0.3.0");
        assert_eq!(m.engine, "^1.75.0");
        assert_eq!(m.categorias, "Programming Languages");
        assert!(m.descricao.contains("\"aspas\""));
    }

    /// `version` aparece duas vezes no JSON — a de topo e uma aninhada em
    /// `contributes`. Ler a aninhada daria um `.vsix` com a versão errada.
    #[test]
    fn chave_aninhada_nao_e_confundida_com_a_de_topo() {
        assert_eq!(json_str(PACOTE, &["version"]).as_deref(), Some("0.3.0"));
    }

    #[test]
    fn crc32_bate_com_o_valor_conhecido() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn zip_tem_assinatura_e_fim_de_diretorio() {
        let bytes = zip(&[("a.txt".to_string(), b"oi".as_slice())]);
        assert_eq!(&bytes[..4], &0x0403_4b50u32.to_le_bytes());
        assert!(bytes.windows(4).any(|w| w == 0x0605_4b50u32.to_le_bytes()));
    }

    #[test]
    fn content_types_declara_toda_extensao_presente() {
        let nomes = ["extension/extension.js", "extension/icons/a.svg"].map(String::from);
        let ct = content_types(&nomes);
        assert!(ct.contains("Extension=\".js\""));
        assert!(ct.contains("Extension=\".svg\""));
        assert!(ct.contains("Extension=\".vsixmanifest\""));
    }

    /// Sem `Default` que o cubra, um arquivo sem extensão precisa de `Override`
    /// — do contrário o pacote é inválido para a especificação OPC.
    #[test]
    fn arquivo_sem_extensao_ganha_override() {
        let ct = content_types(&[String::from("extension/LICENSE")]);
        assert!(ct.contains("<Override PartName=\"/extension/LICENSE\""));
    }
}
