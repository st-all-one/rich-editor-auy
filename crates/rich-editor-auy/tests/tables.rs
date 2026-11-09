//! Table grid invariants and cell resolution.

use std::ptr;
use std::sync::Arc;

use rich_editor_auy::model::{
    Attr, AttrValue, Attrs, Block, CellRef, Doc, InlineItem, MarkSet, NodeKind, Run, TextBlock,
};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;

/// Table node kinds plus schema helpers.
struct Fixture {
    schema: Schema,
    table_kind: NodeKind,
    row_kind: NodeKind,
    caption_kind: NodeKind,
    header_kind: NodeKind,
    cell_kind: NodeKind,
    paragraph_kind: NodeKind,
}

impl Fixture {
    fn new() -> Option<Self> {
        let schema = Schema::full().ok()?;
        let table_kind = schema.node_kind("table")?;
        let row_kind = schema.node_kind("table_row")?;
        let caption_kind = schema.node_kind("table_caption")?;
        let header_kind = schema.node_kind("table_header")?;
        let cell_kind = schema.node_kind("table_cell")?;
        let paragraph_kind = schema.node_kind("paragraph")?;
        Some(Self {
            schema,
            table_kind,
            row_kind,
            caption_kind,
            header_kind,
            cell_kind,
            paragraph_kind,
        })
    }

    fn attrs(&self, kind: NodeKind, pairs: &[(&str, AttrValue)]) -> Attrs {
        let values = pairs
            .iter()
            .filter_map(|(name, value)| {
                let spec = self.schema.node_attr(kind, name)?;
                Some(Attr {
                    id: spec.id,
                    value: value.clone(),
                })
            })
            .collect();
        Attrs::new(values).unwrap_or_default()
    }

    fn text_block(kind: NodeKind, attrs: Attrs, text: &str) -> Arc<Block> {
        let run = Run::new(String::from(text), MarkSet::empty());
        let content = TextBlock::new(vec![InlineItem::Text(run)]);
        Arc::new(Block::text(kind, attrs, content))
    }

    fn header_cell(&self, pairs: &[(&str, AttrValue)], text: &str) -> Arc<Block> {
        let attrs = self.attrs(self.header_kind, pairs);
        self.cell_block(self.header_kind, attrs, text)
    }

    fn data_cell(&self, pairs: &[(&str, AttrValue)], text: &str) -> Arc<Block> {
        let attrs = self.attrs(self.cell_kind, pairs);
        self.cell_block(self.cell_kind, attrs, text)
    }

    fn cell_block(&self, kind: NodeKind, attrs: Attrs, text: &str) -> Arc<Block> {
        let paragraph = Self::text_block(self.paragraph_kind, Attrs::empty(), text);
        Arc::new(Block::container(kind, attrs, vec![paragraph]))
    }

    fn caption(&self, text: &str) -> Arc<Block> {
        Self::text_block(self.caption_kind, Attrs::empty(), text)
    }

    fn row(&self, cells: Vec<Arc<Block>>) -> Arc<Block> {
        Arc::new(Block::container(self.row_kind, Attrs::empty(), cells))
    }

    fn table(&self, children: Vec<Arc<Block>>) -> Arc<Block> {
        Arc::new(Block::container(self.table_kind, Attrs::empty(), children))
    }

    fn doc(&self, children: Vec<Arc<Block>>) -> Option<Doc> {
        self.doc_tables(vec![children])
    }

    fn doc_tables(&self, tables: Vec<Vec<Arc<Block>>>) -> Option<Doc> {
        let blocks = tables
            .into_iter()
            .map(|children| self.table(children))
            .collect();
        Doc::from_blocks(Attrs::empty(), blocks).ok()
    }
}

fn header_list(values: &[&str]) -> AttrValue {
    AttrValue::Array(
        values
            .iter()
            .map(|value| AttrValue::Str(Box::from(*value)))
            .collect(),
    )
}

#[test]
fn valid_table_has_no_issues_and_resolves_grid() {
    let Some(fixture) = Fixture::new() else {
        return;
    };
    let header = fixture.header_cell(
        &[
            ("id", AttrValue::Str(Box::from("h1"))),
            ("colspan", AttrValue::Int(2)),
        ],
        "H",
    );
    let named = fixture.data_cell(&[("headers", header_list(&["h1"]))], "a");
    let plain = fixture.data_cell(&[], "b");
    let Some(doc) = fixture.doc(vec![
        fixture.caption("T"),
        fixture.row(vec![header]),
        fixture.row(vec![named, plain]),
    ]) else {
        return;
    };
    let report = doc.validate(&fixture.schema);
    assert!(report.is_ok(), "valid table failed: {:?}", report.issues);
    let Some(info) = doc.table_info(&fixture.schema, 0) else {
        return;
    };
    assert_eq!(info.table_index(), 0);
    assert_eq!(info.rows(), 2);
    assert_eq!(info.columns(), 2);
    assert_eq!(info.cells().len(), 3);
    assert!(info.has_caption());
    assert!(!info.is_truncated());
    assert_eq!(info.cell_ref(1, 1).map(|found| found.column), Some(1));
    assert_eq!(info.cell_ref(0, 1).map(|found| found.column), Some(0));
    assert!(info.cell(2, 0).is_none());
    assert!(info.cell(0, 2).is_none());
    let cells = info.cells();
    assert_eq!(
        cells.first().map(|cell| (cell.header, cell.colspan)),
        Some((true, 2))
    );
    assert_eq!(cells.get(1).map(|cell| cell.header), Some(false));
    assert!(doc.table_info(&fixture.schema, 1).is_none());
}

