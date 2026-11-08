/* ============================================================================
   ORIGEM: src/js/module/Editor.js — EXCERTOS (linhas 339-500, 620-700, 735-800, 1024-1054)
   PROJETO: summernote/summernote (MIT) — ver ref/summernote/LICENSE
   Cópia para análise no dossiê `rich-editor-auy`. Nada deste código será
   reutilizado no produto; é material de estudo. Direitos: Summernote Team.
   O módulo central: eventos, keymap/undo, before/afterCommand, insertImage, foco/vazio.
   ============================================================================ */

// ---------------------------------------------------------------------------
// [1/4] src/js/module/Editor.js:339-500 — initialize(): listeners, snapshot, keymap, input/change
// ---------------------------------------------------------------------------
  initialize() {
    // bind custom events
    this.$editable.on('keydown', (event) => {
      if (event.keyCode === key.code.ENTER) {
        this.context.triggerEvent('enter', event);
      }
      this.context.triggerEvent('keydown', event);

      // keep a snapshot to limit text on input event
      this.snapshot = this.history.makeSnapshot();
      this.hasKeyShortCut = false;
      if (!event.isDefaultPrevented()) {
        if (this.options.shortcuts) {
          this.hasKeyShortCut = this.handleKeyMap(event);
        } else {
          this.preventDefaultEditableShortCuts(event);
        }
      }
      if (this.isLimited(1, event)) {
        const lastRange = this.getLastRange();
        if (lastRange.eo - lastRange.so === 0) {
          return false;
        }
      }
      this.setLastRange();

      // record undo in the key event except keyMap.
      if (this.options.recordEveryKeystroke) {
        if (this.hasKeyShortCut === false) {
          this.history.recordUndo();
        }
      }
    }).on('keyup', (event) => {
      this.setLastRange();
      this.context.triggerEvent('keyup', event);
    }).on('focus', (event) => {
      this.setLastRange();
      this.context.triggerEvent('focus', event);
    }).on('blur', (event) => {
      this.context.triggerEvent('blur', event);
    }).on('mousedown', (event) => {
      this.context.triggerEvent('mousedown', event);
    }).on('mouseup', (event) => {
      this.setLastRange();
      this.history.recordUndo();
      this.context.triggerEvent('mouseup', event);
    }).on('scroll', (event) => {
      this.context.triggerEvent('scroll', event);
    }).on('paste', (event) => {
      this.setLastRange();
      this.context.triggerEvent('paste', event);
    }).on('copy', (event) => {
      this.context.triggerEvent('copy', event);
    }).on('input', () => {
      // To limit composition characters (e.g. Korean)
      if (this.isLimited(0) && this.snapshot) {
        this.history.applySnapshot(this.snapshot);
      }
    });

    this.$editable.attr('spellcheck', this.options.spellCheck);

    this.$editable.attr('autocorrect', this.options.spellCheck);

    if (this.options.disableGrammar) {
      this.$editable.attr('data-gramm', false);
    }

    // init content before set event
    this.$editable.html(dom.html(this.$note) || dom.emptyPara);

    this.$editable.on(env.inputEventName, func.debounce(() => {
      this.context.triggerEvent('change', this.$editable.html(), this.$editable);
    }, 10));

    this.$editable.on('focusin', (event) => {
      this.context.triggerEvent('focusin', event);
    }).on('focusout', (event) => {
      this.context.triggerEvent('focusout', event);
    });

    if (this.options.airMode) {
      if (this.options.overrideContextMenu) {
        this.$editor.on('contextmenu', (event) => {
          this.context.triggerEvent('contextmenu', event);
          return false;
        });
      }
    } else {
      if (this.options.width) {
        this.$editor.outerWidth(this.options.width);
      }
      if (this.options.height) {
        this.$editable.outerHeight(this.options.height);
      }
      if (this.options.maxHeight) {
        this.$editable.css('max-height', this.options.maxHeight);
      }
      if (this.options.minHeight) {
        this.$editable.css('min-height', this.options.minHeight);
      }
    }

    this.history.recordUndo();
    this.setLastRange();
  }

  destroy() {
    this.$editable.off();
  }

  handleKeyMap(event) {
    const keyMap = this.options.keyMap[env.isMac ? 'mac' : 'pc'];
    const keys = [];

    if (event.metaKey) { keys.push('CMD'); }
    if (event.ctrlKey && !event.altKey) { keys.push('CTRL'); }
    if (event.shiftKey) { keys.push('SHIFT'); }

    const keyName = key.nameFromCode[event.keyCode];
    if (keyName) {
      keys.push(keyName);
    }

    const eventName = keyMap[keys.join('+')];

    if (keyName === 'TAB' && !this.options.tabDisable) {
      this.afterCommand();
    } else if (eventName) {
      if (this.context.invoke(eventName) !== false) {
        event.preventDefault();
        return true;
      }
    } else if (key.isEdit(event.keyCode)) {
      if (key.isRemove(event.keyCode)) {
        this.context.invoke('removed');
      }
      this.afterCommand();
    }
    return false;
  }

  preventDefaultEditableShortCuts(event) {
    // B(Bold, 66) / I(Italic, 73) / U(Underline, 85)
    if ((event.ctrlKey || event.metaKey) &&
      lists.contains([66, 73, 85], event.keyCode)) {
      event.preventDefault();
    }
  }

  isLimited(pad, event) {
    pad = pad || 0;

    if (typeof event !== 'undefined') {
      if (key.isMove(event.keyCode) ||
          key.isNavigation(event.keyCode) ||
          (event.ctrlKey || event.metaKey) ||
          lists.contains([key.code.BACKSPACE, key.code.DELETE], event.keyCode)) {
        return false;
      }
    }


// [... cortadas as linhas 501-619: isLimited/checkLinkUrl/createRange(s)/setLastRange/saveRange ...]

// ---------------------------------------------------------------------------
// [2/4] src/js/module/Editor.js:620-700 — currentStyle, undo/redo/commit, beforeCommand/afterCommand
// ---------------------------------------------------------------------------
  }

  /**
   * currentStyle
   *
   * current style
   * @return {Object|Boolean} unfocus
   */
  currentStyle() {
    let rng = range.create();
    if (rng) {
      rng = rng.normalize();
    }
    return rng ? this.style.current(rng) : this.style.fromNode(this.$editable);
  }

  /**
   * style from node
   *
   * @param {jQuery} $node
   * @return {Object}
   */
  styleFromNode($node) {
    return this.style.fromNode($node);
  }

  /**
   * undo
   */
  undo() {
    this.context.triggerEvent('before.command', this.$editable.html());
    this.history.undo();
    this.context.triggerEvent('change', this.$editable.html(), this.$editable);
  }

  /*
  * commit
  */
  commit() {
    this.context.triggerEvent('before.command', this.$editable.html());
    this.history.commit();
    this.context.triggerEvent('change', this.$editable.html(), this.$editable);
  }

  /**
   * redo
   */
  redo() {
    this.context.triggerEvent('before.command', this.$editable.html());
    this.history.redo();
    this.context.triggerEvent('change', this.$editable.html(), this.$editable);
  }

  /**
   * before command
   */
  beforeCommand() {
    this.context.triggerEvent('before.command', this.$editable.html());

    // Set styleWithCSS before run a command
    document.execCommand('styleWithCSS', false, this.options.styleWithCSS);

    // keep focus on editable before command execution
    this.focus();
  }

  /**
   * after command
   * @param {Boolean} isPreventTrigger
   */
  afterCommand(isPreventTrigger) {
    this.normalizeContent();
    this.history.recordUndo();
    if (!isPreventTrigger) {
      this.context.triggerEvent('change', this.$editable.html(), this.$editable);
    }
  }

  /**
   * handle tab key
   */

