# Clause Library Audit

**Scope:** 21 curated library clauses audited against the CLAUSE-CATALOG.md, which consolidates 1,683 clauses from 69 historical Emittiv fee proposals grouped into 23 types.

**Audit date:** 2026-06-14

---

## Part 1: Per-Clause Verdicts

### 1. Preliminaries
- **Category:** Services | **Library ID:** `clause:78e6plb1kqtjdn9t00ix`
- **Catalog match:** Preliminaries — Variant A (standard deliverables block)
- **Verdict: THIN**

The curated body matches the catalog's Variant A deliverables block and required-information list almost exactly, but the library version uses a "Stages X" preamble sentence *before* the block in many real proposals (Variant B), and the catalog's canonical required-information list includes **"Any established package budgets"** as the final item. The library clause omits it. The catalog also lists "Program and Area allocations" whereas the library lists "Programme and area allocations" - a trivial capitalisation drift.

**What's missing:** `- Any established package budgets` (present in 24+ historical proposals).

The catalog canonical also uses the plural "Typical deliverables include:" lead (rather than "Preliminaries" as a heading over the deliverables). The library adds an "access to online storage / collaboration platforms" and "Introduction to existing design team members" line that do not appear in any catalog variant - these are library-only additions (not wrong, just not drawn from history).

---

### 2. Defined Role
- **Category:** Legal | **Library ID:** `clause:ue3apbntxruyj0yef7ww`
- **Catalog match:** Defined Role — Variant A (standard, lighting/video/sound)
- **Verdict: OUTDATED / DIVERGENT**

The curated body uses the generic placeholder `[Company Name]` and says "the design of the lighting [and associated systems as indicated]". The historical canonical (Variant A, 61/69 documents) reads:

> **"emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team."**

Key differences:
1. The library clause omits the "Regulations / standards / landlord guidelines" paragraph that appears in 61 of 69 historical proposals. This is a material omission - it establishes the client's obligation to disclose applicable standards.
2. The library uses `[Company Name]` placeholder throughout; historical proposals all use "emittiv" in running text.
3. "The design of the lighting [and associated systems as indicated]" is a weaker scope statement than the full historical "lighting, video, and sound [as indicated in the Packages section above]".

---

### 3. Prepared For / Contact Details
- **Category:** Administrative | **Library ID:** `clause:6ya3aapu3od7ifpimo1s`
- **Catalog match:** Prepared For clause type
- **Verdict: OK (template)**

The curated clause is a placeholder template. Its field layout (Name / Position / Company / City, Country / Email / Phone) exactly matches the catalog's recommended standard field order. No historical wording to diverge from - this is inherently per-recipient. One minor note: the catalog recommends "U.A.E." (with periods) rather than leaving country as free text; the template has "Client City, Country" which is correct as a placeholder.

---

### 4. Prepared By / Company Details
- **Category:** Administrative | **Library ID:** `clause:ug2oxrlmvl2byfq9f2v1`
- **Catalog match:** Prepared By clause type - Martin Robert signature (lighting)
- **Verdict: OUTDATED / DIVERGENT**

The curated clause uses generic placeholder text (Name, Title, Company Name, etc.). The dominant historical signature across 58+ documents is a specific, named block:

> **Martin Robert / Founder and Lighting Director / emittiv llc-fz / Dubai, UAE / Email: martin@emittiv.com / Phone: +971.5858.555.69**

The library clause treats this as fully generic template, which is correct for a "Prepared By" placeholder, but the template phone format (`+XX.5555.555.55`) is fictitious and doesn't match the canonical dotted-format (`+971.5858.555.69`). The catalog recommends standardising on `emittiv llc-fz` as the entity name (some historical variants used "emittive design" or "emittiv design" typos). The library template doesn't reinforce this.

**Practical recommendation:** Replace with the actual Martin Robert canonical signature or at minimum document the canonical field values in the clause body/notes.

---

