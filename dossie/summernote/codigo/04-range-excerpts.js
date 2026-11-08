/* ============================================================================
   ORIGEM: src/js/core/range.js — EXCERTOS (linhas 119-300, 380-600, 718-737 do original)
   PROJETO: summernote/summernote (MIT) — ver ref/summernote/LICENSE
   Cópia para análise no dossiê `rich-editor-auy`. Nada deste código será
   reutilizado no produto; é material de estudo. Direitos: Summernote Team.
   WrappedRange: seleção normalizada + inserção de nós/HTML (a álgebra de ranges do editor).
   ============================================================================ */

// ---------------------------------------------------------------------------
// [1/3] src/js/core/range.js:119-300 — WrappedRange: construção, nativeRange(), select(), normalize()
// ---------------------------------------------------------------------------
class WrappedRange {
  constructor(sc, so, ec, eo) {
    this.sc = sc;
    this.so = so;
    this.ec = ec;
    this.eo = eo;

    // isOnEditable: judge whether range is on editable or not
    this.isOnEditable = this.makeIsOn(dom.isEditable);
    // isOnList: judge whether range is on list node or not
    this.isOnList = this.makeIsOn(dom.isList);
    // isOnAnchor: judge whether range is on anchor node or not
    this.isOnAnchor = this.makeIsOn(dom.isAnchor);
    // isOnCell: judge whether range is on cell node or not
    this.isOnCell = this.makeIsOn(dom.isCell);
    // isOnData: judge whether range is on data node or not
    this.isOnData = this.makeIsOn(dom.isData);
  }

  // nativeRange: get nativeRange from sc, so, ec, eo
  nativeRange() {
    if (env.isW3CRangeSupport) {
      const w3cRange = document.createRange();
      w3cRange.setStart(this.sc, this.so);
      w3cRange.setEnd(this.ec, this.ec.data ? Math.min(this.eo, this.ec.data.length) : this.eo);

      return w3cRange;
    } else {
      const textRange = pointToTextRange({
        node: this.sc,
        offset: this.so,
      });

      textRange.setEndPoint('EndToEnd', pointToTextRange({
        node: this.ec,
        offset: this.eo,
      }));

      return textRange;
    }
  }

  getPoints() {
    return {
      sc: this.sc,
      so: this.so,
      ec: this.ec,
      eo: this.eo,
    };
  }

  getStartPoint() {
    return {
      node: this.sc,
      offset: this.so,
    };
  }

  getEndPoint() {
    return {
      node: this.ec,
      offset: this.eo,
    };
  }

  /**
   * select update visible range
   */
  select() {
    const nativeRng = this.nativeRange();
    if (env.isW3CRangeSupport) {
      const selection = document.getSelection();
      if (selection.rangeCount > 0) {
        selection.removeAllRanges();
      }
      selection.addRange(nativeRng);
    } else {
      nativeRng.select();
    }

    return this;
  }

  /**
   * Moves the scrollbar to start container(sc) of current range
   *
   * @return {WrappedRange}
   */
  scrollIntoView(container) {
    const height = $(container).height();
    if (container.scrollTop + height < this.sc.offsetTop) {
      container.scrollTop += Math.abs(container.scrollTop + height - this.sc.offsetTop);
    }

    return this;
  }

