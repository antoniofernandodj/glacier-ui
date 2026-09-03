use crate::ContextMap;
use crate::error::Result;
use crate::parser::{BoolAttr, NodeType, NumAttr, UiNode};
use crate::stylesheet::{
    StateStyles, StyleRule, StyleSheet, resolve_classes, resolve_state_classes,
};
use std::cell::OnceCell;
use std::collections::HashMap;

/// Splits a `<script>...</script>` block out of an XML document, returning the
/// markup with the block removed and the script body (if any).
///
/// The script is stripped *before* XML parsing, so it may sit as a sibling of
/// the root element (it would otherwise make the document multi-rooted). The
/// markup parser ignores the script; its Lua body is interpreted at runtime by
/// [`crate::luau::LuauComponent`].
///
/// O bloco é substituído por **tantas quebras de linha quantas ele ocupava**, em
/// vez de simplesmente sumir. Sem isso, todo o markup abaixo de um `<script>`
/// inline de 30 linhas subiria 30 linhas aos olhos do parser de XML — e um erro
/// na linha 80 sairia reportado como linha 50, que é pior do que não ter linha
/// nenhuma: manda o autor olhar para um trecho inocente.
pub fn strip_script(xml: &str) -> (String, Option<String>) {
    let Some(open_start) = find_script_open(xml) else {
        return (xml.to_string(), None);
    };
    // Find the end of the opening tag (supports `<script>` and `<script ...>`).
    let Some(gt_rel) = xml[open_start..].find('>') else {
        return (xml.to_string(), None);
    };
    let body_start = open_start + gt_rel + 1;
    let lower_tail = xml[body_start..].to_ascii_lowercase();
    let Some(close_rel) = lower_tail.find("</script>") else {
        return (xml.to_string(), None);
    };

    let body_end = body_start + close_rel;
    let close_end = body_end + "</script>".len();
    let script = xml[body_start..body_end].to_string();

    let mut markup = String::with_capacity(xml.len());
    markup.push_str(&xml[..open_start]);
    for _ in 0..xml[open_start..close_end].matches('\n').count() {
        markup.push('\n');
    }
    markup.push_str(&xml[close_end..]);
    (markup, Some(script))
}

/// Índice do `<script` que abre o bloco de script — ignorando um citado dentro
/// de um comentário XML (`<!-- <script> -->`), que não é um bloco de verdade.
///
/// É a **única** definição de onde o bloco começa, e por isso é `pub(crate)`:
/// além do [`strip_script`] aqui, a camada Luau precisa exatamente da mesma
/// resposta (`crate::luau::extract_script`/`extract_script_src`). Enquanto ela
/// tinha uma varredura própria — um `find("<script")` cru, sem pular
/// comentários —, as duas discordavam: o parser de markup via o bloco certo e o
/// Luau via o do comentário, extraindo como código o texto entre a tag citada e
/// o `</script>` de verdade. O sintoma era um erro de sintaxe Luau na linha 1,
/// sem relação visível com o comentário que o causou.
///
/// O índice vale tanto no texto original quanto no minúsculo: `to_ascii_lowercase`
/// só troca bytes ASCII, então nenhum deslocamento muda.
pub(crate) fn find_script_open(xml: &str) -> Option<usize> {
    let lower = xml.to_ascii_lowercase();
    let mut from = 0;
    while let Some(i) = lower[from..].find("<script").map(|i| from + i) {
        // Dentro de um comentário? Basta olhar para trás: se o `<!--` mais
        // recente ainda não foi fechado por um `-->`, estamos comentados.
        let before = &lower[..i];
        let open = before.rfind("<!--");
        let closed = open.is_none_or(|o| before[o..].contains("-->"));
        if closed {
            return Some(i);
        }
        from = i + 7;
    }
    None
}

/// Normalizes bare directives like `else` or `senao` (without value) inside XML tags
/// by rewriting them to `else=""` or `senao=""` before XML parsing.
pub fn normalize_bare_directives(xml: &str) -> String {
    let mut result = String::with_capacity(xml.len());
    let mut in_tag = false;
    let mut in_comment = false;
    let mut quote_char = None;
    let chars: Vec<char> = xml.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if in_comment {
            // Check for end of comment "-->"
            if i + 2 < chars.len() && chars[i] == '-' && chars[i + 1] == '-' && chars[i + 2] == '>'
            {
                result.push('-');
                result.push('-');
                result.push('>');
                in_comment = false;
                i += 3;
            } else {
                result.push(chars[i]);
                i += 1;
            }
            continue;
        }

        // Check for start of comment "<!--"
        if i + 3 < chars.len()
            && chars[i] == '<'
            && chars[i + 1] == '!'
            && chars[i + 2] == '-'
            && chars[i + 3] == '-'
        {
            result.push_str("<!--");
            in_comment = true;
            i += 4;
            continue;
        }

        let c = chars[i];
        if !in_tag {
            if c == '<' {
                in_tag = true;
                quote_char = None;
            }
            result.push(c);
            i += 1;
        } else {
            // We are inside a tag
            if c == '>' {
                in_tag = false;
                result.push(c);
                i += 1;
            } else if let Some(q) = quote_char {
                if c == q {
                    quote_char = None;
                }
                result.push(c);
                i += 1;
            } else {
                // Not in quotes
                if c == '"' || c == '\'' {
                    quote_char = Some(c);
                    result.push(c);
                    i += 1;
                } else {
                    // Bare directives: an attribute name with no `="…"`
                    // value, rewritten to `name=""` so the XML parser
                    // accepts it. Longest word first so a shorter word that
                    // happens to be a PREFIX of a longer one (there isn't
                    // one today, but keep the invariant) never shadows it.
                    const BARE_WORDS: &[&str] = &["not_empty", "senao", "empty", "else", "vazio"];

                    let mut matched_len = None;
                    let mut replaced_with = None;
                    let remaining_len = chars.len() - i;
                    for word in BARE_WORDS {
                        let len = word.len();
                        if remaining_len >= len {
                            let candidate: String = chars[i..i + len].iter().collect();
                            if candidate.eq_ignore_ascii_case(word) {
                                matched_len = Some(len);
                                replaced_with = Some(*word);
                                break;
                            }
                        }
                    }

                    // A longer attribute name that merely STARTS with one of
                    // `BARE_WORDS` (`else-if`, `not_empty_something`) must be
                    // left alone — only the bare word followed by a
                    // name-boundary char (whitespace, `=`, `>`, `/`) counts.
                    // Without this check, `else-if="…"` got rewritten into
                    // the invalid `else=""-if="…"` (the bug this comment
                    // fixes, found while adding `else-if` — ver
                    // `docs/plano-convergencia-templates-gui-webui.md` Fase 1
                    // item 2 no rustploy).
                    if let (Some(len), Some(_)) = (matched_len, replaced_with)
                        && i + len < chars.len()
                    {
                        let next_char = chars[i + len];
                        if next_char.is_ascii_alphanumeric() || next_char == '-' || next_char == '_'
                        {
                            matched_len = None;
                            replaced_with = None;
                        }
                    }

                    if let (Some(len), Some(word)) = (matched_len, replaced_with) {
                        // Check preceding character (must be whitespace for an attribute)
                        let preceded_ok = i > 0 && chars[i - 1].is_ascii_whitespace();

                        if preceded_ok {
                            // Check succeeding characters to see if it's followed by '='
                            let mut next_idx = i + len;
                            while next_idx < chars.len() && chars[next_idx].is_ascii_whitespace() {
                                next_idx += 1;
                            }
                            let is_followed_by_equals =
                                next_idx < chars.len() && chars[next_idx] == '=';

                            if !is_followed_by_equals {
                                // It is a bare attribute! Replace it.
                                result.push_str(word);
                                result.push_str("=\"\"");
                                i += len;
                                continue;
                            }
                        }
                    }

                    result.push(c);
                    i += 1;
                }
            }
        }
    }
    result
}

/// O contexto **durante a avaliação**: a base (o contexto do motor) mais uma
/// cadeia de camadas com as variáveis locais — as vars de um item de `for-each`
/// (`{item.nome}`) e as props de um componente.
///
/// Existe para não **clonar a base**. A versão anterior fazia
/// `let mut local_context = context.clone()` por **item** de lista: com 45 linhas
/// na tela e um log de 100 KB no contexto, isso é copiar ~5 MB de string por
/// reavaliação — e a reavaliação roda a cada tecla e a cada mensagem do SSE. Era
/// o que fazia uma árvore de 600 nós custar 6,5 ms quando os nós em si custam
/// uma fração disso.
///
/// A busca vai da camada mais **interna** para a mais externa e só então na base,
/// então uma var local sombreia uma chave global de mesmo nome — exatamente o que
/// o `insert` sobre o clone fazia. As camadas têm poucas entradas (os campos de
/// um item), então a varredura linear é mais barata que um `HashMap`.
#[derive(Clone, Copy)]
pub struct EvalCtx<'a> {
    base: &'a ContextMap,
    /// A camada mais interna; cada uma aponta para a de fora (lista ligada na
    /// pilha, sem alocação).
    layer: Option<&'a Layer<'a>>,
    /// Registrador de leituras — toda chave consultada por [`EvalCtx::get`] é
    /// anotada aqui. É o que dá o **conjunto de dependências** de uma subárvore,
    /// e portanto o que torna possível saber que ela *não* precisa ser
    /// reconstruída. `None` quando ninguém está rastreando (avaliação avulsa).
    reads: Option<&'a Reads>,
    /// Identidade da **instância** desta posição na árvore avaliada: um hash do
    /// caminho (nó do AST + índice do item, acumulado a cada nível de
    /// `for-each`). Duas linhas de uma lista compartilham o nó do AST mas têm
    /// caminhos distintos — sem isso, uma sobrescreveria a entrada de cache da
    /// outra e o cache nunca acertaria.
    path: u64,
    /// Quantas camadas há sobre a base. É o que dá sentido à profundidade
    /// registrada em cada leitura (ver [`Frame`]).
    depth: u32,
}

/// Coleta as chaves de contexto lidas durante a avaliação, em **quadros**
/// aninhados: um por subárvore candidata a cache.
///
/// Ao fechar um quadro, suas leituras são mescladas no quadro de fora — uma
/// chave lida lá no fundo de uma subárvore também é dependência de todos os
/// ancestrais dela. Sem essa propagação, o cache do pai acharia que não depende
/// de algo de que depende, e serviria uma árvore velha: o pior tipo de bug de
/// UI, silencioso e intermitente. É por isso que o rastreamento vive no
/// [`EvalCtx::get`] — o **único** caminho de leitura — e não numa análise
/// estática do template, que poderia esquecer um caso.
/// As dependências de uma subárvore: cada chave de contexto que ela leu e o
/// valor que a chave tinha na avaliação. A entrada de cache só vale enquanto
/// **todas** ainda casarem.
pub type Deps = Vec<(String, Option<String>)>;

#[derive(Default)]
pub struct Reads {
    frames: std::cell::RefCell<Vec<Frame>>,
}

/// Um quadro de leituras: as chaves lidas por uma subárvore, cada uma com o
/// valor visto e a **profundidade da camada que a resolveu** (0 = a base).
///
/// A profundidade é o que impede uma variável local de contaminar quem está por
/// fora. `{l.nome}` só existe na camada do item; se ela subisse até o conjunto de
/// dependências do *template*, o motor iria perguntar "o contexto ainda tem
/// `l.nome` com o valor X?" — e a resposta é sempre não, porque `l.nome` nunca
/// esteve no contexto. O template ficaria **eternamente sujo** e nunca
/// reaproveitaria nada. Ao fechar um quadro de profundidade `d`, só sobem as
/// leituras resolvidas *fora* dele (`src < d`).
struct Frame {
    depth: u32,
    reads: FxMap<String, (Option<String>, u32)>,
}

impl Reads {
    /// Anota a leitura de `key` (o valor visto e a profundidade que a resolveu)
    /// no quadro corrente.
    fn record(&self, key: &str, value: Option<&str>, src: u32) {
        if let Some(frame) = self.frames.borrow_mut().last_mut() {
            // A mesma chave é lida muitas vezes por subárvore, e o `entry` da
            // `std` exige a chave **possuída** — ou seja, alocava uma `String`
            // a cada leitura só para descobrir que já havia uma igual guardada.
            // A consulta antes da inserção troca essa alocação por um segundo
            // hash, e o hash aqui é o [`FxHasher`].
            if frame.reads.contains_key(key) {
                return;
            }
            frame
                .reads
                .insert(key.to_string(), (value.map(str::to_string), src));
        }
    }

    fn push(&self, depth: u32) {
        self.frames.borrow_mut().push(Frame {
            depth,
            reads: FxMap::default(),
        });
    }

    /// Fecha o quadro corrente, devolvendo **todas** as suas dependências (é o
    /// que valida a entrada de cache dele, avaliada com as camadas em vigor) e
    /// propagando para o quadro de fora só as que vêm de fora dele.
    fn pop(&self) -> Deps {
        let mut frames = self.frames.borrow_mut();
        let Some(frame) = frames.pop() else {
            return Vec::new();
        };
        if let Some(parent) = frames.last_mut() {
            for (k, (v, src)) in &frame.reads {
                if *src < frame.depth {
                    parent
                        .reads
                        .entry(k.clone())
                        .or_insert_with(|| (v.clone(), *src));
                }
            }
        }
        frame.reads.into_iter().map(|(k, (v, _))| (k, v)).collect()
    }