### 5. Contract Details / Site Attendance
- **Category:** Commercial | **Library ID:** `clause:tdadxdufwfs39f2dmb3s`
- **Catalog match:** Contract Details & Site Attendance — combined form (Variant 4, Dubai office)
- **Verdict: OK**

The curated clause correctly captures the two standard components (Anticipated Programme caveats + Site Attendance paragraph). The structure matches Variant 4 (the dominant combined form, 37+ proposals). The programme caveat sentences match the catalog canonical ("do not take into consideration Client review or Revision periods", "confirmed with the Client during Preliminaries", "may require additional staff and fees"). The Site Attendance paragraph matches Variant 1/4 ("Client's [Location] based office", "does not provide for any travel out of country", "prefers remote meetings wherever possible").

Minor: the library uses the `[Company Name]` placeholder; historical proposals use "emittiv". Not a content divergence.

---

### 6. Fees / Payment Terms
- **Category:** Commercial | **Library ID:** `clause:f4z5ib331d3bnca1y7mw`
- **Catalog match:** Design Phase Fees (Group A canonical) + Payment Terms (Variant 4)
- **Verdict: THIN / DIVERGENT**

The curated clause combines fee applicability preamble, fee variation notice, and payment terms into one block. Comparing to the catalog:

**Fee preamble:** The opening sentence matches Group A canonical: "These rates are applicable if the complete scope described in this proposal is awarded to [Company Name]. Variations to the proposed scope may result in adjustments to the nominated fees." - OK.

**Fee variation notice:** "[Phase] Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage." - This is the post-contract estimate qualifier language (catalog Group I), not the standard design-fee notice. The canonical design fee variation notice reads: **"Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after [Stage] has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time."** The library omits this critical sentence from the fee section.

**Payment Terms:** The library includes "All prices shown are in [Currency]", VAT, mobilisation, 30-day payment window, monthly invoicing, back-to-back/retention refusals. The historical canonical (Variant 4, most common, 25+ docs) uses **14 calendar days** not 30. The 30-day form (Variant 5) is used on fewer proposals. This is a material difference.

**What's missing:** The canonical "Before exceeding the above fee..." variation notice in the fees section. Daily billing rate reference line in the payment terms.

---

### 7. Proposal Validity
- **Category:** Commercial | **Library ID:** `clause:vcwdo4kiwzp96ttpwyua`
- **Catalog match:** Proposal Validity — Variant A (60 days)
- **Verdict: DIVERGENT**

The curated clause reads: "This proposal is valid for [XX] days from the date of issue." The catalog canonical (62 of 69 documents) reads: **"This proposal shall remain valid for a period of sixty [60] days from the date of issue."**

Key differences:
1. "valid for [XX] days" vs "shall remain valid for a period of sixty [60] days" - the catalog recommends the bracketed numeral form.
2. The library uses a variable placeholder `[XX]`; the default should be sixty [60] with 90 or 120 as explicit named overrides.
3. The catalog's canonical adds "After this period..." as a follow-on (the library's second sentence "After this period, [Company Name] reserves the right to review and adjust..." is a library-only addition not found in any historical variant).

---

### 8. Payment Schedule
- **Category:** Commercial | **Library ID:** `clause:q63xowhds0myb1of9cf4`
- **Catalog match:** Payment Schedule — Variant B (markdown table)
- **Verdict: OK (template)**

The library clause is a markdown table placeholder with Mobilisation, Stage 1, Stage 2, and Total rows. This matches the catalog Variant B (pipe/markdown table form) structure and includes Mobilisation as a percentage - consistent with historical practice (30% mobilisation dominates). The library uses `[XX]%` for mobilisation, which is correct as a template. The table structure (Stage | Milestone | Fee | Payment columns) matches the catalog recommendation. Verdict is OK as a skeleton; the actual percentages and amounts are project-specific.

---

