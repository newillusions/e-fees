# Assumptions Clause — Working Review

**Status:** Gap analysis done 2026-06-14 — shortlist below awaiting selection
**Last updated:** 2026-06-14

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

## Gap Analysis — 2026-06-14

Mapped the 16 candidates against the live clause library (21 real clauses; the
`GET /clauses` API is 500-broken so this was read direct from the `clause` table).

**Live clause library:**
- *Administrative:* Prepared For / Contact Details · Prepared By / Company Details
- *Commercial:* Fees / Payment Terms · Optional Services · Payment Schedule · Additional Payment Terms · Contract Details / Site Attendance · Proposal Validity
- *Legal:* Design Phase Notes · Limitation of Liability · Post Contract Phase Notes · Basis of Appointment · Defined Role
- *Services:* Preliminaries · Stage X / Xa / Xb [Service] · Construction Supervision · Post Completion / DLP · Handover & Close Out

**Verdicts** (covered = drop as redundant · partial = verify clause body before deciding · GAP = genuine standalone assumption):

| # | Candidate (abridged) | Verdict | Maps to |
|---|---|---|---|
| 1 | Client appoints LDC to coordinate disciplines | **GAP** (client-obligation angle) | Defined Role / Basis of Appointment define *emittiv's* role, not the client's duty to appoint |
| 2 | Arch/structural/MEP info provided timely, free | **GAP** | — |
| 3 | Client/LDC provides complete design brief pre-start | **GAP** | — |
| 4 | Site access arranged by Client/Contractor, free | covered | Contract Details / Site Attendance |
| 5 | Programme allows adequate time per stage | partial | Design Phase Notes (verify) |
| 6 | Single design iteration; redesign = extra fees | partial | Design Phase Notes / Additional Payment Terms (verify) |
| 7 | Scope = arch/decorative lighting; emergency/exit/aviation/specialist excluded | **GAP** | Services clauses list inclusions, not these exclusions |
| 8 | ~~Landscape excluded~~ | DROP | unsafe default (often in scope) |
| 9 | ~~BIM/Revit excluded~~ | DROP | unsafe default (often in scope) |
| 10 | Equipment procurement/supply/install by others | **GAP** | — |
| 11 | Main Contractor provides power to luminaire positions | **GAP** | — |
| 12 | Mock-ups = separate fees/reimbursable | covered | Optional Services |
| 13 | Fees based on scope/scale at proposal time | covered | Contract Details / Proposal Validity |
| 14 | Fees exclusive of VAT/withholding | covered | Fees / Payment Terms |
| 15 | Travel/accommodation reimbursable at cost | partial | Fees / Payment Terms (verify) |
| 16 | No unreasonable delay; suspension >[X]mo = reassess | partial | Proposal Validity (verify) |

> Title-level mapping. "covered"/"partial" verdicts should be confirmed against the
> clause *body text* before final removal — only the **GAP** rows are confidently new.

### Recommended standard assumptions (the genuine gaps — your selection)
1. The Client shall appoint a Lead Design Consultant to coordinate all design disciplines and issue consolidated information packages.
2. The Client/LDC will provide a complete design brief and all relevant architectural, structural, and MEP drawings/specifications in a timely manner at no cost to emittiv. *(merges candidates 2 + 3)*
3. Services are limited to architectural / decorative lighting design; emergency, exit signage, aviation obstruction, and specialist systems lighting are excluded unless explicitly stated.
4. Lighting equipment procurement, supply, and installation are by others.
5. The Main Contractor shall provide adequate power provisions to all luminaire positions as indicated on the lighting layout drawings.

Then decide partials 5/6/15/16 after reading the relevant clause bodies, and whether
to implement these as **toggleable assumptions in the scope-assembly system**.

### Side findings (2026-06-14, while pulling clauses)
- **BUG:** `GET /clauses` on e-fees-scope (10.0.21.81:3201) returns 500 — `Failed to deserialize field 'created_at' on type 'Clause': Expected datetime, got string`. The classic v3 `Datetime`-vs-`String` gotcha; the list endpoint is unusable until the `Clause` model uses `surrealdb_types::Datetime`. `/clauses/categories` and `/clauses/{id}` paths unaffected.
- **Test hygiene:** the `clause` table holds ~25 leftover `DELETE ME` clauses (categories "DELETE ME - *") from smoke runs that cleanup never removed.

## Sources
- Corpus: 51 ingested PDFs in `proposal_corpus` table (SurrealDB 10.0.23.11)
- Live clause library read 2026-06-14 from the `clause` table (4 categories, 21 real clauses)
- Web research: RIBA sub-consultant appointment terms, MEP/lighting consultant fee proposal standards