    /// Propaga as dependências de uma subárvore **reaproveitada do cache** (que
    /// portanto não foi reavaliada, e não registrou leitura nenhuma) para o
    /// quadro corrente — senão o ancestral acharia que não depende delas.
    ///
    /// `depth` é a profundidade da subárvore reusada: mesma regra do `pop`, só
    /// sobe o que foi resolvido fora dela.
    fn merge(&self, deps: &[(String, Option<String>)], depth: u32, ctx: &EvalCtx) {
        let mut frames = self.frames.borrow_mut();
        let Some(frame) = frames.last_mut() else {
            return;
        };
        for (k, v) in deps {
            // A entrada de cache guarda o valor, não a origem — recalculamos a
            // profundidade contra as camadas de agora (as mesmas contra as quais
            // as dependências acabaram de ser validadas).
            if ctx.src_depth(k) < depth {
                frame
                    .reads
                    .entry(k.clone())
                    .or_insert_with(|| (v.clone(), ctx.src_depth(k)));
            }
        }
    }
}

/// Subárvores já avaliadas, guardadas entre reavaliações e reaproveitadas quando
/// nada de que dependem mudou.
///
/// Reusar custa um `clone` da subárvore; medido na árvore real do rustploy,
/// clonar é **14× mais barato** que reavaliar (0,75 µs/nó contra 10,5 µs/nó — o
/// grosso de avaliar um nó é resolver o estilo dele e montar um `UiNode` de ~40
/// campos). É essa razão que faz a memoização valer a pena.
#[derive(Default)]
pub struct EvalCache {
    /// A época dos [`crate::render_inputs::RenderInputs`] em que estas entradas
    /// foram construídas. Quando ela avança — folha de estilo nova, viewport
    /// cruzando `@media`, markup recarregado —, tudo aqui pode estar obsoleto e
    /// o cache se descarta sozinho em [`EvalCache::sync`]. É o que tirou essa
    /// invariante das mãos de quem escreve o call-site.
    epoch: u64,
    entries: FxMap<u64, CacheEntry>,
    /// Entradas tocadas na passada corrente. O que sobrar fora daqui ao final é
    /// lixo (uma linha que saiu da lista) e é varrido — senão o cache cresceria
    /// sem limite ao longo da vida do app.
    live: std::collections::HashSet<u64, Fx>,
    /// Arrays de `for-each` **já parseados**, por chave de contexto.
    ///
    /// Uma coleção mora no contexto como **texto** (`"[{...},{...}]"`), e o
    /// `for-each` precisava dela como `Value` — então parseava a lista inteira
    /// a cada reavaliação, mesmo quando nada nela tinha mudado e todos os itens
    /// vinham do cache. Numa lista de 300 linhas isso era ~14% de todo o
    /// trabalho de uma mudança de estado: parse, um `BTreeMap` por objeto, e o
    /// descarte de tudo em seguida.
    ///
    /// A validade é conferida **comparando o texto**, não um hash: a cópia
    /// guardada aqui e a do contexto têm de ser iguais byte a byte. Um `memcmp`
    /// de 20 KB é ordens de grandeza mais barato que o parse, e não abre a
    /// porta para o que um hash de 64 bits abriria — servir a lista velha por
    /// colisão, silenciosamente.
    json: FxMap<String, (String, std::sync::Arc<Vec<serde_json::Value>>)>,
}

struct CacheEntry {
    /// As chaves de que a subárvore depende, e o valor que tinham quando ela foi
    /// construída. A entrada só vale enquanto **todos** ainda casarem.
    ///
    /// Compartilhada por `Arc` porque um acerto de cache precisa dela **e** do
    /// `&mut` no cache ao mesmo tempo: antes, o jeito de sair dessa disputa era
    /// clonar o vetor inteiro de pares de `String` a cada acerto — trabalho puro
    /// de empréstimo, invisível e pago por item de lista.
    deps: std::sync::Arc<Deps>,
    nodes: crate::parser::Children,
}

impl EvalCache {
    /// Alinha o cache com a época atual dos [`crate::render_inputs::RenderInputs`]:
    /// se ela avançou, **descarta tudo** e devolve `true`.
    ///
    /// É o coração da correção do cache. O que ele rastreia são chaves de
    /// *contexto*; folha de estilo, viewport e markup mudam a árvore sem passar
    /// por leitura nenhuma. Em vez de pedir a cada call-site que se lembre de
    /// avisar — oito lembretes espalhados, na primeira versão, e um deles já
    /// estava furado —, os inputs contam as próprias mudanças e o cache confere a
    /// conta.
    pub fn sync(&mut self, epoch: u64) -> bool {
        if self.epoch == epoch {
            return false;
        }
        self.epoch = epoch;
        self.entries.clear();
        self.live.clear();
        self.json.clear();
        true
    }

    /// Remove as entradas não usadas na última passada (itens que sumiram da
    /// lista). Chamado ao fim de cada avaliação de template.
    fn sweep(&mut self) {
        self.entries.retain(|k, _| self.live.contains(k));
        self.live.clear();
    }

    /// O array de `chave` já parseado, reaproveitando o parse anterior quando o
    /// texto no contexto é **exatamente** o mesmo. Devolve `None` quando a
    /// chave não existe ou não guarda um array JSON.
    fn array(
        &mut self,
        chave: &str,
        bruto: &str,
    ) -> Option<std::sync::Arc<Vec<serde_json::Value>>> {
        if let Some((texto, arr)) = self.json.get(chave)
            && texto == bruto
        {
            return Some(std::sync::Arc::clone(arr));
        }
        let serde_json::Value::Array(arr) = serde_json::from_str(bruto).ok()? else {
            // Não é array: não guarda nada (e limpa o que houvesse), para não
            // manter viva a lista antiga de uma chave que virou outra coisa.
            self.json.remove(chave);
            return None;
        };
        let arr = std::sync::Arc::new(arr);
        self.json
            .insert(chave.to_string(), (bruto.to_string(), arr.clone()));
        Some(arr)
    }
}

/// O item de um `for-each` **ainda como JSON**, resolvido campo a campo só
/// quando alguém lê.
///
/// A versão ansiosa disto — materializar `{item.campo}` para todo campo, mais
/// uma serialização do item inteiro para o `{item}` de um `spread=` — rodava
/// por item **antes** da consulta ao cache, ou seja, também para o item que ia
/// ser reaproveitado inteiro. Numa lista de 2000 linhas de 4 campos eram 8000
/// `format!` mais 8000 clones mais 2000 serializações, jogados fora em seguida.
///
/// Preguiçoso, cada leitura paga só por si — e o caso comum não paga nada: um
/// campo de **texto** sai emprestado direto do JSON, sem alocar. O que sobra
/// (número, booleano, o item inteiro) materializa uma vez, numa [`OnceCell`],
/// e fica.
struct ItemVars<'a> {
    /// O nome da variável do laço (`var="l"` → `"l"`).
    var: &'a str,
    /// Os campos do objeto: o nome **emprestado do próprio JSON**, o valor cru,
    /// e a materialização sob demanda de quem não for string.
    campos: Vec<(&'a str, &'a serde_json::Value, OnceCell<String>)>,
    /// O item inteiro (`{item}`, o que um `spread=` repassa a um componente),
    /// serializado só se for lido.
    inteiro: (&'a serde_json::Value, OnceCell<String>),
}

impl ItemVars<'_> {
    /// O valor de `key` neste item, ou `None` se a chave não é dele.
    fn get(&self, key: &str) -> Option<&str> {
        if key == self.var {
            let (v, memo) = &self.inteiro;
            // `json_scalar` já faz a distinção que importa: uma string sai crua
            // (item escalar), e todo o resto — objeto incluído — sai como JSON,
            // que é o que um `spread=` espera receber de volta.
            return Some(memo.get_or_init(|| json_scalar(v)));
        }
        let campo = key.strip_prefix(self.var)?.strip_prefix('.')?;
        let (_, val, memo) = self.campos.iter().find(|(k, _, _)| *k == campo)?;
        Some(match val {
            // O caso esmagadoramente comum: sai emprestado, zero alocação.
            serde_json::Value::String(s) => s.as_str(),
            outro => memo.get_or_init(|| json_scalar(outro)),
        })
    }
}

/// O hash da `std` é o SipHash, escolhido para resistir a ataque de colisão em
/// mapa alimentado por entrada hostil — um servidor web. Aqui não há nada
/// disso: as chaves são nomes de variável do próprio app e caminhos de nó
/// gerados pelo motor, e o custo aparece no perfil como **18% de todo o
/// trabalho** de uma mudança de estado (hashar um `u64` com SipHash, uma vez
/// por consulta ao cache, é o caso mais absurdo).
///
/// Este é o FxHash do rustc: uma multiplicação e um shift por palavra. Trocado
/// só nos mapas **internos** do avaliador — o `HashMap` do contexto é tipo
/// público (`GlacierUI::context`) e fica como está.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

    #[inline]
    fn adiciona(&mut self, palavra: u64) {
        self.hash = (self.hash.rotate_left(5) ^ palavra).wrapping_mul(Self::SEED);
    }
}

impl std::hash::Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut resto = bytes;
        while resto.len() >= 8 {
            let (a, b) = resto.split_at(8);
            self.adiciona(u64::from_ne_bytes(a.try_into().unwrap()));
            resto = b;
        }
        if !resto.is_empty() {
            let mut buf = [0u8; 8];
            buf[..resto.len()].copy_from_slice(resto);
            self.adiciona(u64::from_ne_bytes(buf));
        }
    }

    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.adiciona(n);
    }

    #[inline]
    fn write_usize(&mut self, n: usize) {
        self.adiciona(n as u64);
    }

    #[inline]
    fn write_u8(&mut self, n: u8) {
        self.adiciona(n as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// `BuildHasher` do [`FxHasher`], para os mapas internos do avaliador.
pub type Fx = std::hash::BuildHasherDefault<FxHasher>;

/// `HashMap` com o hash rápido.
pub(crate) type FxMap<K, V> = HashMap<K, V, Fx>;

/// O mesmo, visível fora do avaliador.
pub type FxMapPub<K, V> = HashMap<K, V, Fx>;

/// Um conjunto de variáveis locais empilhado sobre o contexto. Ver [`EvalCtx`].
pub struct Layer<'a> {
    vars: Vec<(String, String)>,
    /// As variáveis de um item de `for-each`, preguiçosas. Ver [`ItemVars`].
    item: Option<ItemVars<'a>>,
    outer: Option<&'a Layer<'a>>,
}

impl<'a> Layer<'a> {
    fn new(outer: Option<&'a Layer<'a>>) -> Self {
        Self {
            vars: Vec::new(),
            item: None,
            outer,
        }
    }

    fn set(&mut self, key: String, value: String) {
        // Uma chave repetida na MESMA camada sobrescreve (semântica de `insert`).
        match self.vars.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = value,
            None => self.vars.push((key, value)),
        }
    }

    /// O valor de `key` e a **profundidade** da camada que o resolveu, sabendo
    /// que `self` está em `depth`. Cada passo para fora desce um nível; 0 é a
    /// base. Ver [`Frame`].
    fn get(&self, key: &str, depth: u32) -> Option<(&str, u32)> {
        let mut cur = Some(self);
        let mut d = depth;
        while let Some(l) = cur {
            // `vars` primeiro: é onde mora o que foi escrito por cima do item
            // (o `{var.__dragging}`, por exemplo, que sobrescreve um campo
            // homônimo — a mesma precedência que o `set` ansioso dava).
            if let Some((_, v)) = l.vars.iter().find(|(k, _)| k == key) {
                return Some((v, d));
            }
            if let Some(v) = l.item.as_ref().and_then(|i| i.get(key)) {
                return Some((v, d));
            }
            cur = l.outer;
            d = d.saturating_sub(1);
        }
        None
    }
}

impl<'a> EvalCtx<'a> {
    /// Contexto de avaliação sobre `base`, sem camadas nem rastreamento.
    pub fn new(base: &'a ContextMap) -> Self {
        Self {
            base,
            layer: None,
            reads: None,
            path: 0,
            depth: 0,
        }
    }

    /// O mesmo, rastreando as leituras em `reads` (o que habilita o cache).
    fn tracked(base: &'a ContextMap, reads: &'a Reads) -> Self {
        Self {
            base,
            layer: None,
            reads: Some(reads),
            path: 0,
            depth: 0,
        }
    }

    /// Resolve `key` sem registrar a leitura: o valor e a profundidade da camada
    /// que o deu (0 = base).
    fn lookup(&self, key: &str) -> (Option<&str>, u32) {
        match self.layer.and_then(|l| l.get(key, self.depth)) {
            Some((v, d)) => (Some(v), d),
            None => (self.base.get(key).map(String::as_str), 0),
        }
    }

    /// A profundidade da camada que resolve `key` hoje (0 = base/ausente).
    fn src_depth(&self, key: &str) -> u32 {
        self.lookup(key).1
    }

    /// O valor de `key`: camadas locais (da mais interna para a mais externa)
    /// primeiro, base depois.
    ///
    /// **Único** caminho de leitura do contexto durante a avaliação — é o que
    /// permite ao rastreamento ser completo por construção, em vez de depender
    /// de eu ter lembrado de anotar cada call-site.
    pub fn get(&self, key: &str) -> Option<&str> {
        let (value, src) = self.lookup(key);
        if let Some(reads) = self.reads {
            reads.record(key, value, src);
        }
        value
    }

    /// O mesmo contexto com `layer` empilhada por cima (a camada precisa viver
    /// no frame do chamador — é isso que torna a operação O(1), sem cópia), o
    /// caminho estendido por `step` (a identidade desta instância; ver
    /// [`EvalCtx::path`]) e a profundidade incrementada.
    /// Quantas camadas há sobre a base — a profundidade da expansão. Lida pela
    /// guarda de recursão de `eval_owned`.
    fn depth(&self) -> u32 {
        self.depth
    }

    fn with<'c>(&self, layer: &'c Layer<'c>, step: u64) -> EvalCtx<'c>
    where
        'a: 'c,
    {
        EvalCtx {
            base: self.base,
            layer: Some(layer),
            reads: self.reads,
            path: mix(self.path, step),
            depth: self.depth + 1,
        }
    }

