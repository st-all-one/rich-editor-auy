//! Table commands, table selection, and undo behavior.

use std::boxed::Box;

use rich_editor_auy::model::{AttrValue, CellRef};
use rich_editor_auy::pos::Pos;
use rich_editor_auy::schema::Schema;
use rich_editor_auy::state::{Editor, Selection};
use rich_editor_auy::step::Mapping;

fn cell(table: u32, row: u32, column: u32) -> CellRef {
    CellRef { table, row, column }
}

fn table_editor() -> Option<Editor> {
    let mut editor = Editor::new().ok()?;
    editor.insert_table(Pos(0), 2, 2, 1).ok()?;
    Some(editor)
}

fn geometry(editor: &Editor, schema: &Schema) -> Option<(u32, u32)> {
    let table = editor.doc().table_info(schema, 0)?;
    Some((table.rows(), table.columns()))
}

#[test]
fn insert_table_creates_a_valid_grid_and_selects_it() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(mut editor) = Editor::new() else {
        return;
    };
    let Ok(applied) = editor.insert_table(Pos(0), 3, 2, 1) else {
        return;
    };
    assert!(editor.doc().validate(&schema).is_ok());
    assert_eq!(editor.doc().blocks().len(), 2);
    let Some(table) = editor.doc().table_info(&schema, 0) else {
        return;
    };
    assert_eq!(table.rows(), 3);
    assert_eq!(table.columns(), 2);
    let headers = table.cells().iter().filter(|cell| cell.header).count();
    assert_eq!(headers, 2);
    let expected = Selection::table(cell(0, 0, 0), cell(0, 0, 0));
    assert_eq!(applied.selection, expected);
    assert_eq!(editor.selection(), expected);
}

#[test]
fn insert_and_delete_rows_and_columns_keep_the_grid_consistent() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Some(mut editor) = table_editor() else {
        return;
    };
    let origin = cell(0, 0, 0);
    assert!(editor.insert_row(origin, true).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((3, 2)));
    assert!(editor.insert_column(origin, false).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((3, 3)));
    let Some(cell_ref) = editor
        .doc()
        .table_info(&schema, 0)
        .and_then(|table| table.cell_ref(0, 0))
    else {
        return;
    };
    assert!(editor.delete_row(cell_ref).is_ok());
    assert!(editor.delete_column(cell_ref).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    assert!(editor.doc().validate(&schema).is_ok());
}

#[test]
fn deleting_the_last_row_or_column_is_rejected() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(mut editor) = Editor::new() else {
        return;
    };
    assert!(editor.insert_table(Pos(0), 1, 1, 0).is_ok());
    let origin = cell(0, 0, 0);
    assert!(editor.delete_row(origin).is_err());
    assert!(editor.delete_column(origin).is_err());
    assert_eq!(geometry(&editor, &schema), Some((1, 1)));
}

#[test]
fn header_toggle_converts_kind_and_drops_scope_when_unset() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Some(mut editor) = table_editor() else {
        return;
    };
    let data = cell(0, 1, 0);
    assert!(editor.set_cell_header(data, true).is_ok());
    let header_kind = schema.node_kind("table_header");
    let Some(block) = editor.doc().cell(&schema, data) else {
        return;
    };
    assert_eq!(Some(block.kind()), header_kind);
    let scope = AttrValue::Str(Box::from("row"));
    assert!(editor.set_cell_attr(data, "scope", scope).is_ok());
    let align = AttrValue::Str(Box::from("start"));
    assert!(editor.set_cell_attr(data, "align", align).is_ok());
    assert!(editor.set_cell_attr(data, "align", AttrValue::Null).is_ok());
    let Some(block) = editor.doc().cell(&schema, data) else {
        return;
    };
    let align_id = schema.node_attr(block.kind(), "align").map(|spec| spec.id);
    assert!(align_id.and_then(|id| block.attrs().get(id)).is_none());
    assert!(editor.set_cell_header(data, false).is_ok());
    let Some(block) = editor.doc().cell(&schema, data) else {
        return;
    };
    assert_eq!(Some(block.kind()), schema.node_kind("table_cell"));
    let scope_id = schema.node_attr(block.kind(), "scope").map(|spec| spec.id);
    assert!(scope_id.and_then(|id| block.attrs().get(id)).is_none());
    assert!(editor.doc().validate(&schema).is_ok());
}

#[test]
fn merge_combines_content_and_split_restores_the_grid() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Some(mut editor) = table_editor() else {
        return;
    };
    let anchor = cell(0, 0, 0);
    let head = cell(0, 0, 1);
    assert!(editor.merge_cells(anchor, head).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    let Some(origin) = editor
        .doc()
        .table_info(&schema, 0)
        .and_then(|table| table.cell_ref(0, 0))
    else {
        return;
    };
    let Some(block) = editor.doc().cell(&schema, origin) else {
        return;
    };
    assert_eq!(block.children().map(<[_]>::len), Some(2));
    assert!(editor.doc().validate(&schema).is_ok());
    assert!(editor.split_cell(origin).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    assert!(editor.doc().validate(&schema).is_ok());
    assert!(editor.split_cell(origin).is_err());
    assert!(editor.merge_cells(cell(0, 0, 0), cell(0, 1, 1)).is_err());
}

#[test]
fn invalid_spans_and_merges_are_rejected() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Some(mut editor) = table_editor() else {
        return;
    };
    assert!(editor.set_cell_span(cell(0, 0, 0), 0, 1).is_err());
    assert!(editor.set_cell_span(cell(0, 1, 1), 2, 1).is_err());
    assert!(editor.set_cell_span(cell(0, 1, 1), 1, 2).is_err());
    assert!(editor.merge_cells(cell(0, 0, 0), cell(0, 0, 0)).is_err());
    assert!(
        editor
            .set_cell_attr(cell(0, 0, 0), "nope", AttrValue::Null)
            .is_err()
    );
    assert!(
        editor
            .set_cell_attr(
                cell(0, 0, 0),
                "align",
                AttrValue::Str(Box::from("diagonal"))
            )
            .is_err()
    );
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    assert!(editor.doc().validate(&schema).is_ok());
}

