//! Node declarations for the complete schema.

use crate::a11y::{A11yRole, A11ySpec, NamePolicy};
use crate::model::{AttrId, AttrValue};
use crate::schema::{AttrType, NodeSpec};

use super::attrs::{bool_attr, enum_attr, int_attr, list_attr, str_attr};

const ALIGN: &[&str] = &["start", "end", "left", "right", "center", "justify"];
const SCOPE: &[&str] = &["col", "row", "colgroup", "rowgroup"];
const CODE_FORMATS: &[&str] = &["xml", "html", "json", "yaml", "sql", "text", "code"];

pub(super) fn text() -> NodeSpec {
    NodeSpec::new("text")
        .group("inline")
        .inline()
        .a11y(A11ySpec::new(A11yRole::Text).with_name(NamePolicy::TextContent))
}

pub(super) fn paragraph() -> NodeSpec {
    NodeSpec::new("paragraph")
        .group("block")
        .content("inline*")
        .marks_all()
        .attr(enum_attr("align", ALIGN))
        .a11y(A11ySpec::new(A11yRole::Paragraph).with_name(NamePolicy::TextContent))
}

pub(super) fn heading() -> NodeSpec {
    NodeSpec::new("heading")
        .group("block")
        .content("inline*")
        .marks_all()
        .attr(
            int_attr("level", 1, 6)
                .required()
                .with_default(AttrValue::Int(1)),
        )
        .attr(enum_attr("align", ALIGN))
        .a11y(
            A11ySpec::new(A11yRole::Heading { level: 1 })
                .with_name(NamePolicy::TextContent)
                .with_level(AttrId(1)),
        )
}

pub(super) fn bullet_list() -> NodeSpec {
    NodeSpec::new("bullet_list")
        .group("block")
        .content("list_item+")
        .attr(bool_attr("tight"))
        .attr(bool_attr("checked"))
        .a11y(A11ySpec::new(A11yRole::List))
}

pub(super) fn ordered_list() -> NodeSpec {
    NodeSpec::new("ordered_list")
        .group("block")
        .content("list_item+")
        .attr(int_attr("start", 1, 1_000_000))
        .attr(bool_attr("reversed"))
        .attr(bool_attr("tight"))
        .attr(enum_attr("type", &["1", "a", "A", "i", "I"]))
        .a11y(A11ySpec::new(A11yRole::List))
}

pub(super) fn list_item() -> NodeSpec {
    NodeSpec::new("list_item")
        .content("block+")
        .attr(bool_attr("checked"))
        .a11y(A11ySpec::new(A11yRole::ListItem))
}

pub(super) fn blockquote() -> NodeSpec {
    NodeSpec::new("blockquote")
        .group("block")
        .content("block+")
        .a11y(A11ySpec::new(A11yRole::Quote))
}

pub(super) fn code_block() -> NodeSpec {
    NodeSpec::new("code_block")
        .group("block")
        .content("text*")
        .attr(str_attr("language"))
        .attr(str_attr("meta"))
        .attr(bool_attr("line_numbers"))
        .a11y(A11ySpec::new(A11yRole::CodeBlock).with_name(NamePolicy::TextContent))
}

pub(super) fn raw_code() -> NodeSpec {
    NodeSpec::new("raw_code")
        .group("block")
        .content("text*")
        .attr(str_attr("language"))
        .attr(enum_attr("format", CODE_FORMATS))
        .attr(str_attr("filename"))
        .attr(str_attr("meta"))
        .attr(bool_attr("line_numbers"))
        .a11y(A11ySpec::new(A11yRole::CodeBlock).with_name(NamePolicy::TextContent))
}

pub(super) fn table() -> NodeSpec {
    NodeSpec::new("table")
        .group("block")
        .content("table_caption? table_row+")
        .attr(str_attr("summary"))
        .attr(list_attr(
            "col_widths",
            AttrType::Int {
                min: 1,
                max: 1_000_000,
            },
        ))
        .a11y(A11ySpec::new(A11yRole::Table))
}

pub(super) fn table_caption() -> NodeSpec {
    NodeSpec::new("table_caption")
        .content("text*")
        .marks_all()
        .a11y(A11ySpec::new(A11yRole::TableCaption).with_name(NamePolicy::TextContent))
}

pub(super) fn table_row() -> NodeSpec {
    NodeSpec::new("table_row")
        .content("(table_header | table_cell)+")
        .a11y(A11ySpec::new(A11yRole::TableRow))
}

fn cell(name: &str, role: A11yRole) -> NodeSpec {
    NodeSpec::new(name)
        .content("block+")
        .marks_all()
        .attr(int_attr("colspan", 1, 1_000))
        .attr(int_attr("rowspan", 1, 1_000))
        .attr(enum_attr("scope", SCOPE))
        .attr(enum_attr("align", ALIGN))
        .attr(str_attr("abbr"))
        .attr(list_attr("headers", AttrType::Str))
        .a11y(A11ySpec::new(role).with_name(NamePolicy::TextContent))
}

