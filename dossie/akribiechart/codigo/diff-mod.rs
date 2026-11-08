//! Diff semântico entre dois documentos resolvidos.
//!
//! Compara o `.akrc` resolvido — topologia, rótulos e estilo —, nunca geometria: o layout é
//! derivado e muda sem mudança de intenção. Identidade: `node`/`group` por `id`, `edge` por
//! `(from, to, ordinal)` e `note` por `near` + posição entre as notas da mesma âncora (a gramática
//! não dá id a nota). O resultado é ordenado por id e agrupado na ordem `added`, `removed`,
//! `changed` (ADR-0002).

use std::collections::BTreeMap;

use crate::model::{Chart, Edge, Note};

use describe::{
    edge_fields, edge_identity, edge_text, group_fields, group_identity, group_text, node_fields,
    node_identity, node_text, note_fields, note_identity, note_text,
};

mod describe;

/// Tipo de mudança.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Elemento que só existe no documento novo.
    Added,
    /// Elemento que só existe no documento antigo.
    Removed,
    /// Elemento presente nos dois, com ao menos um campo diferente.
    Changed,
}

/// Uma mudança semântica.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Adição, remoção ou alteração.
    pub kind: Kind,
    /// Descrição do elemento (`node api "API"`, `edge api->db#0`).
    pub element: String,
    /// Campo alterado (só em [`Kind::Changed`]).
    pub field: Option<&'static str>,
    /// Valor no documento antigo.
    pub before: Option<String>,
    /// Valor no documento novo.
    pub after: Option<String>,
}

impl Change {
    fn added(element: String) -> Self {
        Self {
            kind: Kind::Added,
            element,
            field: None,
            before: None,
            after: None,
        }
    }

    fn removed(element: String) -> Self {
        Self {
            kind: Kind::Removed,
            element,
            field: None,
            before: None,
            after: None,
        }
    }

    /// Linha de texto: `+ node dlq "DLQ"`, `- node legacy`, `~ node a shape: rect -> rounded`.
    pub fn line(&self) -> String {
        match self.kind {
            Kind::Added => format!("+ {}", self.element),
            Kind::Removed => format!("- {}", self.element),
            Kind::Changed => format!(
                "~ {} {}: {} -> {}",
                self.element,
                self.field.unwrap_or("?"),
                self.before.as_deref().unwrap_or(""),
                self.after.as_deref().unwrap_or("")
            ),
        }
    }
}

/// Compara dois documentos resolvidos.
pub fn diff(old: &Chart, new: &Chart) -> Vec<Change> {
    let mut out = Output::default();

    describe::chart_changes(old, new, &mut out.changed);

    compare(
        old.groups.as_slice(),
        new.groups.as_slice(),
        |group| group.id.clone(),
        group_identity,
        group_text,
        group_fields,
        &mut out,
    );
    compare(
        old.nodes.as_slice(),
        new.nodes.as_slice(),
        |node| node.id.clone(),
        node_identity,
        node_text,
        node_fields,
        &mut out,
    );
    let mut old_edges = keyed_edges(&old.edges);
    let mut new_edges = keyed_edges(&new.edges);
    old_edges.sort_by(|left, right| left.0.cmp(&right.0));
    new_edges.sort_by(|left, right| left.0.cmp(&right.0));
    compare_edges(&old_edges, &new_edges, &mut out);
    let mut old_notes = keyed_notes(&old.notes);
    let mut new_notes = keyed_notes(&new.notes);
    old_notes.sort_by(|left, right| left.0.cmp(&right.0));
    new_notes.sort_by(|left, right| left.0.cmp(&right.0));
    compare_notes(&old_notes, &new_notes, &mut out);

    out.added
        .sort_by(|left, right| left.element.cmp(&right.element));
    out.removed
        .sort_by(|left, right| left.element.cmp(&right.element));
    out.changed.sort_by(|left, right| {
        left.element
            .cmp(&right.element)
            .then_with(|| left.field.cmp(&right.field))
    });
    out.added
        .into_iter()
        .chain(out.removed)
        .chain(out.changed)
        .collect()
}

