//! Accessibility diagnostics: stable codes, severities, arguments, and reports.

use crate::pos::{DocPath, Range};
use alloc::boxed::Box;

/// Stable accessibility diagnostic code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum A11yCode {
    /// Missing required alternative text.
    A001,
    /// Missing required accessible name.
    A002,
    /// Invalid heading.
    A003,
    /// Heading level jump.
    A004,
    /// Invalid list structure.
    A005,
    /// Table without caption or header.
    A006,
    /// Missing required document language.
    A007,
    /// Invalid direction.
    A008,
    /// Duplicate node identifier.
    A009,
    /// Purely visual content without an alternative.
    A010,
}

impl A11yCode {
    /// Returns the stable code string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::A001 => "A001",
            Self::A002 => "A002",
            Self::A003 => "A003",
            Self::A004 => "A004",
            Self::A005 => "A005",
            Self::A006 => "A006",
            Self::A007 => "A007",
            Self::A008 => "A008",
            Self::A009 => "A009",
            Self::A010 => "A010",
        }
    }
}

/// Issue severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum A11ySeverity {
    /// Blocks a strict report.
    Error,
    /// Diagnostic only.
    Warning,
}

/// Structured argument for a localized message catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(
    variant_size_differences,
    reason = "diagnostic arguments are cold data"
)]
pub enum A11yArg {
    /// Text argument.
    Text(Box<str>),
    /// Numeric argument.
    Number(u32),
    /// Node kind argument.
    NodeKind(crate::model::NodeKind),
    /// Attribute argument.
    AttrId(crate::model::AttrId),
}

/// One accessibility issue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct A11yIssue {
    /// Stable code.
    pub code: A11yCode,
    /// Severity.
    pub severity: A11ySeverity,
    /// Node path.
    pub path: DocPath,
    /// Logical range.
    pub range: Range,
    /// Message arguments.
    pub args: Box<[A11yArg]>,
}

/// Deterministic, sorted report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct A11yReport {
    /// Errors sorted by `(path, code)`.
    pub errors: Box<[A11yIssue]>,
    /// Warnings sorted by `(path, code)`.
    pub warnings: Box<[A11yIssue]>,
}

impl A11yReport {
    /// Returns whether the report has no errors.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Returns whether the report has errors or warnings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}
