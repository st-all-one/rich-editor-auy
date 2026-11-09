/** One public schema preset. */
export interface SchemaPreset {
  readonly name: string;
  readonly nodes: readonly string[];
  readonly marks: readonly string[];
}

/**
 * The single public schema.
 *
 * Every node and mark the ingestion boundary accepts is declared here; there is no basic or
 * custom preset in the public API.
 */
export function fullSchema(): SchemaPreset {
  return {
    name: "full",
    nodes: [
      "text",
      "paragraph",
      "heading",
      "bullet_list",
      "ordered_list",
      "list_item",
      "blockquote",
      "code_block",
      "raw_code",
      "table",
      "table_caption",
      "table_row",
      "table_header",
      "table_cell",
      "figure",
      "caption",
      "separator",
      "image",
      "image_block",
      "video",
      "audio",
      "attachment",
      "hard_break",
    ],
    marks: [
      "strong",
      "em",
      "code",
      "underline",
      "strike",
      "highlight",
      "sup",
      "sub",
      "link",
      "ref",
    ],
  };
}