pub(super) fn table_header() -> NodeSpec {
    cell("table_header", A11yRole::TableCell { header: true })
}

pub(super) fn table_cell() -> NodeSpec {
    cell("table_cell", A11yRole::TableCell { header: false })
}

pub(super) fn figure() -> NodeSpec {
    NodeSpec::new("figure")
        .group("block")
        .content("image_block caption?")
        .a11y(A11ySpec::new(A11yRole::Figure))
}

pub(super) fn caption() -> NodeSpec {
    NodeSpec::new("caption")
        .content("text*")
        .marks_all()
        .a11y(A11ySpec::new(A11yRole::Caption).with_name(NamePolicy::TextContent))
}

pub(super) fn separator() -> NodeSpec {
    NodeSpec::new("separator")
        .group("block")
        .content("")
        .a11y(A11ySpec::new(A11yRole::Separator))
}

fn image_spec(name: &str) -> NodeSpec {
    NodeSpec::new(name)
        .atom()
        .attr(str_attr("alt").required())
        .attr(str_attr("src"))
        .attr(str_attr("title"))
        .attr(int_attr("width", 1, 100_000))
        .attr(int_attr("height", 1, 100_000))
        .attr(int_attr("intrinsic_width", 1, 100_000))
        .attr(int_attr("intrinsic_height", 1, 100_000))
        .attr(str_attr("srcset"))
        .attr(str_attr("sizes"))
        .attr(str_attr("caption"))
        .attr(str_attr("credit"))
        .attr(enum_attr("loading", &["lazy", "eager"]))
        .attr(enum_attr("decoding", &["sync", "async", "auto"]))
        .attr(str_attr("asset_id"))
        .attr(str_attr("mime"))
        .attr(int_attr("size_bytes", 0, 9_007_199_254_740_991))
        .a11y(A11ySpec::new(A11yRole::Image).with_name(NamePolicy::Attr(AttrId(1))))
}

pub(super) fn image() -> NodeSpec {
    image_spec("image").group("inline").inline()
}

pub(super) fn image_block() -> NodeSpec {
    image_spec("image_block")
}

pub(super) fn video() -> NodeSpec {
    NodeSpec::new("video")
        .group("inline")
        .inline()
        .atom()
        .attr(str_attr("alt").required())
        .attr(str_attr("src"))
        .attr(str_attr("poster"))
        .attr(str_attr("title"))
        .attr(str_attr("transcript"))
        .attr(bool_attr("controls"))
        .attr(str_attr("asset_id"))
        .attr(str_attr("mime"))
        .attr(int_attr("size_bytes", 0, 9_007_199_254_740_991))
        .attr(int_attr("width", 1, 100_000))
        .attr(int_attr("height", 1, 100_000))
        .attr(int_attr("intrinsic_width", 1, 100_000))
        .attr(int_attr("intrinsic_height", 1, 100_000))
        .a11y(A11ySpec::new(A11yRole::Video).with_name(NamePolicy::Attr(AttrId(1))))
}

pub(super) fn audio() -> NodeSpec {
    NodeSpec::new("audio")
        .group("inline")
        .inline()
        .atom()
        .attr(str_attr("alt").required())
        .attr(str_attr("src"))
        .attr(str_attr("title"))
        .attr(str_attr("transcript"))
        .attr(bool_attr("controls"))
        .attr(str_attr("asset_id"))
        .attr(str_attr("mime"))
        .attr(int_attr("size_bytes", 0, 9_007_199_254_740_991))
        .attr(int_attr("width", 1, 100_000))
        .attr(int_attr("height", 1, 100_000))
        .attr(int_attr("intrinsic_width", 1, 100_000))
        .attr(int_attr("intrinsic_height", 1, 100_000))
        .a11y(A11ySpec::new(A11yRole::Audio).with_name(NamePolicy::Attr(AttrId(1))))
}

pub(super) fn attachment() -> NodeSpec {
    NodeSpec::new("attachment")
        .group("inline")
        .inline()
        .atom()
        .attr(str_attr("alt").required())
        .attr(str_attr("src"))
        .attr(str_attr("filename"))
        .attr(str_attr("mime"))
        .attr(int_attr("size_bytes", 0, 9_007_199_254_740_991))
        .attr(str_attr("asset_id"))
        .attr(int_attr("width", 1, 100_000))
        .attr(int_attr("height", 1, 100_000))
        .attr(int_attr("intrinsic_width", 1, 100_000))
        .attr(int_attr("intrinsic_height", 1, 100_000))
        .a11y(A11ySpec::new(A11yRole::Attachment).with_name(NamePolicy::Attr(AttrId(1))))
}

pub(super) fn hard_break() -> NodeSpec {
    NodeSpec::new("hard_break").group("inline").inline().atom()
}
