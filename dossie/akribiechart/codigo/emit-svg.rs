//! SVG acessível e determinístico (ADR-0005).
//!
//! Regras que o teste garante:
//! - `role="img"` + `<title id="akr-t">` + `<desc id="akr-d">` + `aria-labelledby`;
//! - `lang` do documento e `focusable="false"`;
//! - grupos `groups`, `edges`, `nodes`, `notes` nesta ordem, cada figura com `data-id`/`data-kind`;
//! - todo número passa por [`num`] (duas casas, sem zeros) e todo texto por [`escape`];
//! - mesma `Scene` ⇒ mesmos bytes (nenhuma ordenação instável, nenhum `HashMap`).
//!
//! A serialização acumula linhas num `Vec<String>` e fecha com `join("\n")`. A alternativa —
//! escrever direto num `String` pré-dimensionado — foi medida e empatou (a emissão é dezenas de
//! microssegundos dentro de uma execução de ~3 ms, dominada por inicialização e rasterização):
//! ficou a versão mais simples de ler, porque o ganho não pagava a mudança.

use crate::num::format as num;
use crate::paint::{Anchor, EdgeDraw, GroupDraw, NodeDraw, NoteDraw, Scene, Stroke, TextRun};
use crate::theme::Theme;

/// Serializa a cena.
pub fn svg(scene: &Scene, theme: &Theme) -> String {
    let mut parts: Vec<String> = Vec::new();
    let labelled = if scene.title.is_some() {
        " aria-labelledby=\"akr-t akr-d\""
    } else {
        ""
    };
    let width = num(scene.width);
    let height = num(scene.height);
    parts.push(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" \
         viewBox=\"0 0 {width} {height}\" role=\"img\"{labelled} lang=\"{}\" focusable=\"false\">",
        escape(&scene.lang)
    ));
    if let Some(title) = &scene.title {
        parts.push(format!("<title id=\"akr-t\">{}</title>", escape(title)));
    }
    parts.push(format!("<desc id=\"akr-d\">{}</desc>", escape(&scene.desc)));
    let clips: Vec<String> = scene
        .nodes
        .iter()
        .filter_map(|node| {
            node.clip.as_ref().map(|d| {
                format!(
                    "<clipPath id=\"c-{}\"><path d=\"{d}\"/></clipPath>",
                    escape(&id_of(&node.id))
                )
            })
        })
        .collect();
    if !clips.is_empty() {
        parts.push(format!("<defs>{}</defs>", clips.join("")));
    }
    parts.push(format!(
        "<rect id=\"canvas\" width=\"{width}\" height=\"{height}\" fill=\"{}\"/>",
        theme.canvas
    ));
    let stack = scene.font_stack;
    section(
        &mut parts,
        "groups",
        scene.groups.iter().map(|group| group_element(group, stack)),
    );
    section(
        &mut parts,
        "edges",
        scene
            .edges
            .iter()
            .map(|edge| edge_element(edge, theme, stack)),
    );
    section(
        &mut parts,
        "nodes",
        scene.nodes.iter().map(|node| node_element(node, stack)),
    );
    section(
        &mut parts,
        "notes",
        scene.notes.iter().map(|note| note_element(note, stack)),
    );
    parts.push("</svg>".to_string());
    parts.join("\n")
}

fn section(parts: &mut Vec<String>, id: &str, elements: impl Iterator<Item = String>) {
    let elements: Vec<String> = elements.collect();
    if elements.is_empty() {
        return;
    }
    parts.push(format!("<g id=\"{id}\">"));
    parts.extend(elements);
    parts.push("</g>".to_string());
}

fn node_element(node: &NodeDraw, stack: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(fill) = &node.fill {
        if fill.solid {
            parts.push(format!(
                "<path d=\"{}\" fill=\"{}\" stroke=\"none\"/>",
                fill.d, fill.color
            ));
        } else {
            let clip = if node.clip.is_some() {
                format!(" clip-path=\"url(#c-{})\"", escape(&id_of(&node.id)))
            } else {
                String::new()
            };
            parts.push(format!(
                "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"{clip}/>",
                fill.d, fill.color
            ));
        }
    }
    if let Some(stroke) = &node.stroke {
        parts.push(stroke_element(stroke));
    }
    for run in &node.text {
        parts.push(text_element(run, stack));
    }
    wrap(&node.id, "node", node.aria_label.as_deref(), &parts)
}

fn group_element(group: &GroupDraw, stack: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(stroke_element(&group.stroke));
    for run in &group.label {
        parts.push(text_element(run, stack));
    }
    wrap(&group.id, "group", group.aria_label.as_deref(), &parts)
}

fn note_element(note: &NoteDraw, stack: &str) -> String {
    let mut parts: Vec<String> = vec![
        format!(
            "<path d=\"{}\" fill=\"{}\" stroke=\"none\"/>",
            note.fill.d, note.fill.color
        ),
        stroke_element(&note.stroke),
    ];
    for run in &note.text {
        parts.push(text_element(run, stack));
    }
    wrap(&note.id, "note", note.aria_label.as_deref(), &parts)
}

fn edge_element(edge: &EdgeDraw, theme: &Theme, stack: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(stroke_element(&edge.route));
    for arrow in [&edge.tail, &edge.head].into_iter().flatten() {
        parts.push(format!(
            "<path d=\"{}\" fill=\"{}\" stroke=\"none\"/>",
            arrow.d, arrow.color
        ));
    }
    // O fundo do rótulo é desenhado DEPOIS do traço: assim a linha é interrompida sob o texto e
    // nunca encosta nele. Antes, o traço era pintado por cima do fundo e atravessava o rótulo.
    if let Some(label) = &edge.label {
        parts.push(format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>",
            num(label.rect.x),
            num(label.rect.y),
            num(label.rect.w),
            num(label.rect.h),
            theme.canvas
        ));
        for run in &label.text {
            parts.push(text_element(run, stack));
        }
    }
    wrap(&edge.id, "edge", edge.aria_label.as_deref(), &parts)
}

fn wrap(id: &str, kind: &str, aria: Option<&str>, parts: &[String]) -> String {
    let label = aria
        .map(|label| format!(" aria-label=\"{}\"", escape(label)))
        .unwrap_or_default();
    format!(
        "<g data-id=\"{}\" data-kind=\"{kind}\"{label}>{}</g>",
        escape(id),
        parts.join("")
    )
}

fn stroke_element(stroke: &Stroke) -> String {
    let dash = stroke
        .dash
        .map(|dash| format!(" stroke-dasharray=\"{dash}\""))
        .unwrap_or_default();
    format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{dash} \
         stroke-linecap=\"round\" stroke-linejoin=\"round\"/>",
        stroke.d,
        stroke.color,
        num(stroke.width)
    )
}

fn text_element(run: &TextRun, stack: &str) -> String {
    let anchor = match run.anchor {
        Anchor::Middle => "middle",
        Anchor::Start => "start",
    };
    format!(
        "<text x=\"{}\" y=\"{}\" text-anchor=\"{anchor}\" dominant-baseline=\"middle\" \
         font-family=\"{stack}\" font-size=\"{}\" fill=\"{}\">{}</text>",
        num(run.x),
        num(run.y),
        num(run.size),
        run.color,
        escape(&run.text)
    )
}

/// Id seguro para `clipPath` (documento XML não aceita espaço nem `#` em id de referência).
fn id_of(id: &str) -> String {
    id.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

/// Escapa texto para conteúdo e atributo XML.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests;
