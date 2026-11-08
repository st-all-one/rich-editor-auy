/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * ---------------------------------------------------------------------------
 * TRECHO COPIA DO (MIT, ver ref/lexical/LICENSE):
 *   ref/lexical/packages/lexical/src/LexicalUpdates.ts
 *
 * Intervalos:
 *   251-275  $applyTransforms        -> despacho de transforms por tipo de no
 *   288-306  $normalizeAllDirtyTextNodes
 *   333-428  $applyAllTransforms    -> heuristica de fixpoint (leaves antes de elements)
 *
 * Alteracoes: apenas concatenacao de intervalos e este cabecalho.
 * ---------------------------------------------------------------------------
 */

// ===== ref/lexical/packages/lexical/src/LexicalUpdates.ts:251-275 =====
export function $applyTransforms(
  editor: LexicalEditor,
  node: LexicalNode,
  transformsCache: Map<string, Transform<LexicalNode>[]>,
) {
  const type = node.__type;
  const registeredNode = getRegisteredNodeOrThrow(editor, type);
  let transformsArr = transformsCache.get(type);

  if (transformsArr === undefined) {
    transformsArr = Array.from(registeredNode.transforms);
    transformsCache.set(type, transformsArr);
  }

  const transformsArrLength = transformsArr.length;

  for (let i = 0; i < transformsArrLength; i++) {
    transformsArr[i](node);

    if (!node.isAttached()) {
      break;
    }
  }
}


// ===== ref/lexical/packages/lexical/src/LexicalUpdates.ts:288-306 =====
function $normalizeAllDirtyTextNodes(
  editorState: EditorState,
  editor: LexicalEditor,
): void {
  const dirtyLeaves = editor._dirtyLeaves;
  const nodeMap = editorState._nodeMap;

  for (const nodeKey of dirtyLeaves) {
    const node = nodeMap.get(nodeKey);

    if (
      $isTextNode(node) &&
      node.isAttached() &&
      node.isSimpleText() &&
      !node.isUnmergeable()
    ) {
      $normalizeTextNode(node);
    }
  }

// ===== ref/lexical/packages/lexical/src/LexicalUpdates.ts:333-428 =====
function $applyAllTransforms(
  editorState: EditorState,
  editor: LexicalEditor,
): void {
  const dirtyLeaves = editor._dirtyLeaves;
  const dirtyElements = editor._dirtyElements;
  const nodeMap = editorState._nodeMap;
  const compositionKey = $getCompositionKey();
  const transformsCache = new Map();

  let untransformedDirtyLeaves = dirtyLeaves;
  let untransformedDirtyLeavesLength = untransformedDirtyLeaves.size;
  let untransformedDirtyElements = dirtyElements;
  let untransformedDirtyElementsLength = untransformedDirtyElements.size;

  while (
    untransformedDirtyLeavesLength > 0 ||
    untransformedDirtyElementsLength > 0
  ) {
    if (untransformedDirtyLeavesLength > 0) {
      // We leverage editor._dirtyLeaves to track the new dirty leaves after the transforms
      editor._dirtyLeaves = new Set();

      for (const nodeKey of untransformedDirtyLeaves) {
        const node = nodeMap.get(nodeKey);

        if (
          $isTextNode(node) &&
          node.isAttached() &&
          node.isSimpleText() &&
          !node.isUnmergeable()
        ) {
          $normalizeTextNode(node);
        }

        if (
          node !== undefined &&
          $isNodeValidForTransform(node, compositionKey)
        ) {
          $applyTransforms(editor, node, transformsCache);
        }

        dirtyLeaves.add(nodeKey);
      }

      untransformedDirtyLeaves = editor._dirtyLeaves;
      untransformedDirtyLeavesLength = untransformedDirtyLeaves.size;

      // We want to prioritize node transforms over element transforms
      if (untransformedDirtyLeavesLength > 0) {
        infiniteTransformCount++;
        continue;
      }
    }

    // All dirty leaves have been processed. Let's do elements!
    // We have previously processed dirty leaves, so let's restart the editor leaves Set to track
    // new ones caused by element transforms
    editor._dirtyLeaves = new Set();
    editor._dirtyElements = new Map();

    // The root is always considered intentionally dirty if any attached node
    // is dirty and by deleting and re-inserting we will apply its transforms
    // last (e.g. its transform can be used as a sort of "update finalizer")
    const rootDirty = untransformedDirtyElements.delete('root');
    if (rootDirty) {
      untransformedDirtyElements.set('root', true);
    }
    for (const currentUntransformedDirtyElement of untransformedDirtyElements) {
      const nodeKey = currentUntransformedDirtyElement[0];
      const intentionallyMarkedAsDirty = currentUntransformedDirtyElement[1];
      dirtyElements.set(nodeKey, intentionallyMarkedAsDirty);
      if (!intentionallyMarkedAsDirty) {
        continue;
      }

      const node = nodeMap.get(nodeKey);

      if (
        node !== undefined &&
        $isNodeValidForTransform(node, compositionKey)
      ) {
        $applyTransforms(editor, node, transformsCache);
      }
    }

    untransformedDirtyLeaves = editor._dirtyLeaves;
    untransformedDirtyLeavesLength = untransformedDirtyLeaves.size;
    untransformedDirtyElements = editor._dirtyElements;
    untransformedDirtyElementsLength = untransformedDirtyElements.size;
    infiniteTransformCount++;
  }

  editor._dirtyLeaves = dirtyLeaves;
  editor._dirtyElements = dirtyElements;
}
