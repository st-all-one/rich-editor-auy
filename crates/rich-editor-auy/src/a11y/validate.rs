//! Stable accessibility diagnostics.
#![allow(
    clippy::excessive_nesting,
    reason = "explicit deterministic branch structure"
)]

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use crate::model::Doc;
use crate::pos::{DocPath, Range};
use crate::schema::Schema;

use super::codes::{A11yArg, A11yCode, A11yIssue, A11yReport, A11ySeverity};
use super::role::A11yRole;
use super::tree::{A11yNode, A11yTree};

impl Doc {
    /// Validates semantic accessibility invariants.
    #[must_use]
    pub fn validate_a11y(&self, schema: &Schema) -> A11yReport {
        let tree = self.a11y_tree(schema);
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut ids = BTreeSet::new();
        let mut heading = None;
        validate_root(schema, &tree, &mut errors, &mut warnings);
        for child in &tree.root.children {
            validate_node(
                child,
                &child.path,
                &mut ids,
                &mut heading,
                &mut errors,
                &mut warnings,
            );
        }
        errors.sort_by(|left, right| (&left.path, left.code).cmp(&(&right.path, right.code)));
        warnings.sort_by(|left, right| (&left.path, left.code).cmp(&(&right.path, right.code)));
        A11yReport {
            errors: errors.into_boxed_slice(),
            warnings: warnings.into_boxed_slice(),
        }
    }
}

fn validate_root(
    schema: &Schema,
    tree: &A11yTree,
    errors: &mut Vec<A11yIssue>,
    warnings: &mut Vec<A11yIssue>,
) {
    let lang_required = schema
        .doc()
        .attrs
        .iter()
        .any(|attr| &*attr.name == "lang" && attr.required);
    if lang_required && tree.lang.is_none() {
        push(
            errors,
            A11yCode::A007,
            A11ySeverity::Error,
            &DocPath::root(),
            tree.root.range,
            Vec::new(),
        );
    }
    if let Some(dir) = &tree.dir
        && !matches!(&**dir, "ltr" | "rtl" | "auto")
    {
        push(
            warnings,
            A11yCode::A008,
            A11ySeverity::Warning,
            &DocPath::root(),
            tree.root.range,
            Vec::from([A11yArg::Text(dir.clone())]),
        );
    }
}

#[allow(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "single-pass ordered validation"
)]
fn validate_node(
    node: &A11yNode,
    path: &DocPath,
    ids: &mut BTreeSet<crate::model::NodeId>,
    heading: &mut Option<u8>,
    errors: &mut Vec<A11yIssue>,
    warnings: &mut Vec<A11yIssue>,
) {
    if let Some(id) = &node.id
        && !ids.insert(id.clone())
    {
        push(
            errors,
            A11yCode::A009,
            A11ySeverity::Error,
            path,
            node.range,
            Vec::from([A11yArg::Text(Box::from(id.as_str()))]),
        );
    }
    match node.role {
        A11yRole::Image | A11yRole::Video | A11yRole::Audio | A11yRole::Attachment => {
            if node.alt.as_deref().is_none_or(str::is_empty) {
                push(
                    errors,
                    A11yCode::A001,
                    A11ySeverity::Error,
                    path,
                    node.range,
                    Vec::from([A11yArg::Text(Box::from(node.role.name()))]),
                );
            }
        }
        A11yRole::Link => {
            if node.name.as_deref().is_none_or(str::is_empty) {
                push(
                    errors,
                    A11yCode::A002,
                    A11ySeverity::Error,
                    path,
                    node.range,
                    Vec::new(),
                );
            }
        }
        A11yRole::Heading { .. } => {
            let level = node.level.unwrap_or(0);
            if !(1..=6).contains(&level) || node.name.as_deref().is_none_or(str::is_empty) {
                push(
                    errors,
                    A11yCode::A003,
                    A11ySeverity::Error,
                    path,
                    node.range,
                    Vec::from([A11yArg::Number(u32::from(level))]),
                );
            } else {
                if let Some(previous) = *heading
                    && level > previous.saturating_add(1)
                {
                    push(
                        warnings,
                        A11yCode::A004,
                        A11ySeverity::Warning,
                        path,
                        node.range,
                        Vec::from([
                            A11yArg::Number(u32::from(previous)),
                            A11yArg::Number(u32::from(level)),
                        ]),
                    );
                }
                *heading = Some(level);
            }
        }
        A11yRole::List => {
            if !node
                .children
                .iter()
                .any(|child| child.role == A11yRole::ListItem)
            {
                push(
                    errors,
                    A11yCode::A005,
                    A11ySeverity::Error,
                    path,
                    node.range,
                    Vec::new(),
                );
            }
        }
        A11yRole::Table => {
            let has_header = node
                .children
                .iter()
                .any(|child| matches!(child.role, A11yRole::TableCell { header: true }));
            if !has_header && node.caption.is_none() {
                push(
                    warnings,
                    A11yCode::A006,
                    A11ySeverity::Warning,
                    path,
                    node.range,
                    Vec::new(),
                );
            }
        }
        A11yRole::Separator if node.text.as_deref().is_none_or(str::is_empty) => {
            push(
                warnings,
                A11yCode::A010,
                A11ySeverity::Warning,
                path,
                node.range,
                Vec::new(),
            );
        }
        _ => {}
    }
    for child in &node.children {
        validate_node(child, &child.path, ids, heading, errors, warnings);
    }
}

#[allow(clippy::too_many_arguments, reason = "diagnostic constructor")]
fn push(
    list: &mut Vec<A11yIssue>,
    code: A11yCode,
    severity: A11ySeverity,
    path: &DocPath,
    range: Range,
    args: Vec<A11yArg>,
) {
    list.push(A11yIssue {
        code,
        severity,
        path: path.clone(),
        range,
        args: args.into_boxed_slice(),
    });
}
