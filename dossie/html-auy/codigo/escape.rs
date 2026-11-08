//! Escape por contexto: o **único** lugar do crate que decide entidades.
//!
//! Três contextos, três tabelas. Nenhum deles depende de alocação intermediária: o texto seguro é
//! copiado em blocos e só os caracteres perigosos viram entidade.

use alloc::string::String;

/// Escreve `input` como conteúdo de elemento (`&`, `<`, `>`, `"`, `'`, NUL).
///
/// ```
/// let mut out = String::new();
/// html_auy::escape::text(&mut out, "a < b & \"c\"");
/// assert_eq!(out, "a &lt; b &amp; &quot;c&quot;");
/// ```
pub fn text(out: &mut String, input: &str) {
    write_with(out, input, |ch| match ch {
        '&' => Some("&amp;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' => Some("&quot;"),
        '\'' => Some("&#x27;"),
        '\0' => Some("&#xFFFD;"),
        _ => None,
    });
}

/// Escreve `input` como valor de atributo entre aspas duplas.
///
/// Além dos caracteres de texto, espaço em branco vira entidade numérica: é o que impede um valor
/// de quebrar a linha do atributo na serialização.
pub fn attr(out: &mut String, input: &str) {
    write_with(out, input, |ch| match ch {
        '&' => Some("&amp;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' => Some("&quot;"),
        '\'' => Some("&#x27;"),
        '\0' => Some("&#xFFFD;"),
        '\r' => Some("&#13;"),
        '\n' => Some("&#10;"),
        '\t' => Some("&#9;"),
        _ => None,
    });
}

/// Escreve `input` como corpo de `<script type="application/ld+json">`.
///
/// `<`, `>`, `&` e `'` viram escape `\uXXXX` (JSON aceita, HTML não vê `<`), o que torna
/// `</script` impossível por construção. U+2028/U+2029 também são escapados: são quebras de linha
/// em JavaScript e quebram um parser ingênuo.
pub fn json_script(out: &mut String, input: &str) {
    write_with(out, input, |ch| match ch {
        '<' => Some("\\u003c"),
        '>' => Some("\\u003e"),
        '&' => Some("\\u0026"),
        '\'' => Some("\\u0027"),
        '\u{2028}' => Some("\\u2028"),
        '\u{2029}' => Some("\\u2029"),
        _ => None,
    });
}

/// Uma passada: copia trechos seguros, substitui o que precisa de entidade.
fn write_with(out: &mut String, input: &str, replacement: impl Fn(char) -> Option<&'static str>) {
    let mut last = 0;
    for (index, ch) in input.char_indices() {
        let Some(entity) = replacement(ch) else {
            continue;
        };
        if let Some(safe) = input.get(last..index) {
            out.push_str(safe);
        }
        out.push_str(entity);
        last = index.saturating_add(ch.len_utf8());
    }
    if let Some(tail) = input.get(last..) {
        out.push_str(tail);
    }
}
