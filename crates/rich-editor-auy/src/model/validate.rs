//! Structural validation against a schema.
#![allow(
    clippy::excessive_nesting,
    clippy::too_many_lines,
    reason = "one explicit deterministic validation walk"
)]

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::pos::{DocPath, Pos};
use crate::schema::Schema;

use super::table::{TableProblem, scan_table};
use super::{Block, BlockContent, Doc, InlineItem, MAX_RUN_BYTES, MarkSet, NodeId, NodeKind};

/// One structural invariant violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocIssue {
    /// Stable invariant identifier (`I1`–`I12`, `I16`–`I25`).
    pub invariant: &'static str,
    /// Stable structural path.
    pub path: DocPath,
    /// Byte position, when the violation is positional.
    pub position: Option<Pos>,
}

/// Result of structural document validation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocReport {
    /// Violations in deterministic document order.
    pub issues: Vec<DocIssue>,
}

impl DocReport {
    /// Returns whether the document satisfies every checked invariant.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }
}

impl Doc {
    /// Validates structural invariants against a schema.
    #[must_use]
    pub fn validate(&self, schema: &Schema) -> DocReport {
        let mut report = DocReport::default();
        let mut ids = BTreeSet::new();
        let kind_names = self
            .blocks()
            .iter()
            .map(|block| block.kind())
            .collect::<Vec<_>>();
        if !schema.doc_allows(&kind_names) {
            report.issues.push(issue("I1", &[], None));
        }
        for attr in self.attrs() {
            let Some(spec) = schema.doc().attrs.iter().find(|spec| spec.id == attr.id) else {
                report.issues.push(issue("I7", &[], Some(Pos::ZERO)));
                continue;
            };
            if !Schema::value_matches(&spec.kind, &attr.value) {
                report.issues.push(issue("I7", &[], Some(Pos::ZERO)));
            }
        }
        let mut offset = 0_u32;
        let mut path = Vec::new();
        let text_kind = schema.node_kind("text");
        let table_kind = schema.node_kind("table");
        for (index, block) in self.blocks().iter().enumerate() {
            path.push(u32::try_from(index).unwrap_or(u32::MAX));
            validate_block(
                block,
                schema,
                &mut path,
                Pos(offset),
                &mut ids,
                &mut report,
                text_kind,
                table_kind,
            );
            path.pop();
            offset = offset.saturating_add(block.size().0);
        }
        if offset != self.size().0 {
            report.issues.push(issue("I10", &[], Some(Pos(offset))));
        }
        report
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "validation walks inherited path and document position"
)]
fn validate_block(
    block: &Block,
    schema: &Schema,
    path: &mut Vec<u32>,
    start: Pos,
    ids: &mut BTreeSet<NodeId>,
    report: &mut DocReport,
    text_kind: Option<NodeKind>,
    table_kind: Option<NodeKind>,
) {
    let Some(spec) = schema.node(block.kind()) else {
        report.issues.push(issue("I2", path, Some(start)));
        return;
    };
    if schema.validate_attrs(block.kind(), block.attrs()).is_err() {
        report.issues.push(issue("I7", path, Some(start)));
    }
    if let Some(id) = block.id()
        && !ids.insert(id)
    {
        report.issues.push(issue("I16", path, Some(start)));
    }
    match block.content() {
        BlockContent::Text(text) => {
            if block.size().0 != text.size().0 {
                report.issues.push(issue("I10", path, Some(start)));
            }
            let accepts_text = spec.inline
                || text_kind.is_some_and(|kind| schema.content_allows(block.kind(), &[kind]))
                || schema.content_allows(block.kind(), &[]);
            if spec.atom || !accepts_text {
                report.issues.push(issue("I2", path, Some(start)));
            }
            let mut previous = None;
            for item in text.items() {
                match item {
                    InlineItem::Text(run) => {
                        if run.is_empty() {
                            report.issues.push(issue("I3", path, Some(start)));
                        }
                        if run.text.len() > MAX_RUN_BYTES {
                            report.issues.push(issue("I4", path, Some(start)));
                        }
                        if previous
                            .as_ref()
                            .is_some_and(|marks: &MarkSet| marks == &run.marks)
                        {
                            report.issues.push(issue("I4", path, Some(start)));
                        }
                        previous = Some(run.marks.clone());
                        if !run.marks.is_empty() {
                            validate_marks(schema, block.kind(), &run.marks, path, start, report);
                        }
                    }
                    InlineItem::Atom(atom) => {
                        previous = None;
                        if schema.node(atom.kind).is_none()
                            || schema.validate_attrs(atom.kind, &atom.attrs).is_err()
                        {
                            report.issues.push(issue("I7", path, Some(start)));
                        }
                    }
                }
            }
        }
        BlockContent::Blocks(children) => {
            let kinds = children
                .iter()
                .map(|child| child.kind())
                .collect::<Vec<_>>();
            if !schema.content_allows(block.kind(), &kinds) {
                report.issues.push(issue("I2", path, Some(start)));
            }
            if children.is_empty() && !schema.content_can_be_empty(block.kind()) {
                report.issues.push(issue("I8", path, Some(start)));
            }
            let computed = children
                .iter()
                .fold(2_u32, |total, child| total.saturating_add(child.size().0));
            if block.size().0 != computed {
                report.issues.push(issue("I10", path, Some(start)));
            }
            let mut offset = start.0.saturating_add(1);
            for (index, child) in children.iter().enumerate() {
                path.push(u32::try_from(index).unwrap_or(u32::MAX));
                validate_block(
                    child,
                    schema,
                    path,
                    Pos(offset),
                    ids,
                    report,
                    text_kind,
                    table_kind,
                );
                path.pop();
                offset = offset.saturating_add(child.size().0);
            }
        }
        BlockContent::Atom(_) if block.size().0 != 1 => {
            report.issues.push(issue("I10", path, Some(start)));
        }
        BlockContent::Atom(_) => {}
    }
    if Some(block.kind()) == table_kind {
        let Some(scan) = scan_table(0, block, schema) else {
            return;
        };
        for problem in &scan.problems {
            report.issues.push(table_issue(block, start, path, problem));
        }
    }
}