    /// A camada corrente, para uma nova ser encadeada sob ela.
    fn layer(&self) -> Option<&'a Layer<'a>> {
        self.layer
    }

    /// Confere se as dependências guardadas numa entrada de cache ainda batem
    /// com o contexto de agora. É a pergunta "algo de que essa subárvore depende
    /// mudou?" — e nada além disso decide um acerto de cache.
    fn deps_hold(&self, deps: &[(String, Option<String>)]) -> bool {
        deps.iter().all(|(k, v)| self.lookup(k).0 == v.as_deref())
    }
}

impl EvalCtx<'_> {
    /// Abre um quadro de leituras para a subárvore que vem a seguir (no-op se
    /// não há rastreamento).
    fn push_frame(&self) {
        if let Some(r) = self.reads {
            r.push(self.depth);
        }
    }
}

/// Tenta reaproveitar do cache a subárvore desta posição: acerta quando **toda**
/// dependência guardada ainda tem o mesmo valor. Num acerto, empurra os nós
/// (clonados) em `out` e propaga as dependências para o quadro corrente — quem
/// reusa não lê nada, mas continua *dependendo* das mesmas chaves, e o ancestral
/// precisa saber disso.
fn reuse(ctx: &EvalCtx, cache: &mut EvalCache, out: &mut Vec<UiNode>) -> bool {
    let hit = cache
        .entries
        .get(&ctx.path)
        .filter(|e| ctx.deps_hold(&e.deps))
        .map(|e| (std::sync::Arc::clone(&e.deps), e.nodes.clone()));

    let Some((deps, nodes)) = hit else {
        return false;
    };
    if let Some(r) = ctx.reads {
        r.merge(&deps, ctx.depth, ctx);
    }
    out.extend(nodes.iter().cloned());
    cache.live.insert(ctx.path);
    true
}

/// Fecha o quadro de leituras aberto por [`EvalCtx::push_frame`] e guarda a
/// subárvore recém-avaliada com as dependências que ela declarou.
fn store(ctx: &EvalCtx, cache: &mut EvalCache, nodes: &[UiNode]) {
    let Some(reads) = ctx.reads else { return };
    let deps = reads.pop();
    cache.entries.insert(
        ctx.path,
        CacheEntry {
            deps: std::sync::Arc::new(deps),
            nodes: nodes.to_vec().into(),
        },
    );
    cache.live.insert(ctx.path);
}

/// Mistura um passo no hash de caminho (FNV-1a de 64 bits — suficiente para
/// identidade de instância, não é criptografia).
fn mix(path: u64, step: u64) -> u64 {
    let mut h = path ^ 0xcbf2_9ce4_8422_2325;
    for byte in step.to_le_bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// Um valor JSON como o contexto o guarda: a string crua quando é `String`, o
/// JSON serializado para todo o resto. É o que faz uma lista aninhada
/// atravessar a fronteira e voltar a ser lista num `for-each` de dentro — ele
/// reparseia o valor da chave.
fn json_scalar(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Monta a camada de variáveis de **um item** de `for-each`: `{var.campo}` para
/// cada campo de um objeto, ou `{var}` para um escalar. Devolve também a
/// identidade do item (o valor de `reorder_key`), de que o drag-and-drop precisa.
///
/// Substitui o antigo `context.clone()` + `insert` por item — ver [`EvalCtx`].
fn item_layer<'b>(
    item: &'b serde_json::Value,
    var: &'b str,
    reorder_key: Option<&str>,
    context: &EvalCtx<'b>,
) -> (Layer<'b>, Option<String>) {
    let mut layer = Layer::new(context.layer());

    // Só o campo do `reorder_key` é materializado na hora — o drag precisa da
    // identidade do item antes de qualquer leitura, e é UM campo, numa lista
    // que nem entra no cache.
    let this_key = match (item, reorder_key) {
        (serde_json::Value::Object(obj), Some(rk)) => obj.get(rk).map(json_scalar),
        _ => None,
    };

    layer.item = Some(ItemVars {
        var,
        campos: match item {
            serde_json::Value::Object(obj) => obj
                .iter()
                .map(|(k, v)| (k.as_str(), v, OnceCell::new()))
                .collect(),
            // Escalar não tem campo: só o `{var}` resolve, como sempre resolveu.
            _ => Vec::new(),
        },
        inteiro: (item, OnceCell::new()),
    });

    // Drag highlight: expõe se ESTE item é o que está sendo arrastado, para o
    // template poder estilizar a linha agarrada (ver `crate::DRAG_KEY_CONTEXT`).
    if let Some(key) = &this_key {
        let dragging = context.get(crate::DRAG_KEY_CONTEXT) == Some(key.as_str());
        layer.set(format!("{var}.__dragging"), dragging.to_string());
    }

    (layer, this_key)
}

/// Process string template by replacing `{key}` placeholders with values from context
pub fn process_template(template: &str, context: &ContextMap) -> String {
    process_tpl(template, &EvalCtx::new(context))
}

/// O `process_template` de verdade, sobre o [`EvalCtx`] (o público acima é a
/// casca para quem só tem um `HashMap` em mãos).
fn process_tpl(template: &str, context: &EvalCtx) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut key = String::new();
            let mut closed = false;
            while let Some(&nc) = chars.peek() {
                if nc == '}' {
                    chars.next(); // Consume '}'
                    closed = true;
                    break;
                } else {
                    key.push(chars.next().unwrap());
                }
            }
            if closed {
                // Inline default: `{key|default}` uses `default` (the text after
                // the first `|`) when `key` is absent from the context. Without a
                // `|` the behavior is unchanged: a missing key resolves to empty.
                // This is what lets a component default its own props per instance
                // without seeding — or polluting — the global context.
                let (lookup, default) = match key.split_once('|') {
                    Some((k, d)) => (k.trim(), Some(d.trim())),
                    None => (key.trim(), None),
                };
                if let Some(val) = context.get(lookup) {
                    result.push_str(val);
                } else if let Some(d) = default {
                    result.push_str(d);
                }
                // else: unknown key with no default -> empty (unchanged).
            } else {
                result.push('{');
                result.push_str(&key);
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Whether a (already-interpolated) string should be considered true.
fn is_truthy(s: &str) -> bool {
    matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on" | "sim"
    )
}

/// Evaluate an `<if>` condition against the context.
/// With `equals`/`not_equals` it compares strings; otherwise it is a truthy check.
#[allow(clippy::too_many_arguments)]
fn eval_condition(
    cond: &str,
    equals: Option<&str>,
    not_equals: Option<&str>,
    one_of: Option<&str>,
    contains: Option<&str>,
    empty: bool,
    not_empty: bool,
    context: &EvalCtx,
) -> bool {
    let value = process_tpl(cond, context);
    if let Some(eq) = equals {
        return value == process_tpl(eq, context);
    }
    if let Some(ne) = not_equals {
        return value != process_tpl(ne, context);
    }
    if let Some(list) = one_of {
        // Uma única interpolação sobre a lista inteira (não token a token) —
        // cobre tanto o caso comum (literal: `one_of="a b c"`) quanto uma
        // lista dinâmica vinda de uma var (`one_of="{allowed}"`), sem
        // inventar gramática de expressão nova.
        return process_tpl(list, context)
            .split_whitespace()
            .any(|tok| tok == value);
    }
    if let Some(item) = contains {
        // O **simétrico** do `one_of` acima: lá a lista está no markup e o
        // valor da chave é um item; aqui a lista está na chave
        // (`abertas="rede,proxy"`) e o item está no markup. Mesmo ponto do
        // eval, comparação invertida — e é isso que dá ao motor o conjunto
        // nomeado (`Accordion`, seleção múltipla, filtros por tag) sem estado
        // por instância.
        //
        // Os três separadores valem ao mesmo tempo porque quem monta o
        // conjunto é código de app: um `concat` com vírgula e um `table.concat`
        // com espaço são igualmente naturais, e escolher um só seria uma
        // pegadinha sem contrapartida.
        let alvo = process_tpl(item, context);
        let alvo = alvo.trim();
        return value
            .split([',', ';', ' ', '\t', '\n'])
            .map(str::trim)
            .any(|tok| !tok.is_empty() && tok == alvo);
    }
    if empty {
        return json_array_is_empty(&value);
    }
    if not_empty {
        return !json_array_is_empty(&value);
    }
    is_truthy(&value)
}

/// `cond` já é o JSON cru de uma lista no contexto (`ctx.proj_secrets =
/// "[...]"`) — usado por `empty`/`not_empty`. Não é um array JSON válido
/// (chave ainda não populada, JSON malformado) conta como vazio: é a
/// leitura honesta de "sem lista nenhuma ainda", o mesmo estado que um
/// `*_count` inexistente/"0" cobria.
fn json_array_is_empty(value: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(value) {
        Ok(serde_json::Value::Array(items)) => items.is_empty(),
        _ => true,
    }
}

/// The platform glacier-ui is compiled for — `"desktop"` for every target
/// that exists today (the engine only ever runs via native iced), `"web"`
/// once/if a browser (`wasm32`) target is added. Backs
/// `platform="desktop"`/`"web"` on any element (see the gate in
/// [`expand_children`]) — lets desktop-only chrome (borderless titlebar,
/// resize handles) and web-only chrome (PWA/service-worker bits) live in
/// the SAME template instead of forcing a whole second file just for that.
///
/// A compile-time `cfg`, not a runtime setting: nothing today builds
/// glacier-ui for `wasm32`, so this is `"desktop"` unconditionally in
/// practice — the `cfg` exists so the day a web target shows up, templates
/// written against `platform=` today start discriminating for free, no
/// migration.
pub fn current_platform() -> &'static str {
    #[cfg(target_arch = "wasm32")]
    {
        "web"
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "desktop"
    }
}

/// The stylesheets in effect during evaluation, split by scope.
///
/// `global` sheets apply everywhere: loaded via `GlacierUI::load_stylesheet`,
/// via a `<link rel="stylesheet">`, or an inline `<style>` block without
/// `scoped="true"` — all three land in the same set. `by_component` holds only
/// the sheets a component declared with `<style scoped="true">`, keyed by
/// component name; they apply only inside that component's subtree, layered
/// *on top of* the global ones so a scoped class can override a global one
/// locally.
pub struct StyleContext<'a> {
    pub global: &'a [StyleSheet],
    pub by_component: &'a HashMap<String, Vec<StyleSheet>>,
    /// Tamanho atual do viewport `(largura, altura)` em px lógicos, para avaliar
    /// blocos `@media`. `None` = sem info (nenhuma media query ativa).
    pub viewport: Option<(f32, f32)>,
    /// `true` se qualquer sheet ativo (global ou de escopo) declara seletor de
    /// **tag** — atalho para pular a resolução de estilo em nós sem `class`/`id`
    /// quando não há nenhuma regra de tag para casar (ver `eval_owned`).
    pub has_tag_rules: bool,
}

impl<'a> StyleContext<'a> {
    /// The ordered sheets that apply for the given component scope: global
    /// first (lowest priority), then that component's own scoped sheets.
    fn active(&self, scope: Option<&str>) -> Vec<&StyleSheet> {
        let mut sheets: Vec<&StyleSheet> = self.global.iter().collect();
        if let Some(name) = scope
            && let Some(scoped) = self.by_component.get(name)
        {
            sheets.extend(scoped.iter());
        }
        sheets
    }
}