// [... cortadas as linhas 701-734: tab/untab ...]

// ---------------------------------------------------------------------------
// [3/4] src/js/module/Editor.js:735-800 — wrapCommand, removed(), insertImage
// ---------------------------------------------------------------------------
  wrapCommand(fn) {
    return function() {
      this.beforeCommand();
      fn.apply(this, arguments);
      this.afterCommand();
    };
  }
  /**
   * removed (function added by 1der1)
  */
  removed(rng, node, tagName) { // LB
    rng = range.create();
    if (rng.isCollapsed() && rng.isOnCell()) {
      node = rng.ec;
      if( (tagName = node.tagName) &&
				(node.childElementCount === 1) &&
				(node.childNodes[0].tagName === "BR") ){

        if(tagName === "P") {
          node.remove();
        } else if(['TH', 'TD'].indexOf(tagName) >=0) {
          node.firstChild.remove();
        }
      }
    }
  }
  /**
   * insert image
   *
   * @param {String} src
   * @param {String|Function} param
   * @return {Promise}
   */
  insertImage(src, param) {
    return createImage(src, param).then(($image) => {
      this.beforeCommand();

      if (typeof param === 'function') {
        param($image);
      } else {
        if (typeof param === 'string') {
          $image.attr('data-filename', param);
        }
        $image.css('width', Math.min(this.$editable.width(), $image.width()));
      }

      $image.show();
      this.getLastRange().insertNode($image[0]);
      this.setLastRange(range.createFromNodeAfter($image[0]).select());
      this.afterCommand();
    }).fail((e) => {
      this.context.triggerEvent('image.upload.error', e);
    });
  }

  /**
   * insertImages
   * @param {File[]} files
   */
  insertImagesAsDataURL(files) {
    $.each(files, (idx, file) => {
      const filename = file.name;
      if (this.options.maximumImageFileSize && this.options.maximumImageFileSize < file.size) {
        this.context.triggerEvent('image.upload.error', this.lang.image.maximumFileSizeError);
      } else {
        readFileAsDataURL(file).then((dataURL) => {

// [... cortadas as linhas 801-1023: insertImages, formatBlock, fontStyling, unlink, tabelas, resize ...]

// ---------------------------------------------------------------------------
// [4/4] src/js/module/Editor.js:1024-1054 — hasFocus/focus/isEmpty/empty/normalizeContent
// ---------------------------------------------------------------------------
   */
  focus() {
    // [workaround] Screen will move when page is scolled in IE.
    //  - do focus when not focused
    if (!this.hasFocus()) {
      this.$editable.trigger('focus');
    }
  }

  /**
   * returns whether contents is empty or not.
   * @return {Boolean}
   */
  isEmpty() {
    return dom.isEmpty(this.$editable[0]) || dom.emptyPara === this.$editable.html();
  }

  /**
   * Removes all contents and restores the editable instance to an _emptyPara_.
   */
  empty() {
    this.context.invoke('code', dom.emptyPara);
  }

  /**
   * normalize content
   */
  normalizeContent() {
    this.$editable[0].normalize();
  }
}