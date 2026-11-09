//! Semantic accessibility projections and validation.

mod attrs;
mod build;
pub mod codes;
mod json;
mod marks;
pub mod role;
mod text;
pub mod tree;
pub mod validate;

pub use codes::{A11yArg, A11yCode, A11yIssue, A11yReport, A11ySeverity};
pub use role::{A11yRole, A11ySpec, AltPolicy, NamePolicy, RoleId};
pub use tree::{A11yNode, A11yTree, CellScope};