/// Expands a sibling list of children into evaluated nodes, applying the
/// structural rules: `<if>`/`<else>` are resolved against the context (binding
/// `<else>` to the immediately preceding `<if>`), `<ForEach>` is unrolled over
/// its JSON array (re-expanding its own body so nested `if`/`else`/`ForEach`
/// work at any depth), and `<import>`/`<link>` are dropped. Everything else is
/// evaluated normally and pushed to `out`.
#[allow(clippy::too_many_arguments)]
fn expand_children(
    children: &[UiNode],
    context: &EvalCtx,
    templates: &HashMap<String, UiNode>,
    styles: &StyleContext,
    scope: Option<&str>,
    owner: Option<&str>,
    out: &mut Vec<UiNode>,
    // Repassado a cada filho para que um `<slot/>` a qualquer profundidade do
    // template (dentro de um `<if>`, de um `<Row>`, …) ainda o enxergue.
    slot: Option<&SlotContent>,
    cache: &mut EvalCache,
) -> Result<()> {
    // Tracks the result of the immediately preceding `<if>`, so an `<else>`
    // can bind to it. Reset by any other (non-else) node.
    let mut last_if: Option<bool> = None;
    for child in children {
        if matches!(
            child.kind,
            NodeType::Import { .. }
                | NodeType::Link { .. }
                | NodeType::Style { .. }
                | NodeType::Screen(_)
                | NodeType::ComponentRoot
                | NodeType::Resources
                | NodeType::Props(_)
                | NodeType::Prop
        ) {
            continue;
        }

        // 0. Platform gate — independent of if/else-if/else (doesn't touch
        // `last_if`, doesn't need a `cond`): `platform="desktop"`/`"web"` on
        // ANY element, alone or combined with another directive on the same
        // node. A mismatch makes the node vanish entirely, same treatment
        // as the `<import>`/`<link>`/`<style>` skip just above — so it's
        // checked here, before for-each/else/else-if/if even see the node.
        if let Some(platform) = child.if_platform()
            && platform != current_platform()
        {
            continue;
        }

        // 1. Process for-each attribute directive (outer precedence)
        if let Some(items) = child.for_each() {
            let var = child.for_each_var().unwrap_or("item");
            let items_evaluated = process_tpl(items, context);
            // Drag-and-drop: resolved once per for-each, reused by every item.
            let reorder_key = child.reorder_key().map(|s| process_tpl(s, context));
            let on_reorder = child
                .on_reorder()
                .map(|s| namespace_action(process_tpl(s, context), owner));
            if let Some(arr) = context
                .get(&items_evaluated)
                .and_then(|bruto| cache.array(&items_evaluated, bruto))
            {
                // Full identity snapshot, needed by the handle's `DragStart`.
                let full_order: Vec<String> = match &reorder_key {
                    Some(rk) => arr
                        .iter()
                        .filter_map(|item| item.get(rk).and_then(|v| v.as_str()).map(String::from))
                        .collect(),
                    None => Vec::new(),
                };
                // Uma lista reordenável NÃO entra no cache: o corpo de cada
                // item carrega `drag_order` — a ordem inteira da lista —
                // INJETADO por `hydrate_drag_item`, não lido do contexto.
                // Como o rastreamento só enxerga leituras, uma entrada de
                // cache não teria como perceber que a ordem mudou, e serviria
                // um item com a ordem velha. São listas pequenas (env vars);
                // reavaliá-las sempre não custa nada.
                let cacheable = on_reorder.is_none();

                for (index, item) in arr.iter().enumerate() {
                    // Variáveis do item numa CAMADA sobre o contexto, sem
                    // clonar a base (ver `EvalCtx`).
                    let (layer, this_key) = item_layer(item, var, reorder_key.as_deref(), context);
                    let item_ctx = context.with(&layer, mix(child.node_id, index as u64));

                    if cacheable && reuse(&item_ctx, cache, out) {
                        continue;
                    }

                    // Clone the child without the for_each directive
                    let mut clone = child.clone();
                    clone.set_for_each(None);
                    clone.set_for_each_var(None);
                    clone.set_on_reorder(None);
                    clone.set_reorder_key(None);

                    if let (Some(on_reorder), Some(key), Some(rk)) =
                        (&on_reorder, &this_key, &reorder_key)
                    {
                        hydrate_drag_item(
                            std::slice::from_mut(&mut clone),
                            &items_evaluated,
                            key,
                            &full_order,
                            on_reorder,
                            rk,
                        );
                    }

                    // Expand the single child in the new context (which will evaluate its if condition if present)
                    let mut item_out = Vec::new();
                    if cacheable {
                        item_ctx.push_frame();
                    }
                    expand_children(
                        std::slice::from_ref(&clone),
                        &item_ctx,
                        templates,
                        styles,
                        scope,
                        owner,
                        &mut item_out,
                        slot,
                        cache,
                    )?;
                    if cacheable {
                        store(&item_ctx, cache, &item_out);
                    }
                    out.extend(item_out);
                }
            }
            last_if = None;
            continue;
        }

        // 2. Process else attribute directive
        if child.is_else {
            if last_if == Some(false) {
                // Clone child and clear else directive
                let mut clone = child.clone();
                clone.is_else = false;
                out.push(eval_owned(
                    &clone, context, templates, styles, scope, owner, None, None, None, None, slot,
                    cache,
                )?);
            }
            last_if = None;
            continue;
        }

        // 2.5. Process else-if attribute directive — chains off the SAME
        // `last_if` an `<if>`/`<else-if>` before it left behind. Only
        // evaluates its own condition when the chain is still open
        // (`last_if == Some(false)`); once something upstream matched
        // (`Some(true)`), every further `else-if`/`else` in the chain is
        // skipped without even reading its condition — same short-circuit an
        // `if/else if/else` chain has in any imperative language. A stray
        // `else-if` with no `if` before it (`last_if == None`) is likewise a
        // no-op, matching the defensive behaviour of a stray `else` above.
        if let Some(cond) = child.else_if_cond() {
            if last_if == Some(false) {
                let truthy = eval_condition(
                    cond,
                    child.if_equals(),
                    child.if_not_equals(),
                    child.if_one_of(),
                    child.if_contains(),
                    child.if_empty,
                    child.if_not_empty,
                    context,
                );
                if truthy {
                    let mut clone = child.clone();
                    clone.set_else_if_cond(None);
                    clone.set_if_equals(None);
                    clone.set_if_not_equals(None);
                    clone.set_if_one_of(None);
                    clone.set_if_contains(None);
                    clone.if_empty = false;
                    clone.if_not_empty = false;
                    out.push(eval_owned(
                        &clone, context, templates, styles, scope, owner, None, None, None, None,
                        slot, cache,
                    )?);
                }
                last_if = Some(truthy);
            }
            continue;
        }

        // 3. Process if attribute directive
        if let Some(cond) = child.if_cond() {
            let truthy = eval_condition(
                cond,
                child.if_equals(),
                child.if_not_equals(),
                child.if_one_of(),
                child.if_contains(),
                child.if_empty,
                child.if_not_empty,
                context,
            );
            if truthy {
                // Clone child and clear if directives
                let mut clone = child.clone();
                clone.set_if_cond(None);
                clone.set_if_equals(None);
                clone.set_if_not_equals(None);
                clone.set_if_one_of(None);
                clone.set_if_contains(None);
                clone.if_empty = false;
                clone.if_not_empty = false;
                out.push(eval_owned(
                    &clone, context, templates, styles, scope, owner, None, None, None, None, slot,
                    cache,
                )?);
            }
            last_if = Some(truthy);
            continue;
        }

        // 4. Fallback to legacy tag-based conditionals/loops
        match &child.kind {
            // `<import>`/`<link>`/`<style>` declarations are skipped above.
            NodeType::Import { .. }
            | NodeType::Link { .. }
            | NodeType::Style { .. }
            | NodeType::Screen(_)
            | NodeType::ComponentRoot
            | NodeType::Resources
            | NodeType::Props(_)
            | NodeType::Prop => {}
            NodeType::ForEach { items, var } => {
                let items_evaluated = process_tpl(items, context);
                // Drag-and-drop: `onReorder`/`reorderKey` on the `<ForEach>` tag
                // itself (a plain node attribute, same as `onPress`/`cursor`).
                let reorder_key = child.reorder_key().map(|s| process_tpl(s, context));
                let on_reorder = child
                    .on_reorder()
                    .map(|s| namespace_action(process_tpl(s, context), owner));
                if let Some(arr) = context
                    .get(&items_evaluated)
                    .and_then(|bruto| cache.array(&items_evaluated, bruto))
                {
                    let full_order: Vec<String> = match &reorder_key {
                        Some(rk) => arr
                            .iter()
                            .filter_map(|item| {
                                item.get(rk).and_then(|v| v.as_str()).map(String::from)
                            })
                            .collect(),
                        None => Vec::new(),
                    };
                    // Ver o porquê no `for-each` de atributo, acima.
                    let cacheable = on_reorder.is_none();

                    for (index, item) in arr.iter().enumerate() {
                        // Variáveis do item numa CAMADA sobre o contexto, sem
                        // clonar a base (ver `EvalCtx`).
                        let (layer, this_key) =
                            item_layer(item, var, reorder_key.as_deref(), context);
                        let item_ctx = context.with(&layer, mix(child.node_id, index as u64));

                        if cacheable && reuse(&item_ctx, cache, out) {
                            continue;
                        }

                        // The `<ForEach>` tag's body isn't a single node like
                        // the attribute form's — clone its children so the
                        // hydration below has somewhere of its own to live.
                        let mut body: Vec<UiNode> = child.children.to_vec();
                        if let (Some(on_reorder), Some(key), Some(rk)) =
                            (&on_reorder, &this_key, &reorder_key)
                        {
                            hydrate_drag_item(
                                &mut body,
                                &items_evaluated,
                                key,
                                &full_order,
                                on_reorder,
                                rk,
                            );
                        }
                        // Re-run the structural expansion on the body so that
                        // nested `if`/`else`/`ForEach` are honoured per item.
                        let mut item_out = Vec::new();
                        if cacheable {
                            item_ctx.push_frame();
                        }
                        expand_children(
                            &body,
                            &item_ctx,
                            templates,
                            styles,
                            scope,
                            owner,
                            &mut item_out,
                            slot,
                            cache,
                        )?;
                        if cacheable {
                            store(&item_ctx, cache, &item_out);
                        }
                        out.extend(item_out);
                    }
                }
                last_if = None;
            }
            NodeType::If {
                cond,
                equals,
                not_equals,
                one_of,
                contains,
                empty,
                not_empty,
            } => {
                let truthy = eval_condition(
                    cond,
                    equals.as_deref(),
                    not_equals.as_deref(),
                    one_of.as_deref(),
                    contains.as_deref(),
                    *empty,
                    *not_empty,
                    context,
                );
                if truthy {
                    expand_children(
                        &child.children,
                        context,
                        templates,
                        styles,
                        scope,
                        owner,
                        out,
                        slot,
                        cache,
                    )?;
                }
                last_if = Some(truthy);
            }
            NodeType::ElseIf {
                cond,
                equals,
                not_equals,
                one_of,
                contains,
                empty,
                not_empty,
            } => {
                // Same short-circuit as the attribute form (`else-if="…"`
                // above): only rolls its own condition when the chain is
                // still open; once something upstream matched, `last_if`
                // stays `Some(true)` and every further branch is skipped.
                if last_if == Some(false) {
                    let truthy = eval_condition(
                        cond,
                        equals.as_deref(),
                        not_equals.as_deref(),
                        one_of.as_deref(),
                        contains.as_deref(),
                        *empty,
                        *not_empty,
                        context,
                    );
                    if truthy {
                        expand_children(
                            &child.children,
                            context,
                            templates,
                            styles,
                            scope,
                            owner,
                            out,
                            slot,
                            cache,
                        )?;
                    }
                    last_if = Some(truthy);
                }
            }
            NodeType::Else => {
                if last_if == Some(false) {
                    expand_children(
                        &child.children,
                        context,
                        templates,
                        styles,
                        scope,
                        owner,
                        out,
                        slot,
                        cache,
                    )?;
                }
                last_if = None;
            }
            _ => {
                let n = eval_owned(
                    child, context, templates, styles, scope, owner, None, None, None, None, slot,
                    cache,
                )?;
                // A `Fragment` (a multi-root component template, or an explicit
                // `Fragment { … }`) is transparent: splice its already-evaluated
                // children into this list instead of pushing a wrapper node, so
                // e.g. a component that is an `if`/`else` pair renders as two
                // siblings of the surrounding layout.
                if matches!(n.kind, NodeType::Fragment) {
                    out.extend(n.children.into_vec());
                } else {
                    out.push(n);
                }
                last_if = None;
            }
        }
    }
    Ok(())
}

/// Recursively evaluate a UiNode tree, resolving templates and placeholders.
///
/// `styles` are the loaded `.gss` documents; any `class="..."` on a node is
/// resolved against them and merged underneath the node's inline attributes.
/// `scope` is the name of the component being evaluated, used to pick up its
/// `<link>`-scoped stylesheets.
pub fn evaluate_node(
    node: &UiNode,
    context: &ContextMap,
    templates: &HashMap<String, UiNode>,
    styles: &StyleContext,
    scope: Option<&str>,
) -> Result<UiNode> {
    // A fronteira: o motor tem um `HashMap`; a avaliação por dentro trabalha
    // sobre o [`EvalCtx`] em camadas, para não clonar a base por item de lista.
    // Sem cache nem rastreamento — é a avaliação avulsa, para quem só quer a
    // árvore uma vez. O motor usa [`evaluate_template`].
    let ctx = EvalCtx::new(context);
    let mut cache = EvalCache::default();
    eval_owned(
        node, &ctx, templates, styles, scope, None, None, None, None, None, None, &mut cache,
    )
}

/// Avalia um template **rastreando** as chaves de contexto que ele lê e
/// reaproveitando de `cache` as subárvores cujas dependências não mudaram.
///
/// Devolve a árvore e o conjunto de dependências dela — que é o que permite ao
/// motor responder, na próxima mudança de contexto, a pergunta que interessa:
/// *"isto que mudou é lido por esta tela?"*. Se não for, não há o que
/// reconstruir. Ver [`crate::GlacierUI::reevaluate_all`].
pub fn evaluate_template(
    node: &UiNode,
    context: &ContextMap,
    templates: &HashMap<String, UiNode>,
    styles: &StyleContext,
    scope: Option<&str>,
    cache: &mut EvalCache,
) -> Result<(UiNode, Deps)> {
    let reads = Reads::default();
    reads.push(0);
    let ctx = EvalCtx::tracked(context, &reads);
    let tree = eval_owned(
        node, &ctx, templates, styles, scope, None, None, None, None, None, None, cache,
    )?;
    let deps = reads.pop();
    // Entradas de subárvores que sumiram (uma linha removida da lista) viram
    // lixo; varrer aqui mantém o cache do tamanho da tela, não do histórico.
    cache.sweep();
    Ok((tree, deps))
}

/// Prefixes an action with its owning component, so `dispatch` can route it.
/// Actions inside a `<Component name="X">` subtree become `X::action`.
/// Empty actions and navigation are left untouched.
/// Prefixos de ações built-in tratadas pelo próprio motor (`dispatch`) antes de
/// qualquer roteamento a componente — ver `GlacierUI::dispatch`. São globais, não
/// pertencem a componente algum, então **não** podem ser namespaceadas: senão o
/// `strip_prefix("clipboard:")`/`"open:"`/`"window:"` erra dentro de um
/// componente importado (ex.: `ServiceDetail::clipboard:foo`).
const BUILTIN_ACTION_PREFIXES: [&str; 4] = ["clipboard:", "open:", "window:", "style:"];

