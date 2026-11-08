export interface SchemaPreset {
  readonly name: string;
  readonly nodes: readonly string[];
}

export function basicSchema(): SchemaPreset {
  return {
    name: "basic",
    nodes: [
      "paragraph",
      "heading",
      "bullet_list",
      "list_item",
      "image",
      "text",
    ],
  };
}

export function documentSchema(): SchemaPreset {
  return basicSchema();
}