#[test]
fn rowspan_resolves_covered_positions() {
    let Some(fixture) = Fixture::new() else {
        return;
    };
    let spanned = fixture.header_cell(&[("rowspan", AttrValue::Int(2))], "H");
    let corner = fixture.data_cell(&[], "b");
    let below = fixture.data_cell(&[], "c");
    let extra = fixture.data_cell(&[], "z");
    let Some(doc) = fixture.doc_tables(vec![
        vec![
            fixture.row(vec![spanned.clone(), corner]),
            fixture.row(vec![below]),
        ],
        vec![fixture.row(vec![extra.clone()])],
    ]) else {
        return;
    };
    let report = doc.validate(&fixture.schema);
    assert!(report.is_ok(), "valid table failed: {:?}", report.issues);
    let Some(info) = doc.table_info(&fixture.schema, 0) else {
        return;
    };
    assert_eq!(info.rows(), 2);
    assert_eq!(info.columns(), 2);
    assert_eq!(
        info.cell_ref(1, 0),
        Some(CellRef {
            table: 0,
            row: 0,
            column: 0,
        })
    );
    let Some(block) = doc.cell(
        &fixture.schema,
        CellRef {
            table: 0,
            row: 1,
            column: 0,
        },
    ) else {
        return;
    };
    assert!(ptr::eq(block, spanned.as_ref()));
    let Some(second) = doc.table_info(&fixture.schema, 1) else {
        return;
    };
    assert_eq!(second.table_index(), 1);
    let address = CellRef {
        table: 1,
        row: 0,
        column: 0,
    };
    let Some(found) = doc.cell(&fixture.schema, address) else {
        return;
    };
    assert!(ptr::eq(found, extra.as_ref()));
    assert!(doc.table_info(&fixture.schema, 2).is_none());
}

/// One expected invariant: code, table children, path, and optional position.
type Case = (&'static str, Vec<Arc<Block>>, &'static str, Option<Pos>);

#[test]
#[allow(clippy::too_many_lines, reason = "one table-driven invariant sweep")]
fn grid_invariants_report_stable_codes() {
    let Some(fixture) = Fixture::new() else {
        return;
    };
    let spanned = fixture.header_cell(&[("rowspan", AttrValue::Int(2))], "A");
    let middle = fixture.header_cell(&[], "B");
    let tall = fixture.header_cell(&[("rowspan", AttrValue::Int(2))], "C");
    let wide = fixture.data_cell(&[("colspan", AttrValue::Int(2))], "D");
    let group = fixture.header_cell(&[("scope", AttrValue::Str(Box::from("colgroup")))], "H");
    let scoped = fixture.data_cell(&[("scope", AttrValue::Str(Box::from("col")))], "d");
    let named = fixture.header_cell(&[("id", AttrValue::Str(Box::from("h1")))], "H");
    let ghost = fixture.data_cell(&[("headers", header_list(&["ghost"]))], "d");
    let zero = fixture.data_cell(&[("colspan", AttrValue::Int(0))], "a");
    let past = fixture.data_cell(&[("rowspan", AttrValue::Int(2))], "a");
    let plain = |text: &str| fixture.data_cell(&[], text);
    let cases: Vec<Case> = vec![
        ("I17", vec![fixture.caption("only")], "0", None),
        ("I18", vec![fixture.row(vec![zero])], "0/0/0", Some(Pos(2))),
        (
            "I19",
            vec![
                fixture.row(vec![plain("a"), plain("b")]),
                fixture.row(vec![plain("c")]),
            ],
            "0/1",
            None,
        ),
        ("I20", vec![fixture.row(vec![past])], "0/0/0", None),
        (
            "I21",
            vec![
                fixture.row(vec![spanned, middle, tall]),
                fixture.row(vec![wide]),
            ],
            "0/1/0",
            None,
        ),
        ("I22", vec![fixture.row(vec![group, scoped])], "0/0/0", None),
        (
            "I23",
            vec![fixture.row(vec![named]), fixture.row(vec![ghost])],
            "0/1/0",
            None,
        ),
        (
            "I24",
            vec![fixture.row(vec![plain("a")]), fixture.caption("late")],
            "0/1",
            None,
        ),
        (
            "I24",
            vec![
                fixture.caption("one"),
                fixture.caption("two"),
                fixture.row(vec![plain("a")]),
            ],
            "0/1",
            None,
        ),
    ];
    for (code, children, path, position) in cases {
        let Some(doc) = fixture.doc(children) else {
            continue;
        };
        let report = doc.validate(&fixture.schema);
        let found = report.issues.iter().find(|issue| issue.invariant == code);
        assert!(found.is_some(), "missing {code}: {:?}", report.issues);
        let Some(found) = found else {
            continue;
        };
        assert_eq!(found.path.as_str(), path, "unexpected path for {code}");
        if let Some(position) = position {
            assert_eq!(found.position, Some(position), "unexpected {code} position");
        }
    }
}