### 9. Additional Payment Terms
- **Category:** Commercial | **Library ID:** `clause:r7iawtffuw2x42vr77qo`
- **Catalog match:** Payment Terms — "Additional Payment Terms" block (the trailing section of Variant 4/5)
- **Verdict: OK**

This clause matches the historical Additional Payment Terms block closely. All key elements are present: client payment responsibility, bank account statement, stable-coin discount offer (USDT/USDC), daily billing rate with rate card reference, additional expenses clause, stage-gating requirement, and revision/additional-services charging basis.

Minor notes: The library uses `[Currency] [Amount]` placeholders for daily rate. The catalog's canonical is AED 4,500 (most common) or AED 5,000 (recent proposals). The library's stable-coin discount shows `[X]%`; the current standard across recent proposals is 2.5%. These are correct as parameterisable fields.

The library clause also includes "Documentation and payment approvals are required before work on the next stage will begin" and "Further revision of the designs and resubmissions, if required, will be charged at the nominated rates" - both present in the historical canonical. No gaps.

---

### 10. Design Phase Notes
- **Category:** Legal | **Library ID:** `clause:a19bha8li1qo1rsswzmo`
- **Catalog match:** Design Phase Notes — Family A, Variant A1 (8-point standard)
- **Verdict: THIN**

The curated clause captures 8 of the 9 points in the historical canonical. **Missing: the "illustrative renders" point** (Variant A2), which appears in 12 recent proposals (2024-2026) and is the catalog's recommended canonical:

> *"Any images prepared by emittiv shall be illustrative in nature, and intended to convey the overall design intent. If realistic renders are required, these shall be prepared by the Architect with inputs and comments by emittiv."*

The catalog explicitly recommends adopting Variant A2 (9 points, with the renders clause) as the new standard default. The library clause is the older 8-point Variant A1. All 8 existing points in the library match the catalog verbatim (modulo `[Company Name]` vs "emittiv").

---

### 11. Optional Services
- **Category:** Commercial | **Library ID:** `clause:lp9za61q0l6f16narhdh`
- **Catalog match:** Optional Services — Variant A (standard full list, 13 items)
- **Verdict: OK**

The library clause matches Variant A closely. All 13 items from the catalog canonical are present:
- VR / AR / XR Simulations
- Designs for shows within performance spaces or systems
- Content creation for video and sound systems
- Sunlight design and studies
- Mock-ups / models
- Custom designed luminaires
- Formal review of tender documents
- Review and checking of proposed substitute [devices/luminaires]
- Obtaining project related quotations
- Shop and field inspections
- Whole life cost and running cost reports
- Lighting calculations (energy compliance)
- Final lamp schedule
- As built / installed drawings

**Note:** The library adds two items not in any historical clause: "Lighting calculations for regulatory or energy efficiency compliance (to be performed by the Electrical Engineer)" and removes nothing, making it a 15-item variant. The extra item is sensible but is a library-only addition. The catalog Variant A uses "devices/luminaires" for the substitute-check line; the library uses neither term explicitly. Minor.

---

### 12. Assumptions
- **Category:** Legal | **Library ID:** `paw4ejkcmhtwpm0l2miq` (created 2026-06-14, no tags)
- **Catalog match:** Assumptions — Variant 2 (designer/engineer coordination boundary, long form)
- **Verdict: OK**

The curated clause matches the catalog's Variant 2 (the most complete, recommended form) closely. All 8 coordination-boundary bullets are present and match the historical wording. Additionally, it includes the site-visit additional-charge line from Variant 3:
> *"In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal."*