  /**
   * @return {WrappedRange}
   */
  normalize() {
    /**
     * @param {BoundaryPoint} point
     * @param {Boolean} isLeftToRight - true: prefer to choose right node
     *                                - false: prefer to choose left node
     * @return {BoundaryPoint}
     */
    const getVisiblePoint = function(point, isLeftToRight) {
      if (!point) {
        return point;
      }

      // Just use the given point [XXX:Adhoc]
      //  - case 01. if the point is on the middle of the node
      //  - case 02. if the point is on the right edge and prefer to choose left node
      //  - case 03. if the point is on the left edge and prefer to choose right node
      //  - case 04. if the point is on the right edge and prefer to choose right node but the node is void
      //  - case 05. if the point is on the left edge and prefer to choose left node but the node is void
      //  - case 06. if the point is on the block node and there is no children
      if (dom.isVisiblePoint(point)) {
        if (!dom.isEdgePoint(point) ||
            (dom.isRightEdgePoint(point) && !isLeftToRight) ||
            (dom.isLeftEdgePoint(point) && isLeftToRight) ||
            (dom.isRightEdgePoint(point) && isLeftToRight && dom.isVoid(point.node.nextSibling)) ||
            (dom.isLeftEdgePoint(point) && !isLeftToRight && dom.isVoid(point.node.previousSibling)) ||
            (dom.isBlock(point.node) && dom.isEmpty(point.node))) {
          return point;
        }
      }

      // point on block's edge
      const block = dom.ancestor(point.node, dom.isBlock);
      let hasRightNode = false;

      if (!hasRightNode) {
        const prevPoint = dom.prevPoint(point) || { node: null };
        hasRightNode = (dom.isLeftEdgePointOf(point, block) || dom.isVoid(prevPoint.node)) && !isLeftToRight;
      }

      let hasLeftNode = false;
      if (!hasLeftNode) {
        const nextPoint = dom.nextPoint(point) || { node: null };
        hasLeftNode = (dom.isRightEdgePointOf(point, block) || dom.isVoid(nextPoint.node)) && isLeftToRight;
      }

      if (hasRightNode || hasLeftNode) {
        // returns point already on visible point
        if (dom.isVisiblePoint(point)) {
          return point;
        }
        // reverse direction
        isLeftToRight = !isLeftToRight;
      }

      const nextPoint = isLeftToRight ? dom.nextPointUntil(dom.nextPoint(point), dom.isVisiblePoint)
        : dom.prevPointUntil(dom.prevPoint(point), dom.isVisiblePoint);
      return nextPoint || point;
    };

    const endPoint = getVisiblePoint(this.getEndPoint(), false);
    const startPoint = this.isCollapsed() ? endPoint : getVisiblePoint(this.getStartPoint(), true);

    return new WrappedRange(
      startPoint.node,
      startPoint.offset,
      endPoint.node,
      endPoint.offset
    );
  }

