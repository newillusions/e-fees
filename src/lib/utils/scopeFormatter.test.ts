import { describe, it, expect } from 'vitest';
import { formatSectionsAsText } from './scopeFormatter';
import type { ScopeSection } from '$lib/types/scope';

describe('formatSectionsAsText', () => {
  it('formats multiple sections with clauses', () => {
    const sections: ScopeSection[] = [
      {
        number: '1.0',
        title: 'Administrative',
        clauses: [
          { number: '1.1', clause_id: 'c1', title: 'General', body: 'General terms apply' },
          { number: '1.2', clause_id: 'c2', title: 'Meetings', body: 'Regular meetings required' },
        ],
      },
      {
        number: '2.0',
        title: 'Commercial',
        clauses: [
          { number: '2.1', clause_id: 'c3', title: 'Payment', body: 'Net 30 days' },
        ],
      },
    ];

    const result = formatSectionsAsText(sections);
    expect(result).toBe(
      '1.0 ADMINISTRATIVE\n\n' +
      '1.1 General — General terms apply\n' +
      '1.2 Meetings — Regular meetings required\n\n' +
      '2.0 COMMERCIAL\n\n' +
      '2.1 Payment — Net 30 days'
    );
  });

  it('returns empty string for empty array', () => {
    expect(formatSectionsAsText([])).toBe('');
  });

  it('handles section with no clauses', () => {
    const sections: ScopeSection[] = [
      { number: '1.0', title: 'Empty', clauses: [] },
    ];
    expect(formatSectionsAsText(sections)).toBe('1.0 EMPTY');
  });

  it('handles single clause', () => {
    const sections: ScopeSection[] = [
      {
        number: '1.0',
        title: 'Legal',
        clauses: [
          { number: '1.1', clause_id: 'c1', title: 'Liability', body: 'Limited to fee value' },
        ],
      },
    ];
    expect(formatSectionsAsText(sections)).toBe(
      '1.0 LEGAL\n\n' +
      '1.1 Liability — Limited to fee value'
    );
  });
});