This is the catalog recommendation (append variant 3's line to variant 2 as standard). The library clause also uses "emittiv" (not `[Company Name]`), which is correct. The preamble ("The following assumptions have been made...") is present and matches.

**Minor note:** This clause was created 2026-06-14 with no tags and no sort_order in the right position. Consider adding tags (["standard"]) and confirming sort_order placement between Optional Services (70) and Stage X clauses (80).

---

### 13. Stage X - [Service Name] (generic stage template)
- **Category:** Services | **Library ID:** `clause:bvuls8qos0ig7z1wp5uv`
- **Catalog match:** Stages — Detailed Design (Family D, Variant D2 with LOD 300)
- **Verdict: THIN**

The curated clause ("further develop and identify the specifics of the systems to the level that they can be issued for tender and finalised by the awarded contractor for construction from a '[Perspective]' perspective") is the Detailed Design / Basis of Design body. The catalog identifies this as the standard third-stage body and recommends:

**Missing from library clause:** The catalog canonical (D2/LOD 300 body) includes:
- "Carry out further development of package specific 3D layouts based on linked files to LOD 300 standards" - **absent from library**

The library clause has 9 deliverable bullets. The catalog D2 canonical adds the LOD 300 line between "Attend design workshops" and "Prepare final package layouts". This is the superset recommended by the catalog.

Also: the `[Perspective]` placeholder is used in the library, which maps to "Basis of Design" for this stage. Historical proposals consistently say "Basis of Design" not "Open Tender" for the Detailed Design stage; "Open Tender" appears only in the separate Tender Documentation stage.

---

### 14. Stage Xa - [Service Name] (tender documentation)
- **Category:** Services | **Library ID:** `clause:k0ijg2fg5mk3vzisvu7d`
- **Catalog match:** Stages — Tender Documentation (Family E, Variant E1, "Open Tender")
- **Verdict: DIVERGENT**

The curated clause body has "from a '[Perspective]' perspective" and "Final system calculations and compliance with relevant Building Regulations. Final specification and schedule of devices, accessories and associated equipment." - this matches the "Open Tender" canonical.

However, the curated deliverables are **partial**: the catalog canonical (E1) has 5 bullets; the library clause has only the first sentence plus 4 bullet equivalents but is structured as narrative rather than a bulleted list. Specifically, the library text runs the deliverables as a paragraph rather than discrete bullets - harder to read and not matching the proposal format.

**More importantly:** The stage name is "Stage Xa - [Service Name]" which maps to the tender sub-stage (4a), but the body's `[Perspective]` placeholder conflates Basis of Design and Open Tender. In practice, Stage Xa should always read "Open Tender perspective" - this should be pre-filled rather than left variable.

---

### 15. Stage Xb - [Service Name] (tender return review)
- **Category:** Services | **Library ID:** `clause:53k2oa0to0zvot724xmz`
- **Catalog match:** Stages — Tender Return Evaluation (Family F)
- **Verdict: OK**

All three catalog bullets are present verbatim:
- "Review and checking of proposed substitute devices, as selected by others, for compliance with specification."
- "Review and analyse of submittals based on technical compliance, company experience and other factors as required."
- "Prepare scoring matrix comparing submitted package tenders."

Minor: the catalog notes "Review and analyse of submittals" should be corrected to "Review and analysis of submittals" (grammar). Both the catalog and the library retain the grammatically incorrect form.

---

### 16. Stage X - Construction Supervision
- **Category:** Services | **Library ID:** `clause:ndklg1t9dht7z4qges5x`
- **Catalog match:** Stages — Construction Supervision (Post Contract Phase Notes B, standalone deliverables)
- **Verdict: OK**

The library clause captures all key Construction Supervision deliverables. Comparing to the catalog canonical (B1, post-contract supervision):
- Contract meetings: present
- Review shop drawings / submittals: "Review shop drawings and submittals for compliance with design intent" (library) vs "Review Contractors production information" (catalog short form). The library wording is more specific and matches the fuller HoH-supervision variants.
- Resolution of site queries: present
- RFI review: library includes this; catalog short form does not. This is an addition - not from the minimal canon but consistent with fuller variants.
- Clarification sketches / supplementary details: library only
- Coordinate with Contractor and Electrical Engineer: library only
- Mock-ups and first-fix installations: library only
- Final review of package installation: present
- Personnel compliance note: present ("Package Contractor shall supply personnel...")

The library clause is **richer** than the catalog's minimal short form but consistent with fuller variants used on larger projects. No gaps vs. the canon; several additions are defensible.

---

### 17. Stage Xa - Focussing / Aiming and Scene Setting
- **Category:** Services | **Library ID:** `clause:dx21vc45faplx3nwf4tu`
- **Catalog match:** Stages — Focussing / Aiming and Scene Setting (Post Contract Phase Notes B2/B6)
- **Verdict: OK**

All four "Important Notes" from the catalog canonical are present verbatim. The two deliverable bullets ("Supervision of focusing / aiming of adjustable devices" and "Oversee the programming of installed control equipment where applicable") match the catalog. The scene-setting closing line ("emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Contractor]") is present and matches.

Minor: library says "adjustable devices" (plural); catalog says "adjustable device" (singular) in the short form. This is an improvement.

---

### 18. Post Contract Phase Notes
- **Category:** Legal | **Library ID:** `clause:itqptjnqnrkbpzxtd3m5`
- **Catalog match:** Post Contract Phase Notes — Variant A1 (standard, 2 bullets)
- **Verdict: OK**

The curated clause exactly matches the catalog's canonical A1 form (the most common, 17 documents):
- "This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site."
- "In the event of [Company Name] visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal."

Both bullets are present and verbatim. The `[Company Name]` placeholder maps to "emittiv" in real use.

---

### 19. Stage X - Post Completion / DLP
- **Category:** Services | **Library ID:** `clause:dozjvke544b8u0vtxqcj`
- **Catalog match:** Stages — Post Completion / DLP (Post Contract B12, short form)
- **Verdict: OK**

Both catalog bullets are present:
- "Assistance with providing advice on any defects reported by the contract administrator during the defects liability period up to a maximum of 12 months from practical completion."
- "Conduct operational reviews and assessments as appropriate."

This matches the catalog B12 short form verbatim.

---

### 20. Stage X - Hand Over and Close Out
- **Category:** Services | **Library ID:** `clause:y41zwa9vow7gwcoaglp2`
- **Catalog match:** No direct catalog type equivalent
- **Verdict: LIBRARY-ONLY**

The catalog has a "Commissioning / Handover and Defects" section (B10, B11) but that covers defects assistance and basic training only. The library clause is a comprehensive handover/closeout block including:
- Hand over documentation package
- As-built / as-installed documentation
- Final site inspections and snagging
- O&M manuals
- Training sessions
- Final lamp/luminaire schedule
- Close-out report
- Archive and digital asset transfer

None of the 69 historical proposals contain this exact combined clause. The B10 historical variant (HoH) covers defects-report assistance and training with day allowances but does not include the "prepare close-out report", "archive project files", and "transfer digital assets" items. These are well-conceived additions for a modern clause library but are not historically attested.

This clause is fine as a library-only addition; there is no historical comparator to diverge from.

---

### 21. Basis of Appointment
- **Category:** Legal | **Library ID:** `clause:ljnf9ml3o7dqtft95u01`
- **Catalog match:** Basis of Appointment (Variant A1, ACE PSA 2017) + Limitation of Liability + Next Steps
- **Verdict: OK**

The curated clause correctly captures three components:

**Basis of Appointment:** ACE PSA 2017 reference, governing-document protection, and DIFC Courts jurisdiction - all match Variant A1 verbatim (the dominant form, 61/69 documents). The "be read as contained herein" wording matches the A1 sub-variant (not the A2 "and read as contained herein" drop). Good.

**Limitation of Liability:** All three paragraphs present and match the catalog canonical (67/69 documents): consequential-loss exclusion, liability cap ("shall not exceed the amount of compensation actually received"), force majeure.

**Next Steps:** "issue an LPO mentioning the document reference [Document Reference]" - this paragraph does not appear in any catalog variant but is a practical procedural addition. Library-only, no issues.

---

## Part 2: Gap Analysis - Catalog Types Without Curated Clauses

The 23 catalog types and their library coverage:

| # | Catalog Type | Library Clause? | Status |
|---|---|---|---|
| 1 | Cover & Intro (Company Profile, Cover Title Block, RFP Receipt, Project Details, Document Control, Packages/Stages) | None | **GAP** |
| 2 | Prepared For | Prepared For / Contact Details | Covered |
| 3 | Prepared By | Prepared By / Company Details | Covered (thin) |
| 4 | Scope & Services (Appointment confirmation, Areas in/excluded, Services division statement) | None as standalone | **GAP** |
| 5 | Stages (stage-list overviews + individual stage deliverables) | Partially covered via Stage X clauses | Partial |
| 6 | Design Phase Notes | Design Phase Notes | Covered (thin - missing renders clause) |
| 7 | Post Contract Phase Notes | Post Contract Phase Notes | Covered |
| 8 | Design Phase Fees | Partially in Fees / Payment Terms | Partial |
| 9 | Post Contract Fees | None as standalone | **GAP** |
| 10 | Payment Terms | Fees / Payment Terms + Additional Payment Terms | Covered |
| 11 | Payment Schedule | Payment Schedule | Covered |
| 12 | Reimbursables & Provisional Sums | None as standalone | **GAP** |
| 13 | Assumptions | Assumptions | Covered |
| 14 | Exclusions | None as standalone | **GAP** |
| 15 | Proposal Validity | Proposal Validity | Covered (divergent) |
| 16 | Limitation of Liability | Embedded in Basis of Appointment | Covered |
| 17 | Basis of Appointment | Basis of Appointment | Covered |
| 18 | Defined Role | Defined Role | Covered (divergent - missing regulations paragraph) |
| 19 | Contract Details & Site Attendance | Contract Details / Site Attendance | Covered |
| 20 | Optional Services | Optional Services | Covered |
| 21 | Preliminaries | Preliminaries | Covered (thin) |
| 22 | Scope & Services - Stage deliverables (Concept, Schematic stages) | None as dedicated clauses | **GAP** |
| 23 | Focussing / Aiming | Stage Xa | Covered |

### Significant Gaps to Fill

**GAP 1: Scope & Services - Appointment Confirmation + Areas Scope block**
The RFP acknowledgement / appointment confirmation ("emittiv confirms that we are prepared to accept...") and areas-included/excluded block appear in nearly every proposal. No library clause covers this. The catalog provides a clear canonical (C1 for lighting-led public-realm projects). This is arguably the most-used clause family in the corpus outside of the fee/legal sections.

*Canonical ready to adopt:* Yes - Appointment confirmation (A1) and standard lighting areas block (C1) are both clear and stable.

**GAP 2: Cover & Intro - Company Profile**
The "About emittiv" company introduction text appears in 7+ proposals verbatim and is the most reused boilerplate. The library has no cover/intro clause at all.

*Canonical ready to adopt:* Yes - the full 7-proposal verbatim version is well-established.

**GAP 3: Post Contract Fees**
A standalone post-contract fee schedule clause (estimate disclaimer + stage schedule with quantities/rates) is absent from the library. The design-fee section is covered but post-contract (supervision, scene-setting, DLP) scheduling has no template.

*Canonical ready to adopt:* Yes - the catalog Group E/I pattern with the estimate disclaimer boilerplate.

**GAP 4: Exclusions**
No dedicated Exclusions clause in the library. The catalog shows this is a heavily reused clause type (lighting/audio/control discipline-organised list) that appears in a large subset of proposals.

*Canonical ready to adopt:* Yes - the discipline-organised canonical from Dammam Water Park (lighting + audio + control systems).

**GAP 5: Reimbursables & Provisional Sums**
No daily-rate + travel-expense projection clause. Relevant for any proposal that includes reimbursable costs.

*Canonical ready to adopt:* Partially - Variant A1 for the daily-rate + payment-terms paragraph; the reimbursable projection table is project-specific.

**GAP 6: Concept Design stage clause**
The library has a generic "Stage X" and specific post-contract stages but no Concept Design Report clause. This is the most-used individual stage deliverable (25+ proposals).

*Canonical ready to adopt:* Yes - the Specialist Lighting Concept Design Report body (B1) with discipline as the only variable field.

**GAP 7: Schematic Design stage clause**
Same gap as Concept Design. No Schematic Design Report clause despite it appearing in 20+ proposals.

*Canonical ready to adopt:* Yes - the LOD 200 full body (C1) is the recommended standard.

---

## Part 3: Notable Historical Variants Worth Capturing

The following distinct, reusable clause variants from the corpus are not represented in the library but would add real value:

**1. Existing cable infrastructure / retrofit assumption (Assumptions Variant 4)**
> "It is assumed that all cable infrastructure is functional and fit for purpose. This proposal does not provide for replacing or re-pulling of any cables. The Client will provide complete as built system drawings and access to all appropriate equipment as required."

Relevant whenever emittiv is engaged on an upgrade/retrofit project. No current library clause covers this.

**2. BMS/GRMS tie-in scope limitation (Assumptions Variant 7)**
> "Given that these Suites form part of a larger hotel operation, it is assumed that the sensory systems shall be required to tie in to the site wide BMS / GRMS. As a result, our proposal is limited to Concept and Schematic design stages, to allow the systems to be incorporated into the master system by others."

Useful for hotel suites and integrated-controls projects. Distinct enough from the generic assumptions block to deserve its own library entry or a tagged variant.

**3. Interior-only scope block (Scope & Services Variant C2)**
> Lighting: Interior Architectural Lighting + Lighting Control Systems. Excludes: Emergency Lighting, BoH Areas. Optional Audio: Ambient Audio Systems + Audio Control Systems. Excludes: PAVA / Emergency systems.

Used across hospitality and F&B projects (Reserve Cut, Level 63, Shanghai Tang). The library has no scope-limitation clause for interior-only appointments.

**4. Theme-park / attraction exclusions (Exclusions - Shoot the Chute variant)**
The ride-structure / content / animatronics exclusion clause is highly specific to attraction projects and has no equivalent anywhere in the library. Worthwhile addition given the FEC/theme-park volume of the portfolio.

**5. Late payment / suspension of works clause (Payment Terms Variant 6)**
> "Late or non-payment of invoices may result in the suspension of works, at the sole discretion of the Consultant."

Appears in the HoH supervision proposals. Not included in the current Additional Payment Terms library clause. A useful protection clause for high-value supervision engagements.

**6. Involvement-tier rate card (Post Contract Fees Variant J, WAMI)**
The tiered off-site/on-site involvement scale (Light/Medium/Heavy × 6 tiers) is a novel pricing structure for scope-uncertain post-contract engagements. Worth capturing as a library clause type for complex or phased supervision work.

**7. "No interaction with attractions" assumption (Assumptions Variant 6)**
> "No interfaces or interaction shall be required with attractions."

A one-line scoping assumption with high relevance to FEC and theme-park projects. Simple but prevents scope disputes.

---

## Part 4: Prioritised Recommendations

### Priority 1 - Fix material gaps in existing clauses (do now)

**1a. Update Defined Role** - Add the missing "Regulations / standards / landlord guidelines" paragraph. This paragraph appears in 61/69 proposals and is a material client obligation; its omission leaves the library clause incomplete for legal use.
Adopt verbatim: *"Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE. This includes any country, city, state, landlord, client, tenant regulations or guidelines."*

**1b. Update Design Phase Notes** - Add the "illustrative renders" bullet (Variant A2). The catalog explicitly recommends A2 as the new canonical default. Insert after the "CAD files" bullet:
*"Any images prepared by emittiv shall be illustrative in nature, and intended to convey the overall design intent. If realistic renders are required, these shall be prepared by the Architect with inputs and comments by emittiv."*

**1c. Update Proposal Validity** - Standardise to the historical canonical wording: *"This proposal shall remain valid for a period of sixty [60] days from the date of issue."* Remove the non-historical "After this period... reserves the right to review and adjust" sentence (or clearly mark it as a library addition). The 60-day default should be pre-filled, not left as `[XX]`.

**1d. Update Fees / Payment Terms** - Add the canonical fee variation notice. After the fee table placeholder, insert: *"Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after [Stage] has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time."* Also change the payment window to 14 days (the historical dominant default) and make 30 days an explicit variant.

**1e. Update Preliminaries** - Add "Any established package budgets" as the final required-information item (present in 24+ historical proposals, absent from library clause).

### Priority 2 - Add missing clause types (high value)

**2a. Add: Scope & Services - Appointment Confirmation + Areas block**
This is the most-used clause family in the corpus with no library equivalent. Add two clauses:
- Appointment confirmation (the three-sentence "emittiv confirms..." block)
- Standard areas included/excluded (lighting public-realm: Public Realm / Interior / Facade / Landscape / Control Systems; excludes BoH / Shell & Core / Street Lighting requiring Statutory approval)

**2b. Add: Concept Design stage clause**
The most-used individual stage deliverable (25+ proposals). Canonical body: "We will prepare a [Discipline] Concept Design Report for the Project which will provide guidelines for the Client and other Designers..." with the full 7-bullet deliverables list, closing with "We have allowed for up to one revision of the Concept Design Report."

**2c. Add: Schematic Design stage clause**
Second most-used stage clause (20+ proposals). Canonical body: "In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase." with the full LOD 200 list (13 bullets including "3D layouts based on linked files to LOD 200 standards"), closing with revision line.

**2d. Add: Exclusions clause**
Discipline-organised canonical (Dammam Water Park form, drop Rides section for non-attraction work):
- Lighting: Emergency Lighting Systems, BoH Areas, Shell and Core tenanted spaces, Street Lighting requiring Statutory Authority's approval
- Audio: PAVA Systems [with tie-in note]
- Control Systems: Site wide IT networking [with connectivity note]

**2e. Add: Company Profile / About emittiv**
The 7-proposal verbatim boilerplate. Highest-reuse non-templatable text in the corpus. Should be a library clause with status active and a note that it is the standard company profile for cover/intro sections.

### Priority 3 - Address in next iteration

**3a. Add: Post Contract Fees template** - Standalone clause for post-contract fee scheduling with the estimate-disclaimer boilerplate and a parameterisable stage/quantity/rate table.

**3b. Add: Exclusions (attraction/ride variant)** - The Shoot the Chute ride-structure / content / animatronics exclusion clause for FEC and theme-park proposals.

**3c. Add: Retrofit / existing-cable assumption** - Variant 4 of the Assumptions catalog for upgrade/refurbishment projects.

**3d. Add: Interior-only scope block** - C2 variant (Interior Architectural Lighting + Control; excludes Emergency + BoH; optional Audio section) for hospitality and F&B appointments.

**3e. Leave as-is:** Stage Xb (Tender Return Review), Stage X - Construction Supervision, Stage Xa - Focussing/Aiming, Stage X - Post Completion/DLP, Post Contract Phase Notes, Payment Schedule, Additional Payment Terms, Basis of Appointment, Stage X - Hand Over and Close Out. These are OK or Library-Only and need no changes.

---

## Summary Counts

| Verdict | Count |
|---|---|
| OK | 9 |
| THIN (missing content vs canon) | 4 |
| OUTDATED / DIVERGENT | 4 |
| LIBRARY-ONLY (no historical comparator) | 1 |
| Template/placeholder (OK as-is) | 3 |

**Catalog types with no library clause (GAPs):** 7 significant gaps identified, covering: Scope & Services appointment confirmation + areas block, Company Profile, Post Contract Fees, Exclusions, Reimbursables, Concept Design stage, and Schematic Design stage.