  /**
   * returns matched nodes on range
   *
   * @param {Function} [pred] - predicate function
   * @param {Object} [options]
   * @param {Boolean} [options.includeAncestor]
   * @param {Boolean} [options.fullyContains]
   * @return {Node[]}
   */
  nodes(pred, options) {
    pred = pred || func.ok;


// [... cortadas as linhas 301-379 do original: nodes/commonAncestor/expand ...]

// ---------------------------------------------------------------------------
// [2/3] src/js/core/range.js:380-600 — splitText, deleteContents, wrapBodyInlineWithPara, insertNode, pasteHTML
// ---------------------------------------------------------------------------
  /**
   * @param {Boolean} isCollapseToStart
   * @return {WrappedRange}
   */
  collapse(isCollapseToStart) {
    if (isCollapseToStart) {
      return new WrappedRange(this.sc, this.so, this.sc, this.so);
    } else {
      return new WrappedRange(this.ec, this.eo, this.ec, this.eo);
    }
  }

  /**
   * splitText on range
   */
  splitText() {
    const isSameContainer = this.sc === this.ec;
    const boundaryPoints = this.getPoints();

    if (dom.isText(this.ec) && !dom.isEdgePoint(this.getEndPoint())) {
      this.ec.splitText(this.eo);
    }

    if (dom.isText(this.sc) && !dom.isEdgePoint(this.getStartPoint())) {
      boundaryPoints.sc = this.sc.splitText(this.so);
      boundaryPoints.so = 0;

      if (isSameContainer) {
        boundaryPoints.ec = boundaryPoints.sc;
        boundaryPoints.eo = this.eo - this.so;
      }
    }

    return new WrappedRange(
      boundaryPoints.sc,
      boundaryPoints.so,
      boundaryPoints.ec,
      boundaryPoints.eo
    );
  }

  /**
   * delete contents on range
   * @return {WrappedRange}
   */
  deleteContents() {
    if (this.isCollapsed()) {
      return this;
    }

    const rng = this.splitText();
    const nodes = rng.nodes(null, {
      fullyContains: true,
    });

    // find new cursor point
    const point = dom.prevPointUntil(rng.getStartPoint(), function(point) {
      return !lists.contains(nodes, point.node);
    });

    const emptyParents = [];
    $.each(nodes, function(idx, node) {
      // find empty parents
      const parent = node.parentNode;
      if (point.node !== parent && dom.nodeLength(parent) === 1) {
        emptyParents.push(parent);
      }
      dom.remove(node, false);
    });

    // remove empty parents
    $.each(emptyParents, function(idx, node) {
      dom.remove(node, false);
    });

    return new WrappedRange(
      point.node,
      point.offset,
      point.node,
      point.offset
    ).normalize();
  }

  /**
   * makeIsOn: return isOn(pred) function
   */
  makeIsOn(pred) {
    return function() {
      const ancestor = dom.ancestor(this.sc, pred);
      return !!ancestor && (ancestor === dom.ancestor(this.ec, pred));
    };
  }

  /**
   * @param {Function} pred
   * @return {Boolean}
   */
  isLeftEdgeOf(pred) {
    if (!dom.isLeftEdgePoint(this.getStartPoint())) {
      return false;
    }

    const node = dom.ancestor(this.sc, pred);
    return node && dom.isLeftEdgeOf(this.sc, node);
  }

  /**
   * returns whether range was collapsed or not
   */
  isCollapsed() {
    return this.sc === this.ec && this.so === this.eo;
  }

  /**
   * wrap inline nodes which children of body with paragraph
   *
   * @return {WrappedRange}
   */
  wrapBodyInlineWithPara() {
    if (dom.isBodyContainer(this.sc) && dom.isEmpty(this.sc)) {
      this.sc.innerHTML = dom.emptyPara;
      return new WrappedRange(this.sc.firstChild, 0, this.sc.firstChild, 0);
    }

    /**
     * [workaround] firefox often create range on not visible point. so normalize here.
     *  - firefox: |<p>text</p>|
     *  - chrome: <p>|text|</p>
     */
    const rng = this.normalize();
    if (dom.isParaInline(this.sc) || dom.isPara(this.sc)) {
      return rng;
    }

    // find inline top ancestor
    let topAncestor;
    if (dom.isInline(rng.sc)) {
      const ancestors = dom.listAncestor(rng.sc, func.not(dom.isInline));
      topAncestor = lists.last(ancestors);
      if (!dom.isInline(topAncestor)) {
        topAncestor = ancestors[ancestors.length - 2] || rng.sc.childNodes[rng.so];
      }
    } else {
      topAncestor = rng.sc.childNodes[rng.so > 0 ? rng.so - 1 : 0];
    }

    if (topAncestor) {
      // siblings not in paragraph
      let inlineSiblings = dom.listPrev(topAncestor, dom.isParaInline).reverse();
      inlineSiblings = inlineSiblings.concat(dom.listNext(topAncestor.nextSibling, dom.isParaInline));

      // wrap with paragraph
      if (inlineSiblings.length) {
        const para = dom.wrap(lists.head(inlineSiblings), 'p');
        dom.appendChildNodes(para, lists.tail(inlineSiblings));
      }
    }

    return this.normalize();
  }

  /**
   * insert node at current cursor
   *
   * @param {Node} node
   * @param {Boolean} doNotInsertPara - default is false, removes added <p> that's added if true
   * @return {Node}
   */
  insertNode(node, doNotInsertPara = false) {
    let rng = this;

    if (dom.isText(node) || dom.isInline(node)) {
      rng = this.wrapBodyInlineWithPara().deleteContents();
    }

    const info = dom.splitPoint(rng.getStartPoint(), dom.isInline(node));
    if (info.rightNode) {
      info.rightNode.parentNode.insertBefore(node, info.rightNode);
      if (dom.isEmpty(info.rightNode) && (doNotInsertPara || dom.isPara(node))) {
        info.rightNode.parentNode.removeChild(info.rightNode);
      }
    } else {
      info.container.appendChild(node);
    }

    return node;
  }

  /**
   * insert html at current cursor
   */
  pasteHTML(markup) {
    markup = ((markup || '') + '').trim();

    const contentsContainer = $('<div></div>').html(markup)[0];
    let childNodes = lists.from(contentsContainer.childNodes);

    // const rng = this.wrapBodyInlineWithPara().deleteContents();
    const rng = this;
    let reversed = false;

    if (rng.so >= 0) {
      childNodes = childNodes.reverse();
      reversed = true;
    }

    childNodes = childNodes.map(function(childNode) {
      return rng.insertNode(childNode, !dom.isInline(childNode));
    });

    if (reversed) {
      childNodes = childNodes.reverse();
    }
    return childNodes;
  }

  /**
   * returns text in range
   *
   * @return {String}
   */

// [... cortadas as linhas 601-717 do original: toString, getWordRange(s), getWordsMatchRange ...]

// ---------------------------------------------------------------------------
// [3/3] src/js/core/range.js:718-737 — bookmark(editable): como o undo salva a seleção
// ---------------------------------------------------------------------------
  bookmark(editable) {
    return {
      s: {
        path: dom.makeOffsetPath(editable, this.sc),
        offset: this.so,
      },
      e: {
        path: dom.makeOffsetPath(editable, this.ec),
        offset: this.eo,
      },
    };
  }

  /**
   * create offsetPath bookmark base on paragraph
   *
   * @param {Node[]} paras
   */
  paraBookmark(paras) {
    return {