/// Marca uma ação como **do aplicativo**, não do componente que a escreveu:
/// `app:` é removido no lugar do prefixo de dono, então a ação sai "nua" da
/// avaliação e o `dispatch` a entrega à tela atual.
///
/// Existe para o caso do **widget composto que delega**: um builtin como o
/// `<TimePicker/>` recebe `on_pick="abrir_modal"` por prop e repassa ao
/// `<Button/>` interno. Sem escape, o `namespace_action` prefixaria com o dono
/// (`TimePicker::abrir_modal`), o `dispatch` acharia o `TimePicker` no mapa de
/// componentes e chamaria o `update` **dele** — que não conhece ação nenhuma do
/// app. O handler do app nunca rodava, sem erro nenhum: o botão simplesmente
/// não fazia nada. Com `on_click="app:{on_pick}"` a ação volta a ser
/// `abrir_modal` e chega em quem a definiu.
///
/// Escopo: "do app" quer dizer **a tela atual** (é onde o `dispatch` cai quando
/// não há dono), não o componente intermediário que porventura tenha usado o
/// widget. Um componente que delega para outro componente ainda depende de um
/// `ctx.dispatch` no motor, que não existe.
pub const APP_ACTION_PREFIX: &str = "app:";

fn namespace_action(action: String, owner: Option<&str>) -> String {
    // O escape vem antes de tudo: quem escreveu `app:` está dizendo que a ação
    // não é dele, então nem o dono nem os prefixos built-in se aplicam.
    if let Some(bare) = action.strip_prefix(APP_ACTION_PREFIX) {
        return bare.trim().to_string();
    }
    match owner {
        Some(name)
            if !action.is_empty()
                && !BUILTIN_ACTION_PREFIXES
                    .iter()
                    .any(|p| action.starts_with(p)) =>
        {
            format!("{}::{}", name, action)
        }
        _ => action,
    }
}

/// Teto de aninhamento da expansão (componentes + itens de `for-each`, que
/// compartilham o mesmo contador de profundidade). Ver a guarda em `eval_owned`.
///
/// O número é baixo de propósito. `eval_owned` é uma função grande, com muitos
/// locais gordos (duas `StyleRule`, dezenas de `Option<String>`, vários `Vec`),
/// e cada nível de componente empilha um quadro dele **mais** um de
/// `expand_children`. Com 128, a recursão infinita ainda estourava a pilha de
/// 2 MiB de uma thread de teste antes de a guarda disparar — o teto tem de
/// caber na pilha, não só existir.
///
/// 16 continua folgado para markup honesto: profundidade aqui é
/// **aninhamento**, não contagem — um `for-each` de 500 itens abre um nível, não
/// 500. Uma tela densa de verdade chega a uma dúzia.
///
/// Este teto é só a rede de segurança para **ciclos indiretos**; a
/// auto-referência direta (o caso real) é pega por nome, no primeiro nível, sem
/// gastar pilha nenhuma.
const PROFUNDIDADE_MAXIMA: u32 = 16;

/// O conteúdo que um uso de componente escreveu entre as tags, **já avaliado**
/// e repartido pelo destino: o balde anônimo (o que ninguém etiquetou) e um por
/// `slot="nome"`.
///
/// Existe porque um único `Vec` não bastava a partir do momento em que um
/// widget passou a ter mais de uma região — o rodapé de um `<card>`, as ações
/// no cabeçalho de um `<groupbox>`. A partição acontece uma vez, na fronteira
/// do componente, sobre os filhos **crus** (é neles que o atributo `slot`
/// ainda existe); cada balde é expandido no contexto e com o dono de quem
/// escreveu, exatamente como antes.
#[derive(Default)]
pub(crate) struct SlotContent {
    /// O que não foi etiquetado — o que um `<slot/>` sem `name` recebe.
    anonimo: Vec<UiNode>,
    /// `nome -> conteúdo`. Vec de pares, não mapa: são dois ou três slots por
    /// widget, e a busca linear numa lista desse tamanho é mais barata que o
    /// hash (além de preservar a ordem em que o uso escreveu).
    nomeados: Vec<(String, Vec<UiNode>)>,
}

impl SlotContent {
    /// O conteúdo de um slot, ou `None` se o uso não preencheu esse destino —
    /// caso em que o `<slot>` cai no conteúdo de reserva dele.
    fn get(&self, name: Option<&str>) -> Option<&[UiNode]> {
        let bucket = match name {
            None => &self.anonimo,
            Some(n) => &self.nomeados.iter().find(|(k, _)| k == n).map(|(_, v)| v)?[..],
        };
        (!bucket.is_empty()).then_some(bucket)
    }

    fn is_empty(&self) -> bool {
        self.anonimo.is_empty() && self.nomeados.iter().all(|(_, v)| v.is_empty())
    }
}

