/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * ---------------------------------------------------------------------------
 * TRECHO COPIA DO (MIT, ver ref/lexical/LICENSE):
 *   ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts
 *
 * Intervalos:
 *   168-192  getElementOuterTag / getElementInnerTag  -> modelo vira tag semantica
 *   194-265  setTextThemeClassNames                   -> tema como classes CSS
 *   902-924  createDOM                                -> no -> DOM
 *
 * Alteracoes: apenas concatenacao de intervalos e este cabecalho.
 * ---------------------------------------------------------------------------
 */

// ===== ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts:168-192 =====
function getElementOuterTag(node: TextNode, format: number): string | null {
  if (format & IS_CODE) {
    return 'code';
  }
  if (format & IS_HIGHLIGHT) {
    return 'mark';
  }
  if (format & IS_SUBSCRIPT) {
    return 'sub';
  }
  if (format & IS_SUPERSCRIPT) {
    return 'sup';
  }
  return null;
}

function getElementInnerTag(node: TextNode, format: number): string {
  if (format & IS_BOLD) {
    return 'strong';
  }
  if (format & IS_ITALIC) {
    return 'em';
  }
  return 'span';
}

// ===== ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts:194-265 =====
function setTextThemeClassNames(
  tag: string,
  prevFormat: number,
  nextFormat: number,
  dom: HTMLElement,
  textClassNames: TextNodeThemeClasses,
): void {
  const domClassList = dom.classList;
  // Firstly we handle the base theme.
  let classNames = getCachedClassNameArray(textClassNames, 'base');
  if (classNames !== undefined) {
    domClassList.add(...classNames);
  }
  // Secondly we handle the special case: underline + strikethrough.
  // We have to do this as we need a way to compose the fact that
  // the same CSS property will need to be used: text-decoration.
  // In an ideal world we shouldn't have to do this, but there's no
  // easy workaround for many atomic CSS systems today.
  classNames = getCachedClassNameArray(
    textClassNames,
    'underlineStrikethrough',
  );
  let hasUnderlineStrikethrough = false;
  const prevUnderlineStrikethrough =
    prevFormat & IS_UNDERLINE && prevFormat & IS_STRIKETHROUGH;
  const nextUnderlineStrikethrough =
    nextFormat & IS_UNDERLINE && nextFormat & IS_STRIKETHROUGH;

  if (classNames !== undefined) {
    if (nextUnderlineStrikethrough) {
      hasUnderlineStrikethrough = true;
      if (!prevUnderlineStrikethrough) {
        domClassList.add(...classNames);
      }
    } else if (prevUnderlineStrikethrough) {
      domClassList.remove(...classNames);
    }
  }

  for (const key in TEXT_TYPE_TO_FORMAT) {
    const format = key;
    const flag = TEXT_TYPE_TO_FORMAT[format];
    classNames = getCachedClassNameArray(textClassNames, key);
    if (classNames !== undefined) {
      if (nextFormat & flag) {
        if (
          hasUnderlineStrikethrough &&
          (key === 'underline' || key === 'strikethrough')
        ) {
          if (prevFormat & flag) {
            domClassList.remove(...classNames);
          }
          continue;
        }
        if (
          (prevFormat & flag) === 0 ||
          (prevUnderlineStrikethrough && key === 'underline') ||
          key === 'strikethrough'
        ) {
          domClassList.add(...classNames);
        }
      } else if (prevFormat & flag) {
        domClassList.remove(...classNames);
      }
    }
  }
  removeEmptyDOMAttribute(dom, 'class');
}

function diffComposedText(a: string, b: string): [number, number, string] {
  const aLength = a.length;
  const bLength = b.length;

// ===== ref/lexical/packages/lexical/src/nodes/LexicalTextNode.ts:902-924 =====
  createDOM(config: EditorConfig, editor?: LexicalEditor): HTMLElement {
    const format = this.__format;
    const outerTag = getElementOuterTag(this, format);
    const innerTag = getElementInnerTag(this, format);
    const tag = outerTag === null ? innerTag : outerTag;
    const dom = $getDocument().createElement(tag);
    let innerDOM = dom;
    if (this.hasFormat('code')) {
      dom.setAttribute('spellcheck', 'false');
    }
    if (outerTag !== null) {
      innerDOM = $getDocument().createElement(innerTag);
      dom.appendChild(innerDOM);
    }
    const text = this.__text;
    $createTextInnerDOM(innerDOM, this, innerTag, format, text, config);
    const style = this.__style;
    if (style !== '') {
      setDOMStyleFromCSS(dom.style, style);
    }
    return dom;
  }

