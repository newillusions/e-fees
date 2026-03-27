import type { ScopeSection } from '$lib/types/scope';

/**
 * Format scope sections as plain text for clipboard/display.
 *
 * Output format matches auto_number_clauses() raw text:
 *   1.0 ADMINISTRATIVE
 *
 *   1.1 Title — Body text
 *   1.2 Title — Body text
 *
 *   2.0 COMMERCIAL
 *
 *   2.1 Title — Body text
 */
export function formatSectionsAsText(sections: ScopeSection[]): string {
  return sections
    .map((section) => {
      const header = `${section.number} ${section.title.toUpperCase()}`;
      if (section.clauses.length === 0) return header;

      const clauseLines = section.clauses
        .map((c) => `${c.number} ${c.title} — ${c.body}`)
        .join('\n');

      return `${header}\n\n${clauseLines}`;
    })
    .join('\n\n');
}
