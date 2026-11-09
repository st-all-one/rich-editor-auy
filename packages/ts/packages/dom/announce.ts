/**
 * Live-region announcements.
 *
 * The host owns the live region: the adapter creates one polite region per mounted editor, and
 * announces composition state, remote changes and errors. It never announces every keystroke, and
 * it never speaks content the core did not project.
 */

import type { ChangeSetJSON } from "@rich-editor-auy/core";

/** A polite live region bound to one host. */
export interface Announcer {
  /** Announces one message; repeated messages still reach the reader. */
  announce(message: string): void;
  /** Restores the host. */
  destroy(): void;
}

/** Creates a polite live region next to the editor host. */
export function createAnnouncer(host: HTMLElement): Announcer {
  const doc = host.ownerDocument;
  const region = doc.createElement("div");
  region.setAttribute("role", "status");
  region.setAttribute("aria-live", "polite");
  region.setAttribute("aria-atomic", "true");
  region.dataset.auyLive = "true";
  region.hidden = false;
  host.after(region);
  let sequence = 0;
  return {
    announce(message: string): void {
      // Alternating a zero-width space keeps identical messages audible.
      sequence += 1;
      region.textContent = sequence % 2 === 0 ? `${message}\u200B` : message;
    },
    destroy(): void {
      region.remove();
    },
  };
}

/** Describes a remote change for a live region, or returns `undefined` when silent. */
export function describeChange(
  change: ChangeSetJSON | undefined,
): string | undefined {
  if (change === undefined) return undefined;
  if (change.full) return "Document replaced";
  if (change.ranges.length === 0) return undefined;
  const blocks = change.blocks.to - change.blocks.from;
  if (blocks > 1) return `${blocks} blocks changed`;
  return "Content changed";
}
