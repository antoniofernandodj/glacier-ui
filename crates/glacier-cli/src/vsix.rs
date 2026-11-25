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

use crate::json::{self, Valor};

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
        let raiz = json::parse(json).map_err(|e| erro(&format!("package.json inválido: {e}")))?;

        let texto = |caminho: &[&str]| raiz.caminho(caminho).and_then(Valor::como_str);
        let obrigatorio = |chave: &str| {
            texto(&[chave])
                .map(str::to_owned)
                .ok_or_else(|| erro(&format!("package.json sem o campo '{chave}'")))
        };
        let categorias: Vec<&str> = raiz
            .get("categories")
            .and_then(Valor::como_lista)
            .unwrap_or_default()
            .iter()
            .filter_map(Valor::como_str)
            .collect();

        Ok(Self {
            publisher: obrigatorio("publisher")?,
            nome: obrigatorio("name")?,
            versao: obrigatorio("version")?,
            display_name: texto(&["displayName"]).unwrap_or_default().to_owned(),
            descricao: texto(&["description"]).unwrap_or_default().to_owned(),
            engine: texto(&["engines", "vscode"]).unwrap_or("^1.75.0").to_owned(),
            categorias: categorias.join(","),
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

/// CRC-32 (IEEE), o do zip. Também usado pelo vigia do `serve wasm --watch`
/// para comparar CONTEÚDO de arquivo — ver `serve::Impressao`.
pub fn crc32(dados: &[u8]) -> u32 {
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
        let m = Manifesto::ler(PACOTE.as_bytes()).unwrap();
        assert_eq!(m.versao, "0.3.0");
    }

    /// A varredura antiga lia o primeiro `"categories"` do texto, onde quer que
    /// estivesse — inclusive dentro de outro campo — e um campo escrito num
    /// comentário valia como o de verdade.
    #[test]
    fn campo_citado_em_texto_ou_comentario_nao_vale() {
        let json = r#"{
          // "version": "9.9.9"
          "description": "lista em \"categories\": [\"Falsa\"] e \"version\": \"8.8.8\"",
          "name": "x", "publisher": "p", "version": "1.2.3",
          "categories": ["Verdadeira"]
        }"#;
        let m = Manifesto::ler(json.as_bytes()).unwrap();
        assert_eq!(m.versao, "1.2.3");
        assert_eq!(m.categorias, "Verdadeira");
    }

    #[test]
    fn package_json_malformado_diz_onde() {
        let Err(e) = Manifesto::ler(b"{\n  \"name\": ,\n}") else {
            panic!("deveria falhar");
        };
        assert!(e.to_string().contains("2:"), "{e}");
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
