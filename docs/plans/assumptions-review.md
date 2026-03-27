# Assumptions Clause — Working Review

**Status:** Ongoing review
**Last updated:** 2026-03-09

## Context

Assumptions are typically written fresh per proposal and often left sparse or deleted entirely under time pressure. Goal: build a reusable set of standard assumptions that can be toggled per proposal.

## Corpus Findings (51 ingested proposals, 15 contained "Assumptions")

Only 9 substantive assumption bullets found across all historical proposals. Confirms assumptions are project-specific and inconsistently included.

## Candidate Assumptions (requires filtering)

Many of these overlap with existing clauses (Contract Details, Design Phase Notes, Fees/Payment Terms, Optional Services). Only items NOT already covered elsewhere should become standalone assumptions.

### Client & Team Responsibilities
1. The Client shall appoint a Lead Design Consultant (LDC) who will coordinate all design disciplines and issue consolidated information packages
2. All relevant architectural, structural, and MEP drawings/specifications will be provided to emittiv in a timely manner and at no cost
3. The Client or LDC will provide a clear and complete design brief prior to commencement of services
4. Access to site will be arranged by the Client/Main Contractor at no additional cost to emittiv

### Programme & Scope
5. The project programme allows adequate time for each design stage, including review and approval periods
6. Fee proposal is based on a single design iteration per stage — significant redesign or scope changes will be subject to additional fees
7. Services are limited to architectural/decorative lighting design unless otherwise stated — emergency, exit signage, aviation obstruction, and specialist systems lighting are excluded
8. ~~Landscape lighting is excluded unless explicitly included in the scope of services~~ — **NOTE: landscape lighting is often included in scope. Not a safe default assumption.**
9. ~~BIM/Revit modelling services are excluded unless explicitly stated~~ — **NOTE: Revit is often part of the scope when agreed during bidding. Not a safe default exclusion.**

### Procurement & Construction
10. Lighting equipment procurement, supply, and installation are by others
11. The Main Contractor shall provide adequate power provisions to all luminaire positions as indicated on lighting layout drawings
12. Mock-up rooms or lighting mock-ups, if required, will be subject to separate fees and/or reimbursable costs
13. All fees are based on the project scope and scale as described at the time of this proposal

### Commercial
14. Fees are exclusive of VAT (where applicable) and any local withholding taxes
15. Travel and accommodation expenses for site visits are reimbursable at cost unless included in the fee breakdown
16. Fees assume the project will proceed without unreasonable delay — suspension of services beyond [X] months may require fee reassessment

## Review Notes

- Items 8, 9 struck through — these are NOT safe defaults (landscape and Revit are often in scope)
- Several items may already be covered by existing clauses:
  - Fees/Payment Terms clause may cover items 14, 15
  - Contract Details clause may cover items 4, 13
  - Design Phase Notes may cover items 5, 6
  - Optional Services clause may cover item 12
- Need to cross-reference each candidate against existing clause library to identify genuine gaps
- Consider making assumptions toggleable per proposal in the scope assembly system

## Sources
- Corpus: 51 ingested PDFs in `proposal_corpus` table (SurrealDB 10.0.23.11)
- Web research: RIBA sub-consultant appointment terms, MEP/lighting consultant fee proposal standards