/// Core of [`evaluate_node`]. `owner` is the name of the nearest enclosing
/// `<Component>`/`<Include>` reference, used to namespace its actions. `scope`
/// is the component whose `<link>`-scoped stylesheets are currently in effect
/// (it follows the same component boundaries as `owner`).
#[allow(clippy::too_many_arguments)]
fn eval_owned(
    node: &UiNode,
    context: &EvalCtx,
    templates: &HashMap<String, UiNode>,
    styles: &StyleContext,
    scope: Option<&str>,
    owner: Option<&str>,
    // Underlay de **tag-de-componente** (`Card {}`), passado só para a raiz
    // avaliada do template de um componente: entra como o tier de MENOR
    // especificidade (abaixo de tag builtin/classe/id/inline). `None` no caso
    // comum. Aninhamento: o componente interno recebe o do externo já mesclado.
    underlay: Option<&StyleRule>,
    underlay_states: Option<&StateStyles>,
    // Overlay de **classe/id escritos no USO** de um componente, passado só para
    // a raiz avaliada do template dele. Gêmeo do `underlay` acima, no outro
    // extremo: entra ACIMA da classe do template e ABAIXO dos atributos inline
    // dele. A regra em uma frase: a classe escrita no uso vence as classes do
    // template, e perde para o que o template cravou inline. `None` no caso
    // comum. Ver `PLANO_CLASS_EM_COMPONENTE.md`.
    overlay: Option<&StyleRule>,
    overlay_states: Option<&StateStyles>,
    // Conteúdo escrito entre as tags do componente que está sendo expandido —
    // **já avaliado**, no contexto e com o dono de QUEM USOU, repartido por
    // destino. É o que um `<slot/>` no template devolve. `None` fora de um
    // componente. Ver [`SlotContent`] e [`crate::parser::NodeType::Slot`].
    slot: Option<&SlotContent>,
    cache: &mut EvalCache,
) -> Result<UiNode> {
    // `<slot/>`: devolve o conteúdo do uso (ou, se ele não veio, o conteúdo de
    // reserva escrito dentro do próprio `<slot>`) embrulhado num `Fragment`,
    // que `expand_children` espalha na lista do pai. Vem antes de tudo porque
    // o conteúdo já foi avaliado — reavaliá-lo aqui o namespaçaria de novo,
    // desta vez com o dono errado (o componente, não quem chamou).
    if let NodeType::Slot { name } = &node.kind {
        let conteudo: Vec<UiNode> = match slot.and_then(|s| s.get(name.as_deref())) {
            Some(nodes) => nodes.to_vec(),
            // Reserva: os filhos do `<slot>` são do componente, então avaliam
            // no contexto dele — inclusive enxergando as props da instância.
            _ => {
                let mut reserva = Vec::new();
                expand_children(
                    &node.children,
                    context,
                    templates,
                    styles,
                    scope,
                    owner,
                    &mut reserva,
                    None,
                    cache,
                )?;
                reserva
            }
        };
        return Ok(crate::parser::empty_node(NodeType::Fragment, conteudo));
    }
    // A component reference — either the legacy `<Include src="..." />` or a tag
    // named after a registered component (e.g. `<PerfilCard ... />`) — is replaced
    // with the evaluated template root, with its attributes passed in as props.
    let reference: Option<(&String, &ContextMap)> = match &node.kind {
        NodeType::Include { src, props } => Some((src, props)),
        NodeType::Component { name, props } => Some((name, props)),
        _ => None,
    };
    if let Some((name, props)) = reference {
        // Guarda de recursão, em duas camadas. Sem ela, um componente que se
        // referencia estoura a pilha — `SIGABRT`, sem mensagem nem nome.
        //
        // 1. **Auto-referência direta**, o caso que de fato acontece: o dono da
        //    subárvore que está sendo avaliada é o próprio componente que a tag
        //    invoca. Pega no primeiro nível, sem gastar pilha, e é exato — não
        //    depende de teto nenhum.
        if owner == Some(name.as_str()) {
            return Err(crate::error::GlacierError::ComponentRecursion {
                name: name.clone(),
                limite: PROFUNDIDADE_MAXIMA,
            });
        }
        // 2. **Ciclo indireto** (A usa B, B usa A) e aninhamento absurdo: teto de
        //    profundidade, como rede de segurança.
        if context.depth() >= PROFUNDIDADE_MAXIMA {
            return Err(crate::error::GlacierError::ComponentRecursion {
                name: name.clone(),
                limite: PROFUNDIDADE_MAXIMA,
            });
        }
        let template_ast = templates
            .get(name)
            .ok_or_else(|| crate::error::GlacierError::UnknownComponent(name.clone()))?;

        // O conteúdo entre as tags (`<GroupBox>ISTO</GroupBox>`) é avaliado
        // AQUI, ainda no contexto e com o dono de quem escreveu — antes de
        // qualquer camada de props entrar em cena. É o que garante que
        // `<GroupBox><Button on_click="salvar"/></GroupBox>` despache `salvar`
        // para a tela, e não `GroupBox::salvar` para o `update` do builtin.
        //
        // Nada de `slot` do nível de fora atravessa: um `<slot/>` escrito no
        // uso pertence ao componente que envolve ESTE uso, e já foi resolvido
        // pela chamada que nos trouxe até aqui.
        //
        // A partição por destino roda sobre os filhos **crus**, porque é neles
        // que o atributo `slot="footer"` ainda existe (a avaliação o consome).
        // Cada balde é expandido por sua conta, o que preserva a semântica de
        // um `<if>`/`<for-each>` dentro de um slot nomeado.
        let mut slot_conteudo = SlotContent::default();
        if !node.children.is_empty() {
            let mut baldes: Vec<(Option<String>, Vec<&UiNode>)> = Vec::new();
            for filho in &node.children {
                let destino = filho.slot_name().map(str::to_string);
                match baldes.iter_mut().find(|(k, _)| *k == destino) {
                    Some((_, v)) => v.push(filho),
                    None => baldes.push((destino, vec![filho])),
                }
            }
            for (destino, filhos) in baldes {
                let crus: Vec<UiNode> = filhos
                    .into_iter()
                    .map(|f| {
                        // A diretiva já foi consumida pela partição; deixá-la
                        // no clone faria um `<card slot="footer">` aninhado
                        // reetiquetar o conteúdo do componente de dentro.
                        let mut c = f.clone();
                        c.set_slot_name(None);
                        c
                    })
                    .collect();
                let mut saida = Vec::new();
                expand_children(
                    &crus, context, templates, styles, scope, owner, &mut saida, None, cache,
                )?;
                match destino {
                    None => slot_conteudo.anonimo = saida,
                    Some(nome) => slot_conteudo.nomeados.push((nome, saida)),
                }
            }
        }

        // Contrato do componente, quando ele declara um (`<props>` no cabeçalho).
        // Declarar é opcional; a partir do momento em que existe, ele é a
        // verdade — ver `PropDecl`.
        let declaradas = template_ast.children.iter().find_map(|c| match &c.kind {
            NodeType::Props(p) => Some(p),
            _ => None,
        });

        // As props do componente entram numa CAMADA sobre o contexto do uso (que
        // o template do componente enxerga por baixo), sem clonar a base — ver
        // [`EvalCtx`]. Uma prop de mesmo nome que uma chave global a sombreia,
        // como antes.
        let mut layer = Layer::new(context.layer());

        // `spread="{c}"`: o objeto inteiro no lugar de um atributo por campo.
        // Existe porque o call-site de um card em lista era uma parede de
        // mapeamentos IDENTIDADE (`id="{c.id}" nome="{c.nome}" …`) — ruído de
        // digitação, sem informação nenhuma.
        //
        // O que ele deliberadamente NÃO faz é virar uma prop-objeto (`{card.id}`
        // dentro do componente): ali o `<props>` passaria a declarar `card` e
        // mais nada, e `{card.nmae}` voltaria a renderizar vazio em silêncio —
        // o typo invisível que o contrato existe para fechar. Semeando as props
        // DECLARADAS, o dentro do componente não muda (`{id}`, `{nome}`), todas
        // seguem verificadas, e a checagem ainda ganha alcance: uma obrigatória
        // que o DADO não trouxe passa a errar, não só a que o markup esqueceu.
        let spread_raw = props
            .iter()
            .find(|(k, _)| crate::parser::SPREAD_ATTRS.contains(&k.as_str()))
            .map(|(_, v)| process_tpl(v, context));
        let spread: serde_json::Map<String, serde_json::Value> = match spread_raw.as_deref() {
            // Vazio é "nenhum campo", não um erro: a chave ainda não carregou.
            // Se alguma prop obrigatória depender dela, o `MissingProp` abaixo
            // erra apontando QUAL — melhor mensagem do que um spread inválido.
            None | Some("") => serde_json::Map::new(),
            Some(raw) => match serde_json::from_str(raw.trim()) {
                Ok(serde_json::Value::Object(obj)) => obj,
                _ => {
                    return Err(crate::error::GlacierError::InvalidSpread {
                        component: name.clone(),
                        value: raw.to_string(),
                    });
                }
            },
        };

        if let Some(declaradas) = declaradas {
            // Prop passada que não existe no contrato: é aqui que um typo para
            // de ser invisível. Sem isto, `labl="CPU"` não casa com nada, o
            // `{label}` do template atravessa a camada e pega o `label` do
            // contexto de baixo — renderizando um valor plausível e errado.
            for chave in props.keys() {
                // As diretivas (`for-each`, `var`, `if`…) chegam no mesmo mapa,
                // porque `from_node` encaminha TODO atributo como prop — mas
                // quem as lê é o `expand_children`, antes daqui. Ver
                // `parser::DIRECTIVE_ATTRS`.
                if crate::parser::DIRECTIVE_ATTRS.contains(&chave.as_str())
                    || crate::parser::SPREAD_ATTRS.contains(&chave.as_str())
                {
                    continue;
                }
                if !declaradas.iter().any(|d| &d.name == chave) {
                    return Err(crate::error::GlacierError::UnknownProp {
                        component: name.clone(),
                        prop: chave.clone(),
                        declaradas: declaradas.iter().map(|d| d.name.clone()).collect(),
                    });
                }
            }
            // O caminho inverso: prop declarada sem `default` é obrigatória.
            // Com `default`, ele é semeado na camada — o que também impede a
            // queda para o contexto de baixo quando quem chama omite a prop.
            for decl in declaradas {
                if props.contains_key(&decl.name) {
                    continue;
                }
                // O spread entra ANTES do default, e depois do atributo escrito
                // à mão (que o laço final sobrescreve por cima): escrever a prop
                // explicitamente ao lado de um `spread` é como se sobrepõe um
                // campo do objeto.
                if let Some(v) = spread.get(&decl.name) {
                    layer.set(decl.name.clone(), json_scalar(v));
                    continue;
                }
                match &decl.default {
                    Some(v) => layer.set(decl.name.clone(), process_tpl(v, context)),
                    None => {
                        return Err(crate::error::GlacierError::MissingProp {
                            component: name.clone(),
                            prop: decl.name.clone(),
                        });
                    }
                }
            }
        } else {
            // Sem `<props>` não há contrato, e portanto não há o que filtrar:
            // todo campo do objeto entra, do mesmo jeito que o `item_layer` faz
            // com um item de `for-each`. Mantém valendo a regra da 0.61 — quem
            // não declara não é checado.
            for (key, val) in &spread {
                layer.set(key.clone(), json_scalar(val));
            }
        }
        for (key, val_template) in props {
            // O spread não é uma prop: deixá-lo entrar sombrearia uma chave de
            // mesmo nome do contexto de baixo com um blob de JSON.
            if crate::parser::SPREAD_ATTRS.contains(&key.as_str()) {
                continue;
            }
            layer.set(key.clone(), process_tpl(val_template, context));
        }

        // `{slot_footer}` = "true" quando o uso preencheu `slot="footer"`.
        //
        // Sem isto, um widget não consegue **decorar** um slot opcional: o
        // `<card>` quer uma linha divisória acima do rodapé só quando existe
        // rodapé, e o template não tem como perguntar isso — o nome do slot não
        // é uma prop, e o conteúdo dele nem chega ao interpolador. O marcador é
        // a resposta mínima: um booleano por slot nomeado preenchido, que o
        // `<template if>` já sabe ler.
        //
        // Entra DEPOIS das props de propósito: uma prop escrita à mão com o
        // mesmo nome vence, em vez de o motor sobrescrever o que o app pediu.
        for (nome, conteudo) in &slot_conteudo.nomeados {
            if !conteudo.is_empty() {
                layer.set(format!("slot_{nome}"), "true".to_string());
            }
        }
        // Classe/id escritos NO USO (`<spinbox class="campo_num"/>`). Resolvidos
        // aqui, no escopo de QUEM USOU — é lá que a folha com `.campo_num` mora,
        // não no escopo do componente — e entregues à raiz do template como
        // `overlay`. Sem isto, `class` numa tag de componente era lida pelo
        // parser, viajava no mapa de props e não pintava nada: falha silenciosa.
        // Ver `PLANO_CLASS_EM_COMPONENTE.md`.
        //
        // A interpolação acontece contra o contexto de FORA (`context`), então
        // um `class="{estado}"` registra a leitura no quadro de quem chamou,
        // que é a quem a dependência de fato pertence.
        let uso_class = node
            .class
            .as_deref()
            .map(|c| process_tpl(c, context))
            .unwrap_or_default();
        let uso_id = node.id.as_deref().map(|i| process_tpl(i, context));

        // A classe do uso entra na CHAVE do cache. O cache de componente é
        // indexado pelo caminho (derivado do `node_id`), e as dependências que
        // ele guarda são as lidas DENTRO da expansão — a leitura de `{estado}`
        // acima ficou no quadro de fora e não estaria entre elas. Sem misturar
        // o valor resolvido aqui, um `class="{estado}"` que mudasse serviria a
        // árvore antiga, com o estilo velho, para sempre. Mesma armadilha que
        // tirou o uso COM conteúdo de slot do cache na 0.65 — aqui, porém, dá
        // para manter o cache: basta que valores diferentes ocupem entradas
        // diferentes.
        let mut assinatura_estilo = 0u64;
        if !uso_class.is_empty() || uso_id.is_some() {
            for b in uso_class
                .bytes()
                .chain(uso_id.iter().flat_map(|i| i.bytes()))
            {
                assinatura_estilo ^= b as u64;
                assinatura_estilo = assinatura_estilo.wrapping_mul(0x100_0000_01b3);
            }
        }
        let local_context = context.with(&layer, mix(node.node_id, assinatura_estilo));

        // O uso de um componente é uma fronteira natural de cache: é uma
        // subárvore inteira com uma entrada de dados bem definida (as props). É
        // o que faz uma linha de log nova não reconstruir a sidebar — cada
        // `<NavItem/>` dela é um componente cujas props não mudaram.
        //
        // **Exceção do `<slot/>`:** um uso COM conteúdo fica de fora do cache.
        // A entrada é chaveada pelas dependências lidas dentro da expansão, e
        // o conteúdo do slot foi avaliado do lado de fora — suas leituras
        // pertencem ao quadro de quem chamou. Uma entrada aqui não teria como
        // perceber que o conteúdo mudou e serviria a versão velha. Mesmo
        // raciocínio (e mesmo custo desprezível: são os containers da tela) da
        // lista reordenável em `expand_children`.
        let mut reused = Vec::new();
        if slot_conteudo.is_empty() && reuse(&local_context, cache, &mut reused) {
            // O cache guarda uma lista de nós; um componente sempre rende
            // exatamente um (a raiz avaliada do seu template).
            if let Some(root) = reused.pop() {
                return Ok(root);
            }
        }
        local_context.push_frame();

        // Underlay de tag-de-componente: `Card {}` (minúsculo) casa o *nome* do
        // componente no seu uso. Como o componente é inlinado, o estilo é
        // resolvido aqui (sheets do escopo do USO) e passado como underlay de
        // menor especificidade para a raiz avaliada do template. Herda o
        // underlay do componente externo (aninhamento), com este por cima.
        let mut underlay_rule = underlay.cloned().unwrap_or_default();
        let mut underlay_st = underlay_states.cloned().unwrap_or_default();
        if styles.has_tag_rules {
            let active = styles.active(scope);
            let tag = name.to_lowercase();
            underlay_rule.merge_from(&resolve_classes(
                Some(&tag),
                "",
                None,
                &active,
                styles.viewport,
            ));
            underlay_st.merge_from(&resolve_state_classes(
                Some(&tag),
                "",
                None,
                &active,
                styles.viewport,
            ));
        }

        // Overlay: a classe/id do uso, resolvida no escopo do uso (`scope`), no
        // outro extremo da escada de especificidade em relação ao underlay
        // acima. Só custa a busca quando alguém de fato escreveu uma.
        let (overlay_rule, overlay_st) = if uso_class.is_empty() && uso_id.is_none() {
            (None, None)
        } else {
            let active = styles.active(scope);
            (
                Some(resolve_classes(
                    None,
                    &uso_class,
                    uso_id.as_deref(),
                    &active,
                    styles.viewport,
                )),
                Some(resolve_state_classes(
                    None,
                    &uso_class,
                    uso_id.as_deref(),
                    &active,
                    styles.viewport,
                )),
            )
        };

        // The referenced subtree's actions and scoped styles belong to `name`
        // (innermost wins).
        let root = eval_owned(
            template_ast,
            &local_context,
            templates,
            styles,
            Some(name),
            Some(name),
            Some(&underlay_rule),
            Some(&underlay_st),
            overlay_rule.as_ref(),
            overlay_st.as_ref(),
            (!slot_conteudo.is_empty()).then_some(&slot_conteudo),
            cache,
        )?;
        if slot_conteudo.is_empty() {
            store(&local_context, cache, std::slice::from_ref(&root));
        }
        return Ok(root);
    }

    // Resolve `class="..."` into a merged style rule that sits *underneath* the
    // node's inline attributes (inline wins, per CSS precedence). Global sheets
    // apply first, then the current component's scoped sheets. Pseudo-state
    // overlays (`.classe:hover { }` etc.) are resolved alongside the base rule
    // from the very same class list/sheets/viewport, so they stay consistent.
    // Style resolution, by ascending specificity (each overriding the previous):
    //   component-tag underlay  <  builtin-tag  <  class  <  id  <  inline
    // The underlay (from an enclosing `<Card/>`, if any) is the base; the tag
    // (this node's builtin kind), classes and id are merged on top by
    // `resolve_classes`; inline attrs win last, in the per-field match below.
    // `class`/`id` are interpolated (`id="item-{i}"` works). The `styles.active`
    // allocation is skipped for a plain node unless a tag rule is in play.
    let (style, state_styles): (StyleRule, StateStyles) = {
        let mut base = underlay.cloned().unwrap_or_default();
        let mut states = underlay_states.cloned().unwrap_or_default();
        let tag = node.kind.tag_name();
        let needs_lookup =
            node.class.is_some() || node.id.is_some() || (tag.is_some() && styles.has_tag_rules);
        if needs_lookup {
            let active = styles.active(scope);
            let processed = node
                .class
                .as_deref()
                .map(|c| process_tpl(c, context))
                .unwrap_or_default();
            let id = node.id.as_deref().map(|i| process_tpl(i, context));
            base.merge_from(&resolve_classes(
                tag,
                &processed,
                id.as_deref(),
                &active,
                styles.viewport,
            ));
            states.merge_from(&resolve_state_classes(
                tag,
                &processed,
                id.as_deref(),
                &active,
                styles.viewport,
            ));
        }
        // O overlay entra por ÚLTIMO entre os tiers de folha — depois da classe
        // e do id deste nó — porque ele é a classe que quem USOU o componente
        // escreveu, e ela precisa poder redefinir o que o template deixou como
        // padrão. Os atributos inline do template ainda vencem: eles são
        // aplicados no `match` por campo mais abaixo, sobre este resultado.
        if let Some(o) = overlay {
            base.merge_from(o);
        }
        if let Some(o) = overlay_states {
            states.merge_from(o);
        }
        (base, states)
    };

    // Resolve a numeric attribute whose XML value was a `{...}` template (see
    // `NumAttr`): interpolate against the context and parse to f32. `None` if
    // the node had no template for `attr`, or it resolved to a non-number.
    let num_template = |attr: NumAttr| -> Option<f32> {
        node.numeric_templates
            .iter()
            .find(|(a, _)| *a == attr)
            .and_then(|(_, t)| process_tpl(t, context).trim().parse::<f32>().ok())
    };

    // O mesmo para `hidden`/`disabled` escritos com placeholder (ver
    // `BoolAttr`): interpola e aplica o mesmo teste de verdade do `if`
    // (`true`/`1`/`yes`/`on`/`sim`), para os dois concordarem sobre o que é
    // "ligado". `None` se o nó não tinha template para `attr`.
    let bool_template = |attr: BoolAttr| -> Option<bool> {
        node.bool_templates
            .iter()
            .find(|(a, _)| *a == attr)
            .map(|(_, t)| is_truthy(&process_tpl(t, context)))
    };

    // Evaluate current node attributes
    let kind_eval = match &node.kind {
        NodeType::Container => NodeType::Container,
        NodeType::Column => NodeType::Column,
        NodeType::Row => NodeType::Row,
        NodeType::Text {
            content,
            size,
            bold,
            color,
        } => NodeType::Text {
            content: process_tpl(content, context),
            size: num_template(NumAttr::Size).or(*size).or(style.size),
            bold: *bold || style.bold.unwrap_or(false),
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::Button {
            text,
            on_click,
            navigate_to,
            navigate_back,
            color,
        } => NodeType::Button {
            text: process_tpl(text, context),
            on_click: on_click
                .as_ref()
                .map(|o| namespace_action(process_tpl(o, context), owner)),
            navigate_to: navigate_to.as_ref().map(|n| process_tpl(n, context)),
            navigate_back: *navigate_back,
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::TextInput {
            placeholder,
            value_var,
            on_change,
            secure,
        } => NodeType::TextInput {
            placeholder: process_tpl(placeholder, context),
            value_var: process_tpl(value_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
            secure: *secure,
        },
        NodeType::TextArea {
            placeholder,
            value_var,
            on_change,
            readonly,
        } => NodeType::TextArea {
            placeholder: process_tpl(placeholder, context),
            value_var: process_tpl(value_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
            readonly: *readonly,
        },
        NodeType::Image {
            source,
            clip_circle,
        } => NodeType::Image {
            source: process_tpl(source, context),
            clip_circle: *clip_circle,
        },
        NodeType::Svg { source, color } => NodeType::Svg {
            source: process_tpl(source, context),
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::Scrollable { direction } => NodeType::Scrollable {
            direction: direction.clone(),
        },
        NodeType::Checkbox {
            label,
            checked_var,
            on_toggle,
            tristate,
        } => NodeType::Checkbox {
            label: process_tpl(label, context),
            checked_var: process_tpl(checked_var, context),
            on_toggle: namespace_action(process_tpl(on_toggle, context), owner),
            tristate: *tristate,
        },
        NodeType::Toggle {
            label,
            checked_var,
            on_toggle,
        } => NodeType::Toggle {
            label: process_tpl(label, context),
            checked_var: process_tpl(checked_var, context),
            on_toggle: namespace_action(process_tpl(on_toggle, context), owner),
        },
        NodeType::Rule { horizontal } => NodeType::Rule {
            horizontal: *horizontal,
        },
        NodeType::ProgressBar {
            value_var,
            min,
            max,
            vertical,
            show_value,
            color,
        } => NodeType::ProgressBar {
            value_var: process_tpl(value_var, context),
            min: *min,
            max: *max,
            vertical: *vertical,
            show_value: *show_value,
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::DateTimeEdit {
            value_var,
            date,
            time,
            seconds,
            day_first,
            on_change,
        } => NodeType::DateTimeEdit {
            value_var: process_tpl(value_var, context),
            date: *date,
            time: *time,
            seconds: *seconds,
            day_first: *day_first,
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::Calendar {
            value_var,
            end_var,
            month_var,
            today,
            min,
            max,
            mode,
            monday_first,
            months,
            range,
            month_names,
            day_names,
            on_change,
        } => NodeType::Calendar {
            value_var: process_tpl(value_var, context),
            end_var: process_tpl(end_var, context),
            month_var: process_tpl(month_var, context),
            today: process_tpl(today, context),
            min: process_tpl(min, context),
            max: process_tpl(max, context),
            mode: *mode,
            monday_first: *monday_first,
            months: *months,
            range: *range,
            month_names: process_tpl(month_names, context),
            day_names: process_tpl(day_names, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::Pagination {
            value_var,
            total,
            window,
            ends,
            on_change,
        } => NodeType::Pagination {
            value_var: process_tpl(value_var, context),
            total: process_tpl(total, context),
            window: *window,
            ends: *ends,
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::Rating {
            value_var,
            max,
            filled,
            empty,
            size,
            color,
            readonly,
            on_change,
        } => NodeType::Rating {
            value_var: process_tpl(value_var, context),
            max: process_tpl(max, context),
            filled: process_tpl(filled, context),
            empty: process_tpl(empty, context),
            size: *size,
            // A cor cai na classe `.gss` quando o atributo não a dá — o mesmo
            // fallback do `color` do `Button` e do `ProgressBar`.
            color: {
                let c = process_tpl(color, context);
                if c.is_empty() {
                    style.color.clone().unwrap_or_default()
                } else {
                    c
                }
            },
            readonly: *readonly,
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::MaskedInput {
            value_var,
            mask,
            placeholder,
            on_change,
        } => NodeType::MaskedInput {
            value_var: process_tpl(value_var, context),
            // A máscara também interpola, e o resultado volta a passar pelos
            // presets: `mask="{formato}"` com `formato="cpf"` funciona.
            mask: crate::parser::mascara_preset(&process_tpl(mask, context)),
            placeholder: process_tpl(placeholder, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::Radio {
            label,
            value,
            group_var,
            on_change,
        } => NodeType::Radio {
            label: process_tpl(label, context),
            value: process_tpl(value, context),
            group_var: process_tpl(group_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
        },
        NodeType::Slider {
            value_var,
            on_change,
            on_release,
            min,
            max,
            step,
            step_raw,
            shift_step,
            default,
            vertical,
            color,
        } => NodeType::Slider {
            value_var: process_tpl(value_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
            on_release: on_release
                .as_ref()
                .map(|a| namespace_action(process_tpl(a, context), owner)),
            min: *min,
            max: *max,
            step: *step,
            step_raw: step_raw.clone(),
            shift_step: *shift_step,
            default: *default,
            vertical: *vertical,
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::Space => NodeType::Space,
        NodeType::Spinner { color } => NodeType::Spinner {
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::Select {
            options,
            value_var,
            on_change,
            placeholder,
            label_field,
            value_field,
            color,
        } => NodeType::Select {
            options: process_tpl(options, context),
            value_var: process_tpl(value_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
            placeholder: process_tpl(placeholder, context),
            label_field: label_field.clone(),
            value_field: value_field.clone(),
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::ComboEdit {
            options,
            value_var,
            on_change,
            on_select,
            placeholder,
            label_field,
            value_field,
            color,
        } => NodeType::ComboEdit {
            options: process_tpl(options, context),
            value_var: process_tpl(value_var, context),
            on_change: namespace_action(process_tpl(on_change, context), owner),
            on_select: namespace_action(process_tpl(on_select, context), owner),
            placeholder: process_tpl(placeholder, context),
            label_field: label_field.clone(),
            value_field: value_field.clone(),
            color: color
                .as_ref()
                .map(|c| process_tpl(c, context))
                .or_else(|| style.color.clone()),
        },
        NodeType::MenuBar => NodeType::MenuBar,
        NodeType::Menu {
            label,
            icon,
            disabled,
            items,
        } => NodeType::Menu {
            label: process_tpl(label, context),
            icon: icon.as_ref().map(|i| process_tpl(i, context)),
            disabled: *disabled,
            items: items.as_ref().map(|i| process_tpl(i, context)),
        },
        NodeType::MenuItem {
            label,
            icon,
            on_click,
            checked_var,
            disabled,
        } => NodeType::MenuItem {
            label: process_tpl(label, context),
            icon: icon.as_ref().map(|i| process_tpl(i, context)),
            on_click: on_click
                .as_ref()
                .map(|o| namespace_action(process_tpl(o, context), owner)),
            checked_var: checked_var.as_ref().map(|c| process_tpl(c, context)),
            disabled: *disabled,
        },
        NodeType::MenuSeparator => NodeType::MenuSeparator,
        NodeType::ContextMenu { items } => NodeType::ContextMenu {
            items: items.as_ref().map(|i| process_tpl(i, context)),
        },
        NodeType::Form { on_submit, name } => NodeType::Form {
            on_submit: on_submit
                .as_ref()
                .map(|s| namespace_action(process_tpl(s, context), owner)),
            name: name.as_ref().map(|n| process_tpl(n, context)),
        },
        // A `Fragment` carries through evaluation as-is; its children are
        // spliced into the parent by `expand_children` (below), so it stays
        // transparent instead of collapsing into a `Container` box.
        NodeType::Fragment => NodeType::Fragment,
        // Um `<slot/>` só vira conteúdo dentro da expansão de um componente
        // (tratado no topo de `eval_owned`). Chegar aqui significa `<slot/>`
        // escrito fora de um componente: vira `Fragment`, ou seja, some e
        // deixa no lugar o próprio conteúdo de reserva que ele embrulha.
        NodeType::Slot { .. } => NodeType::Fragment,
        NodeType::Include { .. }
        | NodeType::Component { .. }
        | NodeType::Import { .. }
        | NodeType::ForEach { .. }
        | NodeType::If { .. }
        | NodeType::Else
        | NodeType::ElseIf { .. }
        | NodeType::Link { .. }
        | NodeType::Style { .. }
        | NodeType::Screen(_)
        | NodeType::ComponentRoot
        | NodeType::Resources
        | NodeType::Props(_)
        | NodeType::Prop => NodeType::Container,
    };

    // For each style field, the node's inline attribute wins; a `class` value
    // (if any) fills in only where the inline attribute is absent.
    let resolve = |inline: Option<&str>, class: &Option<String>| -> Option<String> {
        inline
            .map(|s| process_tpl(s, context))
            .or_else(|| class.clone())
    };

    let width_eval = resolve(node.width.as_deref(), &style.width);
    let height_eval = resolve(node.height.as_deref(), &style.height);
    let padding_eval = resolve(node.padding.as_deref(), &style.padding);
    let align_x_eval = resolve(node.align_x(), &style.align_x);
    let align_y_eval = resolve(node.align_y(), &style.align_y);
    let background_eval = resolve(node.background.as_deref(), &style.background);
    let border_color_eval = resolve(node.border_color(), &style.border_color);
    let spacing_eval = num_template(NumAttr::Spacing)
        .or(node.spacing)
        .or(style.spacing);
    let border_radius_eval = num_template(NumAttr::BorderRadius)
        .or(node.border_radius)
        .or(style.border_radius);
    let border_width_eval = num_template(NumAttr::BorderWidth)
        .or(node.border_width)
        .or(style.border_width);
    let font_eval = resolve(node.font(), &style.font);
    let gradient_eval = resolve(node.gradient(), &style.gradient);
    let text_align_eval = resolve(node.text_align(), &style.text_align);
    // `on_press` is behavior, not a style field; interpolate it directly so
    // actions like `onPress="window:{cmd}"` can bind context values.
    let on_press_eval = node.on_press().map(|s| process_tpl(s, context));
    let on_double_click_eval = node.on_double_click().map(|s| process_tpl(s, context));
    let cursor_eval = resolve(node.cursor(), &style.cursor);
    let text_color_eval = resolve(node.text_color(), &style.text_color);
    // `tooltip` é conteúdo, não estilo (sem equivalente `.classe { }`, como
    // `on_press`) — interpolado direto pra suportar `tooltip="{var}"`.
    let tooltip_eval = node.tooltip().map(|s| process_tpl(s, context));
    let tooltip_position_eval = node.tooltip_position().map(str::to_string);
    let max_width_eval = num_template(NumAttr::MaxWidth)
        .or(node.max_width)
        .or(style.max_width);
    let max_height_eval = num_template(NumAttr::MaxHeight)
        .or(node.max_height)
        .or(style.max_height);
    // `hidden` resolvido: o inline vence a classe/`@media` (mesma precedência
    // dos demais campos), e um `hidden="{chave}"` vence os dois — é o mais
    // específico que o autor pôde escrever. Consumido em `widget::render_node`
    // (pulado no layout).
    let hidden_eval = bool_template(BoolAttr::Hidden)
        .or(node.hidden)
        .or(style.hidden);
    // `disabled` só existe como atributo inline (sem equivalente `.classe { }`),
    // carregado direto, como `drag_handle` — mas também aceita placeholder.
    let disabled_eval = bool_template(BoolAttr::Disabled).or(node.disabled);
    // Overlays por pseudo-estado: só embrulha num `Box` quando o `.gss`
    // realmente declarou algo para aquele estado, para não pagar uma
    // alocação por nó no caso comum (nenhum `:hover`/`:focus`/etc. no sheet).
    let box_state = |r: StyleRule| -> Option<Box<StyleRule>> {
        if r == StyleRule::default() {
            None
        } else {
            Some(Box::new(r))
        }
    };
    let hover_style_eval = box_state(state_styles.hover);
    let focus_style_eval = box_state(state_styles.focus);
    let active_style_eval = box_state(state_styles.active);
    let disabled_style_eval = box_state(state_styles.disabled);

    // Evaluate children recursively. ForEach/if/else/Import are structural:
    // they are expanded or dropped rather than rendered directly.
    let mut children_eval = Vec::new();
    expand_children(
        &node.children,
        context,
        templates,
        styles,
        scope,
        owner,
        &mut children_eval,
        // Um `<slot/>` a qualquer profundidade do template do componente ainda
        // recebe o conteúdo do uso: `<Column><Row><slot/></Row></Column>`.
        slot,
        cache,
    )?;

    // A `<Form>` hydrates every `formControl`-bound descendant (at any depth,
    // through nested Rows/Columns) with the shared scope, its evaluated
    // `onSubmit` action, and — per control, in document order — the name of
    // the next one, mirroring how a reorderable for-each hydrates its
    // `dragHandle` (see `hydrate_drag_item` below).
    if let NodeType::Form { on_submit, name } = &kind_eval {
        let form_scope = format!("{}::{}", owner.unwrap_or(""), name.as_deref().unwrap_or(""));
        let submit_action = on_submit.clone().unwrap_or_default();
        let mut order = Vec::new();
        collect_form_control_names(&children_eval, &mut order);
        hydrate_form_controls(&mut children_eval, &order, &form_scope, &submit_action);
    }

    Ok(UiNode {
        node_id: node.node_id,
        kind: kind_eval,
        children: children_eval.into(),
        // Numeric templates are resolved into the f32 fields below; nothing left.
        numeric_templates: Vec::new(),
        // Idem para os booleanos: já viraram `hidden_eval`/`disabled_eval`.
        bool_templates: Vec::new(),
        width: width_eval,
        height: height_eval,
        padding: padding_eval,
        spacing: spacing_eval,
        virtualize: num_template(NumAttr::Virtualize).or(node.virtualize),
        background: background_eval,
        border_radius: border_radius_eval,
        border_width: border_width_eval,
        // Classes and id are fully resolved into the fields above; nothing to
        // carry on.
        class: None,
        id: None,
        max_width: max_width_eval,
        max_height: max_height_eval,
        hidden: hidden_eval,
        disabled: disabled_eval,
        if_empty: false,
        if_not_empty: false,
        is_else: false,
        // `drag_handle` is a static marker (no template to resolve); carried
        // through unevaluated so a reorderable item's handle survives eval.
        drag_handle: node.drag_handle,
        look: crate::parser::caixa(crate::parser::Look {
            align_x: align_x_eval,
            align_y: align_y_eval,
            border_color: border_color_eval,
            font: font_eval,
            gradient: gradient_eval,
            text_align: text_align_eval,
            text_color: text_color_eval,
            // A diretiva de destino é consumida na fronteira do componente, ao
            // repartir o conteúdo do uso — nada dela sobrevive à avaliação.
            slot_name: None,
        }),
        interact: crate::parser::caixa(crate::parser::Interact {
            on_press: on_press_eval,
            on_double_click: on_double_click_eval,
            cursor: cursor_eval,
            tooltip: tooltip_eval,
            tooltip_position: tooltip_position_eval,
        }),
        pseudo: crate::parser::caixa(crate::parser::Pseudo {
            hover_style: hover_style_eval,
            focus_style: focus_style_eval,
            active_style: active_style_eval,
            disabled_style: disabled_style_eval,
        }),
        // As diretivas de fluxo são consumidas pela própria avaliação — o que
        // sai daqui já é o resultado delas, nunca a condição.
        cond: None,
        // Hydrated (if at all) by the *parent* for-each's expansion, onto this
        // very node, before it reached this call — carried through as-is
        // (nothing here to interpolate; identities are already resolved).
        // `on_reorder`/`reorder_key` are only meaningful on a for-each node,
        // consumed (and interpolated) directly by `expand_children`'s for-each
        // handling below — nothing to carry on past evaluation.
        drag: crate::parser::caixa(crate::parser::Drag {
            on_reorder: None,
            reorder_key: None,
            drag_list: node.drag_list().map(str::to_string),
            drag_item_key: node.drag_item_key().map(str::to_string),
            drag_order: node.drag_order().map(<[String]>::to_vec),
            drag_on_reorder: node.drag_on_reorder().map(str::to_string),
            drag_reorder_key: node.drag_reorder_key().map(str::to_string),
        }),
        form: crate::parser::caixa(crate::parser::FormBits {
            form_control: node.form_control().map(|s| process_tpl(s, context)),
            // Hydrated (if at all) by the enclosing `<Form>`'s post-pass above,
            // on this very (already evaluated) node — carried through as a
            // default of `None` here, same as the drag_* fields are for a plain
            // for-each item.
            form_scope: node.form_scope().map(str::to_string),
            form_submit_action: node.form_submit_action().map(str::to_string),
            form_next_focus: node.form_next_focus().map(str::to_string),
        }),
    })
}

/// Collects the `form_control` name of every node across `nodes` (a `<Form>`'s
/// already-evaluated subtree) in document order — the tab/Enter order used to
/// find each control's "next" one.
fn collect_form_control_names(nodes: &[UiNode], out: &mut Vec<String>) {
    for node in nodes {
        if let Some(name) = node.form_control() {
            out.push(name.to_string());
        }
        collect_form_control_names(&node.children, out);
    }
}

/// Hydrates every `form_control`-bound node across `nodes` with the enclosing
/// `<Form>`'s `scope` (used to build a stable focus id) and evaluated
/// `on_submit` action, plus the name of the next control in `order` (`None` on
/// the last one).
fn hydrate_form_controls(nodes: &mut [UiNode], order: &[String], scope: &str, on_submit: &str) {
    for node in nodes.iter_mut() {
        if let Some(name) = node.form_control() {
            let next = order
                .iter()
                .position(|n| n == name)
                .and_then(|i| order.get(i + 1))
                .cloned();
            node.set_form_scope(Some(scope.to_string()));
            node.set_form_submit_action(Some(on_submit.to_string()));
            node.set_form_next_focus(next);
        }
        hydrate_form_controls(node.children.to_mut(), order, scope, on_submit);
    }
}

fn hydrate_drag_item(
    nodes: &mut [UiNode],
    list: &str,
    key: &str,
    order: &[String],
    on_reorder: &str,
    reorder_key: &str,
) {
    for node in nodes.iter_mut() {
        node.set_drag_list(Some(list.to_string()));
        node.set_drag_item_key(Some(key.to_string()));
    }
    // Hydrate EVERY `dragHandle` in the item body, not just the first. An item
    // whose body branches on a directive — e.g. `if {e.__dragging} { …handle… }
    // else { …handle… }` — defines the handle once per branch. Only one branch
    // renders per item, and it may not be the first one found; stopping at the
    // first match (the old `find_handle` + `break`) left the *rendered* branch's
    // handle without drag metadata, so `DragStart` fired with no order and the
    // reorder silently did nothing.
    fn hydrate_handles(
        node: &mut UiNode,
        list: &str,
        key: &str,
        order: &[String],
        on_reorder: &str,
        reorder_key: &str,
    ) {
        if node.drag_handle {
            node.set_drag_list(Some(list.to_string()));
            node.set_drag_item_key(Some(key.to_string()));
            node.set_drag_reorder_key(Some(reorder_key.to_string()));
            node.set_drag_order(Some(order.to_vec()));
            node.set_drag_on_reorder(Some(on_reorder.to_string()));
        }
        for c in node.children.to_mut().iter_mut() {
            hydrate_handles(c, list, key, order, on_reorder, reorder_key);
        }
    }
    for node in nodes.iter_mut() {
        hydrate_handles(node, list, key, order, on_reorder, reorder_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- normalize_bare_directives ---------------------------------------

    /// `else`/`senao` sem valor viram `else=""`/`senao=""` — o mecanismo que
    /// deixa `<Text else>` valer como XML.
    #[test]
    fn normalize_bare_directives_reescreve_else_sem_valor() {
        assert_eq!(
            normalize_bare_directives(r#"<Text else>D</Text>"#),
            r#"<Text else="">D</Text>"#
        );
        assert_eq!(
            normalize_bare_directives(r#"<Text senao>D</Text>"#),
            r#"<Text senao="">D</Text>"#
        );
    }

    /// Regressão: um atributo mais longo que só COMEÇA com "else"/"senao"
    /// (`else-if`) não pode ser confundido com o `else` desacompanhado —
    /// virava `else=""-if="…"`, XML inválido ("expected a whitespace not
    /// '-'"). Achado ao implementar `else-if` (Fase 1, item 2 do plano de
    /// convergência de templates do rustploy).
    #[test]
    fn normalize_bare_directives_nao_mexe_em_else_if() {
        let input = r#"<Text else-if="{x}" equals="b">B</Text>"#;
        assert_eq!(normalize_bare_directives(input), input);
    }

    /// `else=""` já explícito (com valor) passa direto, sem virar
    /// `else=""=""`.
    #[test]
    fn normalize_bare_directives_nao_mexe_em_else_com_valor_ja_presente() {
        let input = r#"<Text else="">D</Text>"#;
        assert_eq!(normalize_bare_directives(input), input);
    }

    /// `empty`/`not_empty` sem valor viram `empty=""`/`not_empty=""` — o
    /// mesmo mecanismo do `else`, generalizado (Fase 1, item 4 do plano de
    /// convergência de templates do rustploy).
    #[test]
    fn normalize_bare_directives_reescreve_empty_e_not_empty_sem_valor() {
        assert_eq!(
            normalize_bare_directives(r#"<Text if="{proj_secrets}" empty>vazio</Text>"#),
            r#"<Text if="{proj_secrets}" empty="">vazio</Text>"#
        );
        assert_eq!(
            normalize_bare_directives(r#"<Text if="{proj_secrets}" not_empty>tem</Text>"#),
            r#"<Text if="{proj_secrets}" not_empty="">tem</Text>"#
        );
    }

    /// Regressão do mesmo tipo do `else-if`: `not_empty` embute a palavra
    /// `empty` no meio (`not_€mpty`) — o scanner passa por ela caractere a
    /// caractere e não pode confundir esse `empty` interno (precedido por
    /// `_`, não espaço) com um atributo `empty` desacompanhado de verdade.
    #[test]
    fn normalize_bare_directives_nao_confunde_empty_dentro_de_not_empty() {
        let input = r#"<Text if="{x}" not_empty>A</Text>"#;
        assert_eq!(
            normalize_bare_directives(input),
            r#"<Text if="{x}" not_empty="">A</Text>"#
        );
    }

    #[test]
    fn json_array_is_empty_cobre_lista_vazia_cheia_e_json_invalido() {
        assert!(json_array_is_empty("[]"));
        assert!(!json_array_is_empty(r#"[{"name":"x"}]"#));
        // Chave ainda não populada / JSON malformado: "sem lista" também
        // conta como vazio — é a leitura honesta do estado, não um erro.
        assert!(json_array_is_empty(""));
        assert!(json_array_is_empty("not json"));
        assert!(json_array_is_empty("{}")); // objeto, não array
    }

    #[test]
    fn namespace_action_prefixes_component_actions() {
        assert_eq!(
            namespace_action("connect".to_string(), Some("Login")),
            "Login::connect"
        );
    }

    #[test]
    fn namespace_action_leaves_top_level_actions_untouched() {
        assert_eq!(namespace_action("connect".to_string(), None), "connect");
    }

    #[test]
    fn namespace_action_never_namespaces_builtin_prefixes() {
        // Built-ins (`clipboard:`/`open:`/`window:`) são globais e resolvidos por
        // `GlacierUI::dispatch` via `strip_prefix` — se um componente importado os
        // namespaceasse (ex.: `ServiceDetail::clipboard:foo`), o strip falharia e o
        // clipboard/open/window nunca dispararia. Trava essa regressão.
        for action in ["clipboard:svc_external_url", "open:my_url", "window:close"] {
            assert_eq!(
                namespace_action(action.to_string(), Some("ServiceDetail")),
                action,
                "ação built-in não pode ser namespaceada"
            );
        }
    }

    // --- Seletor de tag (builtin + componente), fim-a-fim pelo eval ------------

    fn parse(xml: &str) -> UiNode {
        UiNode::parse_xml(xml).unwrap()
    }

    /// Avalia `xml` com `sheet` como sheet global e um mapa de componentes.
    fn eval_with(xml: &str, gss: &str, templates: &HashMap<String, UiNode>) -> UiNode {
        let global = vec![StyleSheet::parse(gss).unwrap()];
        let by_component: HashMap<String, Vec<StyleSheet>> = HashMap::default();
        let styles = StyleContext {
            global: &global,
            by_component: &by_component,
            viewport: None,
            has_tag_rules: global.iter().any(|s| s.has_tag_rules()),
        };
        evaluate_node(&parse(xml), &HashMap::default(), templates, &styles, None).unwrap()
    }

    /// Avalia `node` contra `ctx`, sem stylesheet nenhum.
    fn eval_ctx(node: &UiNode, ctx: &ContextMap) -> UiNode {
        let global: Vec<StyleSheet> = Vec::new();
        let by_component: HashMap<String, Vec<StyleSheet>> = HashMap::default();
        let styles = StyleContext {
            global: &global,
            by_component: &by_component,
            viewport: None,
            has_tag_rules: false,
        };
        evaluate_node(node, ctx, &HashMap::default(), &styles, None).unwrap()
    }

    #[test]
    fn builtin_tag_selector_applies_to_node() {
        // `Button { padding: 7 }` casa o kind builtin, sem class/id no nó.
        let out = eval_with(
            r#"<Button text="x" />"#,
            "Button { padding: 7; }",
            &HashMap::default(),
        );
        assert_eq!(out.padding.as_deref(), Some("7"));
    }

    #[test]
    fn inline_wins_over_builtin_tag() {
        let out = eval_with(
            r#"<Button text="x" padding="20" />"#,
            "Button { padding: 7; }",
            &HashMap::default(),
        );
        assert_eq!(out.padding.as_deref(), Some("20"));
    }

    #[test]
    fn component_tag_selector_underlays_inlined_root() {
        // `Card {}` casa o NOME do componente e vira underlay na raiz (Column) do
        // template inlinado. O `background` da raiz (via classe) vence o underlay,
        // mas o `padding`, que só o underlay declara, sobrevive.
        let mut templates = HashMap::default();
        templates.insert(
            "Card".to_string(),
            parse(r#"<Column class="root"><Text content="oi" /></Column>"#),
        );
        let out = eval_with(
            r#"<Card />"#,
            ".root { background: #101010; } Card { padding: 24; background: #ffffff; }",
            &templates,
        );
        // A raiz avaliada é a Column do template.
        assert!(matches!(out.kind, NodeType::Column));
        assert_eq!(out.padding.as_deref(), Some("24")); // só o underlay declara
        assert_eq!(out.background.as_deref(), Some("#101010")); // classe vence o underlay
    }

    #[test]
    fn tag_selector_ignored_without_any_tag_rule() {
        // Sem regra de tag no sheet, um nó pelado não paga resolução e nada muda.
        let out = eval_with(
            r#"<Button text="x" />"#,
            ".unused { padding: 9; }",
            &HashMap::default(),
        );
        assert_eq!(out.padding, None);
    }

    /// Regressão: `hidden`/`disabled` com placeholder eram resolvidos no PARSE,
    /// comparando a string crua (`"{oculto}"` != `"true"`), então o data binding
    /// que a documentação promete nunca ligava — um `hidden="{parado}"` deixava
    /// o spinner girando para sempre. Agora a string vai para `bool_templates` e
    /// é interpolada aqui, com o mesmo teste de verdade do `if`.
    #[test]
    fn hidden_e_disabled_resolvem_placeholder_do_contexto() {
        let mut ctx = HashMap::default();
        ctx.insert("oculto".to_string(), "true".to_string());
        ctx.insert("travado".to_string(), "false".to_string());

        let node = parse(r#"<Button text="x" hidden="{oculto}" disabled="{travado}" />"#);
        let out = eval_ctx(&node, &ctx);
        assert_eq!(out.hidden, Some(true), "hidden deve seguir o contexto");
        assert_eq!(out.disabled, Some(false), "disabled deve seguir o contexto");

        // A mesma árvore com o contexto invertido produz o inverso — é o ponto
        // todo do binding: o valor acompanha o estado, não o texto do template.
        ctx.insert("oculto".to_string(), "false".to_string());
        ctx.insert("travado".to_string(), "sim".to_string());
        let out = eval_ctx(&node, &ctx);
        assert_eq!(out.hidden, Some(false));
        assert_eq!(
            out.disabled,
            Some(true),
            "`sim` também é verdade, como no `if`"
        );
    }

    /// Valor literal continua valendo (o caminho antigo não pode ter regredido),
    /// e uma chave ausente resolve para vazio, que não é verdade.
    #[test]
    fn hidden_literal_e_chave_ausente_seguem_valendo() {
        let ctx = HashMap::default();
        let node = parse(r#"<Button text="x" hidden="true" />"#);
        let out = eval_ctx(&node, &ctx);
        assert_eq!(out.hidden, Some(true));

        let node = parse(r#"<Button text="x" hidden="{nao_existe}" />"#);
        let out = eval_ctx(&node, &ctx);
        assert_eq!(out.hidden, Some(false));
    }
}