/// Maps one grid problem to a structural issue with its document path and position.
fn table_issue(block: &Block, start: Pos, path: &[u32], problem: &TableProblem) -> DocIssue {
    let mut issue_path = path.to_vec();
    let mut position = start;
    if let Some(row) = problem.row
        && let Some((row_pos, row_block)) = child_at(block, start, row)
    {
        issue_path.push(row);
        position = row_pos;
        if let Some(cell) = problem.cell
            && let Some((cell_pos, _)) = child_at(row_block, row_pos, cell)
        {
            issue_path.push(cell);
            position = cell_pos;
        }
    }
    issue(problem.invariant, &issue_path, Some(position))
}

/// Returns the start position of one child block.
fn child_at(block: &Block, start: Pos, index: u32) -> Option<(Pos, &Block)> {
    let target = usize::try_from(index).ok()?;
    let mut cursor = start.0.saturating_add(1);
    for (offset, child) in block.children()?.iter().enumerate() {
        if offset == target {
            return Some((Pos(cursor), child));
        }
        cursor = cursor.saturating_add(child.size().0);
    }
    None
}

#[allow(
    clippy::too_many_arguments,
    reason = "validation walks inherited path and document position"
)]
fn validate_marks(
    schema: &Schema,
    kind: NodeKind,
    marks: &MarkSet,
    path: &[u32],
    start: Pos,
    report: &mut DocReport,
) {
    for mark in schema.marks() {
        if !marks.contains(mark.id) {
            continue;
        }
        if !schema.mark_allowed(kind, mark.id) {
            report.issues.push(issue("I6", path, Some(start)));
            continue;
        }
        for excluded in &mark.excludes {
            if let Some(id) = schema.mark_id(excluded)
                && marks.contains(id)
            {
                report.issues.push(issue("I6", path, Some(start)));
            }
        }
        if let Some(attrs) = marks.attrs_for(mark.id) {
            for attr in attrs {
                let Some(spec) = mark.attrs.iter().find(|spec| spec.id == attr.id) else {
                    report.issues.push(issue("I7", path, Some(start)));
                    continue;
                };
                if !Schema::value_matches(&spec.kind, &attr.value) {
                    report.issues.push(issue("I7", path, Some(start)));
                }
                if crate::schema::is_url_attr(&spec.name)
                    && let crate::model::AttrValue::Str(url) = &attr.value
                    && crate::schema::is_unsafe_url(url)
                {
                    report.issues.push(issue("I7", path, Some(start)));
                }
            }
        }
    }
}

fn issue(invariant: &'static str, path: &[u32], position: Option<Pos>) -> DocIssue {
    let mut doc_path = DocPath::root();
    for component in path {
        doc_path = doc_path.child(*component);
    }
    DocIssue {
        invariant,
        path: doc_path,
        position,
    }
}
