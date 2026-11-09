import type { A11yNode, StateProvider } from "@rich-editor-auy/core";
import { renderTree } from "./render.ts";

export interface MountOptions {
  readonly state: StateProvider;
  readonly editable?: boolean;
  readonly label?: string;
  readonly onInput?: (event: InputEvent, state: StateProvider) => void;
}

export interface MountedEditor {
  destroy(): void;
  view(): HTMLElement;
  dispatch(state: StateProvider): void;
  state(): StateProvider;
}

interface SavedHost {
  readonly html: string;
  readonly editable: string | null;
  readonly role: string | null;
  readonly multiline: string | null;
  readonly lang: string | null;
  readonly dir: string | null;
}

/** Mounts a semantic editor shell over a consumer-owned host element. */
export function mount(host: HTMLElement, options: MountOptions): MountedEditor {
  const saved: SavedHost = {
    html: host.innerHTML,
    editable: host.getAttribute("contenteditable"),
    role: host.getAttribute("role"),
    multiline: host.getAttribute("aria-multiline"),
    lang: host.getAttribute("lang"),
    dir: host.getAttribute("dir"),
  };
  host.setAttribute("role", "textbox");
  host.setAttribute("aria-multiline", "true");
  host.setAttribute(
    "contenteditable",
    options.editable === false ? "false" : "true",
  );
  host.dataset.auyRoot = "true";
  let state = options.state;

  const render = (): void => {
    const tree: A11yNode = state.a11yTree();
    host.setAttribute("aria-label", options.label ?? "Document");
    if (tree.lang !== undefined) host.setAttribute("lang", tree.lang);
    if (tree.dir !== undefined) host.setAttribute("dir", tree.dir);
    renderTree(host, tree);
  };

  const onInput = (event: Event): void => {
    if (options.onInput !== undefined) {
      options.onInput(event as InputEvent, state);
    }
  };
  host.addEventListener("input", onInput);
  render();

  return {
    destroy(): void {
      host.removeEventListener("input", onInput);
      host.innerHTML = saved.html;
      restore(host, "contenteditable", saved.editable);
      restore(host, "role", saved.role);
      restore(host, "aria-multiline", saved.multiline);
      restore(host, "lang", saved.lang);
      restore(host, "dir", saved.dir);
      host.removeAttribute("aria-label");
      delete host.dataset.auyRoot;
    },
    view(): HTMLElement {
      return host;
    },
    dispatch(next: StateProvider): void {
      state = next;
      render();
    },
    state(): StateProvider {
      return state;
    },
  };
}

/** Alias for hosts that prefer explicit binding terminology. */
export function bind(host: HTMLElement, options: MountOptions): MountedEditor {
  return mount(host, options);
}

function restore(host: HTMLElement, name: string, value: string | null): void {
  if (value === null) host.removeAttribute(name);
  else host.setAttribute(name, value);
}