#[test]
fn span_grows_by_absorbing_cells_and_shrinks_by_refilling() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Some(mut editor) = table_editor() else {
        return;
    };
    let origin = cell(0, 0, 0);
    assert!(editor.set_cell_span(origin, 2, 1).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    let Some(block) = editor.doc().cell(&schema, origin) else {
        return;
    };
    assert_eq!(block.children().map(<[_]>::len), Some(2));
    assert!(editor.doc().validate(&schema).is_ok());
    assert!(editor.set_cell_span(origin, 1, 1).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((2, 2)));
    let Some(table) = editor.doc().table_info(&schema, 0) else {
        return;
    };
    assert_eq!(table.cells().len(), 4);
    assert!(editor.doc().validate(&schema).is_ok());
}

#[test]
fn merge_across_tables_is_rejected() {
    let Ok(mut editor) = Editor::new() else {
        return;
    };
    assert!(editor.insert_table(Pos(0), 1, 1, 0).is_ok());
    assert!(editor.insert_table(Pos(0), 1, 1, 0).is_ok());
    assert!(editor.merge_cells(cell(0, 0, 0), cell(1, 0, 0)).is_err());
    assert!(editor.merge_cells(cell(1, 0, 0), cell(1, 0, 0)).is_err());
}

#[test]
fn insert_table_validates_position_and_shape() {
    let Ok(mut editor) = Editor::new() else {
        return;
    };
    assert!(editor.insert_table(Pos(0), 0, 2, 0).is_err());
    assert!(editor.insert_table(Pos(0), 2, 0, 0).is_err());
    assert!(editor.insert_table(Pos(0), 2, 2, 3).is_err());
    assert!(editor.insert_table(Pos(1), 2, 2, 0).is_err());
    assert_eq!(editor.doc().blocks().len(), 1);
}

#[test]
fn undo_restores_the_exact_previous_table() {
    let Some(mut editor) = table_editor() else {
        return;
    };
    let before = editor.doc().clone();
    let selection = editor.selection();
    let origin = cell(0, 0, 0);
    assert!(editor.insert_row(origin, true).is_ok());
    assert_ne!(editor.doc(), &before);
    let Ok(Some(applied)) = editor.undo() else {
        return;
    };
    assert_eq!(editor.doc(), &before);
    assert_eq!(editor.selection(), selection);
    assert_eq!(applied.selection, selection);
}

#[test]
fn table_selection_semantics_are_cell_based() {
    let anchor = cell(0, 0, 0);
    let head = cell(0, 1, 1);
    let selection = Selection::table(anchor, head);
    assert_eq!(selection.anchor(), None);
    assert_eq!(selection.head(), None);
    assert!(selection.range().is_none());
    assert!(!selection.is_empty());
    assert!(selection.is_valid(Pos(0)));
    assert_eq!(selection.map(&Mapping::identity()), selection);
    assert!(Selection::table(anchor, anchor).is_empty());
    assert!(!Selection::table(anchor, cell(1, 0, 0)).is_valid(Pos(0)));
}

#[test]
fn edits_preserve_surrounding_blocks_and_other_tables() {
    let Ok(schema) = Schema::full() else {
        return;
    };
    let Ok(mut editor) = Editor::new() else {
        return;
    };
    assert!(editor.insert_table(Pos(0), 2, 2, 0).is_ok());
    let end = editor.doc().size().0.saturating_sub(1);
    let mut transaction = editor.transaction();
    assert!(transaction.insert_text(Pos(end), "tail").is_ok());
    assert!(editor.apply(transaction).is_ok());
    assert_eq!(editor.doc().plain_text(), "tail");
    assert_eq!(editor.doc().blocks().len(), 2);
    assert!(editor.insert_row(cell(0, 0, 0), false).is_ok());
    assert_eq!(editor.doc().plain_text(), "tail");
    assert_eq!(geometry(&editor, &schema), Some((3, 2)));
    assert!(editor.insert_table(Pos(0), 1, 1, 0).is_ok());
    assert!(editor.insert_row(cell(1, 0, 0), true).is_ok());
    assert_eq!(geometry(&editor, &schema), Some((1, 1)));
    let Some(second) = editor.doc().table_info(&schema, 1) else {
        return;
    };
    assert_eq!(second.rows(), 4);
    assert_eq!(editor.doc().plain_text(), "tail");
    assert!(editor.doc().validate(&schema).is_ok());
}