fn compare<T>(
    old: &[T],
    new: &[T],
    key: impl Fn(&T) -> String,
    identity: impl Fn(&T) -> String,
    describe: impl Fn(&T) -> String,
    fields: impl Fn(&T) -> Vec<(&'static str, String)>,
    out: &mut Output,
) {
    let old_map = index_of(old, &key);
    let new_map = index_of(new, &key);
    let mut keys: Vec<&String> = old_map.keys().chain(new_map.keys()).collect();
    keys.sort();
    keys.dedup();
    for id in keys {
        match (old_map.get(id), new_map.get(id)) {
            (Some(left), Some(right)) => {
                push_fields(
                    &identity(right),
                    &fields(left),
                    &fields(right),
                    &mut out.changed,
                );
            }
            (Some(left), None) => out.removed.push(Change::removed(describe(left))),
            (None, Some(right)) => out.added.push(Change::added(describe(right))),
            (None, None) => {}
        }
    }
}

/// Três listas de mudanças, na ordem em que vão para a saída.
#[derive(Debug, Default)]
struct Output {
    added: Vec<Change>,
    removed: Vec<Change>,
    changed: Vec<Change>,
}

fn index_of<'a, T>(items: &'a [T], key: &impl Fn(&T) -> String) -> BTreeMap<String, &'a T> {
    items.iter().map(|item| (key(item), item)).collect()
}

fn keyed_edges(edges: &[Edge]) -> Vec<(String, &Edge)> {
    edges
        .iter()
        .map(|edge| (format!("{}->{}#{}", edge.from, edge.to, edge.ordinal), edge))
        .collect()
}

fn keyed_notes(notes: &[Note]) -> Vec<(String, &Note)> {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    notes
        .iter()
        .map(|note| {
            let anchor = note.near.clone().unwrap_or_default();
            let index = seen.entry(anchor.clone()).or_insert(0);
            let key = format!("{anchor}#{index}");
            *index = index.saturating_add(1);
            (key, note)
        })
        .collect()
}

fn compare_edges(old: &[(String, &Edge)], new: &[(String, &Edge)], out: &mut Output) {
    for (id, edge) in new {
        match old.iter().find(|(key, _)| key == id) {
            Some((_, before)) => push_fields(
                &edge_identity(edge),
                &edge_fields(before),
                &edge_fields(edge),
                &mut out.changed,
            ),
            None => out.added.push(Change::added(edge_text(edge))),
        }
    }
    for (id, edge) in old {
        if !new.iter().any(|(key, _)| key == id) {
            out.removed.push(Change::removed(edge_text(edge)));
        }
    }
}

fn compare_notes(old: &[(String, &Note)], new: &[(String, &Note)], out: &mut Output) {
    for (id, note) in new {
        match old.iter().find(|(key, _)| key == id) {
            Some((_, before)) => push_fields(
                &note_identity(note),
                &note_fields(before),
                &note_fields(note),
                &mut out.changed,
            ),
            None => out.added.push(Change::added(note_text(note))),
        }
    }
    for (id, note) in old {
        if !new.iter().any(|(key, _)| key == id) {
            out.removed.push(Change::removed(note_text(note)));
        }
    }
}

fn push_fields(
    element: &str,
    before: &[(&'static str, String)],
    after: &[(&'static str, String)],
    changed: &mut Vec<Change>,
) {
    for ((field, old_value), (_, new_value)) in before.iter().zip(after.iter()) {
        if old_value != new_value {
            changed.push(Change {
                kind: Kind::Changed,
                element: element.to_string(),
                field: Some(field),
                before: Some(old_value.clone()),
                after: Some(new_value.clone()),
            });
        }
    }
}

#[cfg(test)]
mod tests;
