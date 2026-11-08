//! Canonical schema fingerprint.

use alloc::string::String;

use super::{DocSpec, MarkSpec, NodeSpec};

#[allow(
    clippy::format_push_string,
    clippy::too_many_lines,
    reason = "one canonical declaration feed for BLAKE3"
)]
pub(super) fn fingerprint(doc: &DocSpec, nodes: &[NodeSpec], marks: &[MarkSpec]) -> String {
    let mut text = String::new();
    text.push_str("rich-editor-auy/schema/v1\n");
    text.push_str(&alloc::format!("doc|{}|{:?}|", doc.content, doc.a11y));
    for attr in &doc.attrs {
        text.push_str(&alloc::format!(
            "{:?}|{}|{:?}|{}|",
            attr.id,
            attr.name,
            attr.kind,
            attr.required
        ));
    }
    for node in nodes {
        text.push_str(&alloc::format!(
            "node|{}|{}|{}|{}|{:?}|{}|{}|{}|{}|{:?}|",
            node.name,
            node.kind.0,
            node.content.as_deref().unwrap_or(""),
            node.groups.join(","),
            node.marks,
            node.inline,
            node.atom,
            node.defining,
            node.isolating,
            node.a11y
        ));
        for attr in &node.attrs {
            text.push_str(&alloc::format!(
                "a|{}|{}|{:?}|{}|{:?}|",
                attr.id.0,
                attr.name,
                attr.kind,
                attr.required,
                attr.default
            ));
        }
    }
    for mark in marks {
        text.push_str(&alloc::format!(
            "mark|{}|{}|{}|{}|{}|{:?}|",
            mark.name,
            mark.id.0,
            mark.inclusive,
            mark.groups.join(","),
            mark.excludes.join(","),
            mark.a11y
        ));
        for attr in &mark.attrs {
            text.push_str(&alloc::format!(
                "a|{}|{}|{:?}|{}|{:?}|",
                attr.id.0,
                attr.name,
                attr.kind,
                attr.required,
                attr.default
            ));
        }
    }
    let mut hasher = blake3::Hasher::new_derive_key("rich-editor-auy/schema/v1");
    hasher.update(text.as_bytes());
    hex(hasher.finalize().as_bytes())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        let high = usize::from(byte >> 4);
        let low = usize::from(byte & 0x0f);
        if let (Some(high), Some(low)) = (DIGITS.get(high), DIGITS.get(low)) {
            out.push(char::from(*high));
            out.push(char::from(*low));
        }
    }
    out
}
