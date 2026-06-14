# Emittiv FP Clause Catalog

Consolidated, deduplicated catalog of every clause variant found across **69 historical fee-proposal documents** (50 projects, across the Inactive / Current / Completed folders), generated 2026-06-14 by mining the `02 Proposal` PDFs. **1,683 clauses** were extracted and grouped into 23 types; each section below lists the distinct variants, which proposals use each, and a recommended canonical wording where one is clear.

Companion files: `INDEX.md` (per-doc index with client + fee), `archive/` (faithful verbatim record per document).

---

## Cover & Intro

This clause type covers the opening material of a Fee Proposal: the cover-page title block, the company profile/about-emittiv boilerplate, the RFP-receipt and acceptance statement, project-details and packages/stages blocks, and the document-control / confidentiality footer. The variants below are grouped by function. Many are per-project one-offs that differ only in the project name, location, and reference number - these are templated fills of a common skeleton rather than distinct clauses, and are consolidated as such.

---

### 1. Company Profile (about emittiv)

The longest-lived, most reused boilerplate. The canonical full version appears verbatim across 7 proposals. A second instance (Neom Trojena Observatory) is the same text inline-formatted with a trailing Confidential paragraph, and a third (Wynn EL-FP) is a trimmed version that drops the project-types list and the website line.

Full version (verbatim representative):

> emittiv is a multidisciplinary sensory design consultancy working with lighting, video, sound, scent, and control systems for the built environment. We design experiences, not just systems - making sure all of our elements work together to deliver a unified and polished concept.
> We work closely with Architects, Interior Designers, Lead Design Consultants and Owners / Operators on projects worldwide.
> Our aim is to enhance people's everyday lives by adding layers that make our spaces more interesting and beautiful.
> Our creative and commercial experience enables us to develop intelligent and appropriate designs within the parameters of budgets and time frames.
> We are entirely independent of any manufacturers or distributors, and operate on a fee for service basis.
>
> Our combined experience covers all types and scale of projects including:
> Attractions, Bars, Brand Activations, Casual Dining Venues, Cinemas, Corporate Headquarters, Department Stores, Destination Locations, Family Entertainment Centres, Galleries, Gaming Venues, Hotels, Landmark Buildings, Landscape, Malls, Museums, Nightclubs, Offices, Parks, Performance Venues, Public Realm, Residential, Retail Flagships, Shopping Centres, Spas, Theatres, Theme Parks.
>
> We can be appointed in a number of ways and tailor our services to a specific project and Clients' needs.
> All project teams are overseen by a Director and/or Associate to ensure a high level of service at all times.
> For further information on our services and project experience please visit our website: www.emittiv.com

Variant differences: 24-96605 Observatory is the same text run inline (project-types list comma-joined) plus a Confidential paragraph; 23-97102 Wynn EL-FP omits the "Our combined experience covers..." project-types paragraph and the final website line.

Used by: 23-97106 Fountain Control, 24-96603 Marasi Gate, 24-97106 RAK Sled, 25-97103 Aljada FG, 25-97103 Aljada Block F+G, 26-96801 Rozana Muscat, 25-97101 Shanghai Tang, 24-96605 Observatory (inline + Confidential), 23-97102 Wynn EL-FP (trimmed)

---

### 2. Cover Title Block

The cover page identity block: optional "make:sense" sub-brand line, "Fee Proposal", the service-line descriptor, project name, location, the proposal reference number, and a release line. Roughly 40 per-project instances share this skeleton, differing only in the variable fills (service line, project, location, number, release). They are one clause, templated.

Representative (verbatim):

> make:sense
>
> Fee Proposal
> Specialist Lighting Sitewide Lighting Consultantcy
> Wynn Al Marjan Island
> Ras Al Khaimah, U.A.E.
> 25-97102-FP
> Release 1

Skeleton:

> [make:sense]
> Fee Proposal
> {Service line} Design and Consultancy
> {Project Name}
> {City, Country}
> {number}-FP
> Release {n}

Notable observed differences across instances: the "make:sense" sub-brand line is present in some, absent in others; release styled "Release 1" / "Release 01" / "Release 2" / "Release 02" inconsistently; reference number sometimes prefixed "e-" (e.g. e-22-96601-FP) and sometimes not (24-96603-FP); discipline suffixes appear in the number for split scopes (e.g. e-23-97102-AA-FP, e-23-97102-EL-FP-01, e-23-97101-LI-FP); the placeholder template uses "Project Name / Project City, Project Country / e-yy-cccnn-FP". Some cover blocks bundle the Document Control / Confidential block inline (see group 5).

Used by: 24-96606 JOH, 25-97102 WAMI, 22-96601 Dammam Waterpark, 22-97111 Palm Fountain, 22-97112 The View, 22-97114 HoH, 22-97115 Kids Activity Park AUH, 23-96601 Dammam Adv World, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-96604 DockX, 23-96605 Jumpoline +35 more (essentially every proposal)

---

### 3. RFP Receipt & Acceptance Statement

The standard opening paragraph confirming receipt of the client's RFP and emittiv's acceptance. Many instances also list the client name and reference documents inline. The core two acceptance sentences are identical across all; the surrounding RFP-source / reference-docs lines vary per project.

Representative core (verbatim):

> emittiv ["Consultant", "We"] have received a Request for Proposal from {Client} ["Client"].
>
> Reference documents provided by the Client, which form the basis of this proposal include:
> - {document list}
>
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.

Differences: client name and reference-document bullets are per-project; some instances append a Validity sentence ("This proposal shall remain valid for a period of sixty [60] days from the date of issue.") and roll Project Details / Packages / Stages into the same block (see groups 4 and 6); the placeholder template uses 'Client Company ["Client"]' and 'Email request of dd MMM yyyy'.

Used by: 22-97114 HoH (Hyperspace), 23-96601 Dammam Adv World (KCC), 23-97102 Wynn (WDD), 23-97105 Tape Theatre (Stickman Tribe), 23-97111 Dubai Island Promenade (Proscape), 24-97102 DMC 2A (P&T), 24-97108 Tamani Hotel (P&T), 24-97113 Level 63 (Conrad), 25-97106 MAF MOE FEC (Sim Leisure), 23-96606 KSA Pav Osaka (placeholder), 23-97110 HoH Dxb (Hyperspace), 24-97101 HoH Supervision (Hyperspace) +2 more (24-96606 JOH x2 - Martin Professional ME)

---

### 4. Project Details Block

States the project name and location, development status, and physical particulars (plot area, GFA, zones, spaces). Per-project content, common skeleton. Often emitted as part of the same paragraph as group 3 or group 6.

Representative (verbatim):

> The Project is known as {Project} and is located in {location}.
> The Project is a proposed new development and currently comprises of:
> - Total Plot Area: Approximately {x}
> - Total GFA / Built Up Area: Approximately {y}

Differences: particulars vary widely (plot area, GFA, floor counts, theatre/zone/area lists); "new development" vs "refurbishment" status varies; sometimes carries embedded Reference-documents and Packages lines.

Used by: 22-97114 HoH, 23-97105 Tape Theatre, 23-97111 Dubai Island Promenade, 24-97102 DMC 2A, 24-97108 Tamani Hotel, 24-97113 Level 63, 25-97106 MAF MOE FEC, 24-96606 JOH

---

### 5. Document Control & Confidentiality

The tracking/distribution register plus the Confidential notice. Appears as a standalone block or bundled into the cover (group 2). The Confidential paragraph is near-identical wherever it appears; the tracking/distribution lines are per-project dated rows.

Representative (verbatim):

> Document Control
> Tracking
> Date: {YYMMDD} | Release: {nn} | Author: MR | Reference: RFP
>
> Distribution
> Date: {YYMMDD} | Release: {nn} | Distribution: {names}
>
> Confidential
> This document contains sensitive commercial information.
> It must not be reproduced or distributed without the express written permission of emittiv.
> This document and its contents are only applicable to the named project, for the named Client.

Differences: some add "This document is maintained as part of a document register. Please confirm with the author that you are using the current version."; date format and pipe/slash separators vary; multi-row tracking for multi-release proposals.

Used by: 22-97115 Kids Activity Park AUH, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-97106 Fountain Control, 23-97108 Ciel Lobby, 23-97109 Ciel VIP, 23-97111 Dubai Island Promenade, 24-96601 Dammam Water Park, 24-96603 Marasi Gate, 24-97108 Tamani Hotel, 24-97110 Masdar B-02, 25-96601 Pit Stop Theatre +8 more

---

### 6. Packages, Stages & Areas Block

Bulleted lists of the technologies (packages), project stages, and in/out-of-scope areas the proposal covers. Per-project content, common skeleton; frequently emitted inside the same paragraph as groups 3 and 4.

Representative (verbatim):

> Packages
> This Proposal includes the following technologies [referred to as packages or systems]:
> - Lighting
> - Sound
> - Associated Control Systems
>
> Stages
> This proposal includes the following project stages:
> - Design Phase
>   - Stage 1 Concept Design
>   - Stage 2 Schematic Design [SD]
>   - Stage 3 Design Development [DD]
>   - Stage 4 Construction Documents [CD]
> - Post Contract Phase
>   - Stage 5 Construction Administration [CA]

Differences: package lists range from single-discipline (Specialist Lighting) to full sensory (Lighting/Video/Sound/Scent/Control); stage naming/numbering varies (Concept/Schematic/Detailed/Tender vs SD/DD/CD; JOH uses a Construction-phase set: System Engineering / Ongoing Management / Programming); some add an Areas block with explicit in-scope and excluded lists (e.g. 23-97110 HoH Dxb).

Used by: 23-96601 Dammam Adv World, 23-97102 Wynn, 23-97105 Tape Theatre, 23-97110 HoH Dxb, 23-97111 Dubai Island Promenade, 24-96606 JOH, 24-97108 Tamani Hotel, 25-97106 MAF MOE FEC

---

**Recommended standard wording:**

For the two clauses with a clearly stable canonical form:

**Company Profile** - use the full 7-proposal version verbatim (group 1, full version above), including the project-types paragraph and the website line. Standardise on "Release 01" two-digit form on the accompanying cover.

**RFP Receipt & Acceptance** - use the core block:

> emittiv ["Consultant", "We"] have received a Request for Proposal from {Client} ["Client"].
>
> Reference documents provided by the Client, which form the basis of this proposal include:
> - {document list}
>
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.
>
> This proposal shall remain valid for a period of sixty [60] days from the date of issue.

(The Validity sentence appears in the more complete instances and should be retained as standard.)

For the **Cover Title Block**, **Document Control & Confidentiality**, and **Packages/Stages/Areas** blocks, no single canonical text is appropriate because the variable content is intrinsic; standardise the *skeleton* (field order and the fixed Confidential paragraph) rather than a fixed body. Recommended fixed elements: the Confidential paragraph exactly as in group 5; "Author: MR | Reference: RFP" tracking format; two-digit "Release 01" styling throughout; and consistent use (or consistent omission) of the "make:sense" sub-brand line - currently mixed.

---

## Prepared For

The "Prepared For" clause is the recipient block at the head of each fee proposal: contact name, position, company, city/country, email, and phone. It is per-recipient by nature, so there is no single canonical body — each variant is a real client contact. Below the distinct recipients are grouped, with near-identical variants merged and their trivial differences noted (phone punctuation, abbreviation of "U.A.E.", title wording, company legal-name length).

---

### Sim Leisure — Josie Booth

> Josie Booth
> General Manager
> Sim Leisure Gulf Contracting LLC
> Dubai, U.A.E.
> Email: josie@simleisure.com
> Phone: +971.50.657.5406

Two variants in use, differing only in phone-number punctuation: `+971.50.657.5406` (dotted) and `+971 50 657 5406` (spaced). Same person, company, and email.

Used by: 23-96602, 22-97113-FP-01, 22-97113-FP-02, 24-97109, 25-97106-FP-01, 25-97106-FP-02, 22-96601, 24-96601, 24-97105, 25-97104 (10 docs)

---

### Conrad Hotels — Nicholas Fernandez

> Nicholas Fernandez
> Director of Engineering
> Conrad Hotels
> Abu Dhabi, U.A.E.
> Email: nicholas.fernandez@conradhotels.com
> Phone: +971 55 500 8914

Used by: 24-97107, 24-97113, 25-97101-FP-01, 25-97101-FP-02, 25-97105 (and 25-97101-FP-01 cross-listed under 25-97105) (6 docs)

---

### Stickman Tribe — Marcos Cain

> Marcos Cain
> Principal and Founder
> Stickman Tribe Ltd.
> Dubai, U.A.E.
> Email: marcos@stickmantribe.com
> Phone: +971 5043 55034

Two variants for the same person: the Ciel projects use company "Stickman Tribe Ltd." with phone `+971 5043 55034`; the Tape Theatre project uses the fuller legal name "Stickman Tribe Turnkey Projects Contracting LLC" with phone `+971.50.435.5034` (dotted). Differences are company legal-name length and phone punctuation only.

Used by: 23-97108-FP-01, 23-97108-FP-02, 23-97109-FP-01, 23-97109-FP-02, 23-97105-FP-01 (+ comments) (6 docs)

---

### Hyperspace — Lizzie Wafer

> Lizzie Wafer
> Producer
> Hyperspace FZ-LLC
> Dubai, U.A.E.
> Email: lizzie@hyper-space.com
> Phone: +971.54.334.0228

Two variants: company "Hyperspace FZ-LLC" with dotted phone `+971.54.334.0228`, versus plain "Hyperspace" with spaced phone `+971 54 334 0228`. Same person and email.

Used by: 24-97104, 23-97110, 24-97101-FP-01, 24-97101-FP-02, 22-97114 (5 docs)

---

### Hyperspace — Carlos Velazquez

> Carlos Velazquez
> Head of Production
> Hyperspace
> Dubai, U.A.E.
> Email: carlos@hyper-space.com
> Phone: +971 50 488 7769

A different Hyperspace contact (Head of Production, not Producer).

Used by: 23-96607 (1 doc)

---

### P&T Group — Emily Mandu

> Emily Mandu
> Bid Coordinator
> P&T Group
> Dubai, U.A.E.
> Email: emily@ptdubai.ae
> Phone: +971 4 358 6803

Phone appears both spaced (`+971 4 358 6803`) and unspaced (`+97143586803`) — same number.

Used by: 24-97108, 24-97111, 25-97108, 25-97109, 24-97110 (5 docs)

---

### P&T Group — Hala Naber

> Hala Naber
> Business Development Manager
> P&T Group
> Dubai, U.A.E.
> Email: hala@ptdubai.ae
> Phone: +971 4 358 6803

Used by: 23-97101, 24-96602, 24-97102 (3 docs)

---

### P&T Group — Raed Al Sayyed

> Raed Al Sayyed
> Senior Associate
> P&T Group
> Dubai, U.A.E.
> Email: raed@ptdubai.ae
> Phone: +971 4 358 6803

Used by: 23-97107 (1 doc)

---

### Martin Professional Middle East — Ahmad Zaiim

> Ahmad Zaiim
> Senior Projects Manager
> Martin Professional Middle East
> Dubai, U.A.E.
> Email: ahmad@martinpro-me.com
> Phone: +971 50 1575180

Used by: 24-96606-FP-01 (+ comments, comments response), 24-96606-FP-02 (4 docs)

---

### KCC Entertainment Design — Louis Martens

> Louis Martens
> Design Department Director
> KCC Entertainment Design
> Wielsbeke, Belgium
> Email: louis@kcc.be
> Phone: +3256439843

Used by: 23-96601, 23-96604, 23-96605 (3 docs)

---

### Wynn (Al Marjan Island) — Derek Sands

> Derek Sands
> Executive Vice President
> Wynn Al Marjan Island FZ-LLC
> Ras Al Khaimah, U.A.E.
> Email: derek.sands@wynndevelopment.com
> Phone: +971 50 692 7859

Two variants for the same person: title/company "Executive Vice President — Wynn Al Marjan Island FZ-LLC" (WAMI projects) versus "Executive Vice President - UAE — Wynn Design and Development" (Wynn project 23-97102), and city spelled "Ras Al Khaimah" vs "Ras al-Khaimah". Same email and phone.

Used by: 25-97102-FP-01 (+ WDD notes), 25-97102-FP-02, 23-97102-AA, 23-97102-EL (4 docs)

---

### Nakheel Malls / Dubai Islands — Ryan Marginson

> Ryan Marginson
> Senior Manager - Technical Production
> Nakheel Malls
> Dubai, U.A.E.
> Email: ryan.marginson@nakheel.com
> Phone: +971 4 3903333

Several near-identical variants for the same person at the same office, differing only in formatting and one differing in company line:
- City punctuation: "Dubai, U.A.E." / "Dubai, U. A .E." / "Dubai, U. A. E."
- Title dash: hyphen "Senior Manager - Technical Production" vs en-dash "Senior Manager — Technical Production"
- Phone `+971 4 3903333` for most; one Dubai Islands variant uses company "Dubai Islands L.L.C", phone `+971 50 8234272`, email cased `Ryan.Marginson@nakheel.com`.

Used by: 22-97111-FP-01 (+ signed), 22-97112-FP-01 (+ signed), 23-97106 (5 docs)

---

### Sanderson International Theme — Meenakshi Chauhan

> Meenakshi Chauhan
> Design manager
> Sanderson International Theme LLC
> Dubai, U.A.E.
> Email: meenakshi.chauhan@sandersonuae.com
> Phone: +971 58 842 9529

Used by: 22-97115-FP-01, 22-97115-FP-02 (2 docs)

---

### Proscape — Rajesh Naidu

> Rajesh Naidu
> Senior Manager - MEP
> Proscape LLC
> Dubai, U.A.E.
> Email: rajeshnaidu@proscapeuae.com
> Phone: +971.55.300.6433

Used by: 23-97111-FP-01, 23-97111-FP-02 (2 docs)

---

### Jouzy — Ramez Bakr

> Ramez Bakr
> Head of Electrical Engineering
> Jouzy
> Dubai, U.A.E.
> Email: r.bakr@jouzy.com
> Phone: +971 50 746 9942

Used by: 25-97103-FP-01, 25-97103-FP-02 (2 docs)

---

### Afniah Architects + Engineers — Amr AlAzzazi

> Amr AlAzzazi
> Design Director
> Afniah Architects and Engineers
> Dammam, K.S.A.
> Email: amr.alazzazi@afniah.com
> Phone: +966 50 386 5692

Used by: 23-96603 (1 doc)

---

### Afniah Architects + Engineers — Abdulsattar Al Hulaimy

> Abdulsattar Al Hulaimy
> Projects Coordinator
> Afniah Architects + Engineers
> Dammam, K.S.A.
> Email: a.sattar@afniah.com
> Phone: +966 55 109 3214

Same firm as above ("Afniah Architects and Engineers" / "Afniah Architects + Engineers"), different contact.

Used by: 24-96603 (1 doc)

---

### RAK Leisure — Quinton Symons

> Quinton Symons
> Procurement Manager
> RAK Leisure LLC
> Ras Al Khaimah, U.A.E.
> Email: quinton@rakhospitality.com
> Phone: +971 5653 98500

Used by: 24-97106 (1 doc)

---

### Tabanlioglu Architects — Umar Sikandry

> Umar Sikandry
> TBC
> Tabanlioglu Architects
> Dubai, U.A.E.
> Email: alhan@tabanlioglu.com
> Phone: +971 4 392 7 666

Note: position is a placeholder ("TBC") and the email handle (`alhan@`) does not match the named contact — likely incomplete data.

Used by: 24-97112 (1 doc)

---

### Tait — Andrew Hawkes

> Andrew Hawkes
> Project Director
> Tait
> Dubai, U.A.E.
> Email: andrew.hawkes@taittowers.com
> Phone: +971 521 918 532

Used by: 25-96601 (1 doc)

---

### Imtiaz Developments — Irene Nale

> Irene Nale
> Design Administration Coordinator
> Imtiaz Developments
> Dubai, U.A.E.
> Email: i.nale@imtiaz.ae
> Phone: +971 4 430 3703

Used by: 25-97107 (1 doc)

---

### Mojo Architecture & Interior Design — Martin Meijer

> Martin Meijer
> Design Director
> Mojo Architecture & Interior Design
> Dubai, UAE
> Email: martin@mojo.ae
> Phone: +971 50 903 9345

Used by: 26-96801 (1 doc)

---

### U+A — Eugene De-Villiers

> Eugene De-Villiers
> Director - Head of Projects KSA
> U+A
> Dubai, U.A.E.
> Email: eugene.de-villiers@ua-intl.co.uk
> Phone: +971 50 466 0851

Used by: 24-96605 (1 doc)

---

### Template placeholder (not a real recipient)

> Contact Name
> Contact Position
> Client Company
> Client City, U.A.E.
> Email: client@contact.email
> Phone: +971 contact phone

This is the unfilled template block, left in one proposal (project number also a placeholder `e-yy-cccnn-FP-01`). It defines the canonical field layout.

Used by: 23-96606 (1 doc)

---

**Recommended standard wording:**

There is no single content-canonical "Prepared For" clause — each is a genuine client recipient. What can be standardised is the field layout and formatting, derived from the template block above:

```
{Contact Name}
{Contact Position}
{Client Company}
{Client City}, {Country}
Email: {email}
Phone: {phone}
```

Formatting conventions to enforce for consistency (the variants above drift on all of these):
- Country: write "U.A.E." (periods, uppercase) and "K.S.A." — not "UAE".
- City: no internal spacing in the abbreviation — "Dubai, U.A.E.", never "Dubai, U. A. E."
- Title separator: hyphen with spaces ("Senior Manager - Technical Production"), never an en-dash or em-dash.
- Phone: single consistent style — spaced groups with a leading `+` country code (e.g. `+971 50 657 5406`), not dotted (`+971.50.657.5406`) and not run-together (`+97143586803`).
- Use the full registered company legal name where known, consistently across that client's proposals.

---

## Prepared By

This clause identifies the author of the proposal and provides their contact block. Three families appear in the corpus: a Martin Robert (lighting) signature, an Andrew Hawkes (sound/audio) signature, and a longer "about emittiv" consultancy description. Within the signature families, the variants differ only in entity name (`emittiv llc-fz` vs `emittiv design`/`emittive design`), title wording, and phone formatting.

---

### 1. Martin Robert signature (lighting) — dominant variant

The overwhelmingly common form. Used in 58 documents (and a near-identical group below brings the family to 63).

> Martin Robert
> Founder and Lighting Director
> emittiv llc-fz
> Dubai, UAE
> Email: martin@emittiv.com
> Phone: +971.5858.555.69

Used by: 22-97114 HoH, 22-97115 Kids Activity Park AUH (FP-01/02), 23-96601 Dammam Adv World, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-96604 DockX, 23-96605 Jumpoline, 23-96607 HOH KSA, 23-97101 Marina Island, 23-97102 Wynn (AA/EL), 23-97106 Fountain Control, +46 more

### 1b. Same signature, minor wording/entity differences

These are the same Martin Robert lighting signature with trivial differences — folded into family 1. Differences noted per group:

- **`emittive design` (typo in entity name), 2 docs** — 22-97113 MAF FEC (FP-01/02). Identical except entity reads "emittive design" instead of "emittiv llc-fz".
- **`emittiv design` entity, 1 doc** — 22-96601 Dammam Waterpark. Entity reads "emittiv design".
- **"Lighting Director" (no "Founder and"), spaced phone, 1 doc** — 26-96801 Rozana Muscat. Title drops "Founder and"; phone formatted `+971 5858 555 69`.
- **With appended Document Control block, 1 doc** — 24-96605 Observatory. Same signature plus a trailing "Document Control" line (Date: 240911 | Release: 01 | Author: MR | Reference: RFP | Distribution...). The Document Control content is project-specific metadata, not part of the standard signature.

---

### 2. Andrew Hawkes signature (sound/audio)

Used on sound and audio-discipline proposals. Two close variants differ in title ("Sound Director" vs "Audio Director"), entity presence, and phone formatting.

**2a. Sound Director (no entity line), 4 docs:**

> Andrew Hawkes
> Founder and Sound Director
> Email: andrew@emittiv.com
> Phone: +971 504 296 588

Used by: 22-97111 TPF Sound System (signed + unsigned), 22-97112 The View Exp Ctr (signed + unsigned)

**2b. Audio Director (full block), 2 docs:**

> Andrew Hawkes
> Founder and Audio Director
> emittiv llc-fz
> Dubai, UAE
> Email: andrew@emittiv.com
> Phone: +971.50.429.6588

Used by: 23-97105 Tape Theatre (FP-01, FP-01-AH comments)

Note: both phone forms are the same number (+971 50 429 6588), formatted differently.

---

### 3. "About emittiv" consultancy description

A long-form paragraph describing the practice, used in place of (or alongside) a personal signature. Two variants: full version with website line, and a shortened version missing the final website sentence.

**3a. Full version, 3 docs:**

> emittiv is a multidisciplinary sensory design consultancy working with lighting, video, sound, scent, and control systems for the built environment. We design experiences, not just systems - making sure all of our elements work together to deliver a unified and polished concept.
> We work closely with Architects, Interior Designers, Lead Design Consultants and Owners / Operators on projects worldwide.
> Our aim is to enhance people's everyday lives by adding layers that make our spaces more interesting and beautiful.
> Our creative and commercial experience enables us to develop intelligent and appropriate designs within the parameters of budgets and time frames.
> We are entirely independent of any manufacturers or distributors, and operate on a fee for service basis.
>
> Our combined experience covers all types and scale of projects including: Attractions, Bars, Brand Activations, Casual Dining Venues, Cinemas, Corporate Headquarters, Department Stores, Destination Locations, Family Entertainment Centres, Galleries, Gaming Venues, Hotels, Landmark Buildings, Landscape, Malls, Museums, Nightclubs, Offices, Parks, Performance Venues, Public Realm, Residential, Retail Flagships, Shopping Centres, Spas, Theatres, Theme Parks.
>
> We can be appointed in a number of ways and tailor our services to a specific project and Clients' needs. All project teams are overseen by a Director and/or Associate to ensure a high level of service at all times. For further information on our services and project experience please visit our website: www.emittiv.com

Used by: 23-96601 Dammam Adv World, 25-97102 WAMI, 25-97105 Shanghai Tang v2

**3b. Shortened version (drops the closing website sentence), 1 doc:** 24-96606 JOH (FP-01 comments). Identical to 3a except the final sentence ends at "...high level of service at all times." with no website reference.

---

**Recommended standard wording:**

For the author signature, use the dominant Martin Robert lighting block verbatim (correcting the entity name to the canonical `emittiv llc-fz`, dropping the "emittive"/"emittiv design" typos):

> Martin Robert
> Founder and Lighting Director
> emittiv llc-fz
> Dubai, UAE
> Email: martin@emittiv.com
> Phone: +971.5858.555.69

For sound/audio proposals, use the parallel Andrew Hawkes block with the full entity lines and consistent title ("Audio Director" per the most complete variant 2b):

> Andrew Hawkes
> Founder and Audio Director
> emittiv llc-fz
> Dubai, UAE
> Email: andrew@emittiv.com
> Phone: +971.50.429.6588

When a practice description is wanted, use the full "About emittiv" paragraph (3a) including the closing website line. Keep phone formatting consistent (dotted form `+971.5858.555.69` / `+971.50.429.6588`) and the entity name as `emittiv llc-fz` across all variants.

---

## Scope & Services

The "Scope & Services" clause type spans four recurring sub-clauses across the proposal corpus: (A) the **RFP acknowledgement / appointment confirmation** opener, (B) the **project understanding** block (project name, location, plot area, packages, reference documents), (C) the **areas included / excluded** scope block, and (D) the **stages / services division** statement. Many proposals combine several of these into one body; the variants below are grouped by their dominant purpose. Stage-deliverable narratives (Concept, Schematic, Detailed Design, Tender, Post-Tender) also appear under this clause type and are catalogued at the end.

---

### A. Appointment confirmation (RFP acknowledgement opener)

**A1 - Standard appointment confirmation (canonical, client-named).** The most common opener: confirms receipt of the RFP, willingness to accept, and qualification. The client name varies; reference-document lists and project-detail blocks are frequently appended.

> emittiv ["Consultant", "We"] have received a Request for Proposal from Nakheel Malls ["Client"].
>
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
>
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.

Used by: 22-97111 TPF Sound System (signed + FP-01), 22-97112 The View Exp Ctr. The same three-sentence core (with varying client name, reference-document list, validity clause, project details, packages and areas appended) recurs in 22-97115, 23-96601 through 23-96607, 23-97102, 23-97106, 23-97108, 23-97109, 23-97111, 24-96601, 24-96602, 24-96606, 24-97101, 24-97104, 24-97105, 24-97106, 24-97109, 24-97110, 24-97111, 24-97112, 25-96601, 25-97101 through 25-97109, 26-96801 +30 more. Trivial differences: bullet style (• vs -), line-break vs paragraph spacing, and presence/absence of the "Reference documents provided by the Client…" sentence.

---

### B. Project understanding & packages

**B1 - Project details + packages (no opener).** Standalone project-understanding block: project name, location, development description (plot area / floors), then the technologies/packages list. Appears on its own when the appointment opener is a separate clause.

> The Project is known as Kids Activity Park and is located in Abu Dhabi, U.A.E.
> The Project is a proposed new development and currently comprises of:
> - Total Plot Area: Approximately 3181 m2
>
> Packages
> This Proposal includes the following technologies [referred to as packages or systems]:
> - Lighting
> - Video
> - Sound
> - SFX
> - Control Systems

Used by: 22-97115 Kids Activity Park, 23-96604 DockX, 24-96602 Intercontinental Jeddah, 26-96801 Rozana Muscat, 23-96606 KSA Pav Osaka (template placeholder form), 24-96603 Marasi Gate, 24-96602 (packages-only sub-block). Trivial differences: which detail rows are present (plot area, GFA, floors, height), and bullet style.

**B2 - Packages list only.** A bare technologies/packages enumeration, used where project details sit in a separate clause.

> This Proposal includes the following technologies [referred to as packages or systems]:
> - Lighting
> - Video
> - Sound
> - Acoustics
> - Technologies [control systems and SFX as required]

Used by: 23-96604 DockX, 24-96602 Intercontinental Jeddah, 24-97102 DMC 2A, 24-97113 Level 63, 25-97105 Shanghai Tang v2, 23-96606 KSA Pav Osaka. Note: several of these append "An optional price for Acoustics Design and consultancy is also included." (24-97113, 25-97105).

---

### C. Areas included / excluded (scope boundaries)

**C1 - Lighting public-realm scope, standard inclusions/exclusions (canonical).** The most reused areas block for lighting-led projects. Inclusions cover public realm, interior, facade, landscape and control systems; exclusions cover BoH, shell-and-core, and statutory-approval street lighting.

> The proposed scope includes the following areas:
>
> - Lighting:
>   - Public Realm Lighting
>   - Interior Architectural Lighting
>   - Facade Lighting
>   - Landscape / Water Feature Lighting
>   - Lighting Control Systems
>
> The proposed scope excludes the following areas:
> - Lighting:
>   - BoH Areas
>   - Shell and Core tenanted spaces
>   - Street Lighting requiring a Statutory Authority's approval

Used by: 24-96602 Intercontinental Jeddah, 24-97102 DMC 2A, 23-96603 Khobar Grand Mosque, 23-96606 KSA Pav Osaka, 24-97111 Masdar OS-48, 24-97110 Masdar B-02, 25-97103 Aljada FG, 25-97109 Mapletree, 26-96801 Rozana Muscat +3 more. Trivial differences: bullet glyph (•, -, ‐), whether "Parking Areas"/"Entertainment Lighting Infrastructure" is added to inclusions, and whether "Emergency Lighting Systems" / "Daylight Calculations" appear in exclusions. Sub-variants worth noting: 24-96603 adds "Parking Areas" and an interior-limited note; 25-97109 adds "Daylight Calculations"; 26-96801 swaps inclusion order (Interior before Public Realm) and adds Emergency Lighting to exclusions.

**C2 - Interior-only lighting + control (with optional audio).** Tighter interior-fitout scope, common to hospitality / FEC / experiential venues. Often paired with an Audio sub-list.

> The proposed scope includes the following areas:
>
> • Lighting:
>   - Interior Architectural Lighting
>   - Lighting Control Systems
> • Audio:
>   - Ambient Audio Systems
>   - Audio Control Systems
>
> The proposed scope excludes the following areas:
>
> • Lighting:
>   - Emergency Lighting
>   - BoH Areas
> • Audio:
>   - PAVA / Emergency systems

Used by: 24-97107 Reserve Cut, 24-97113 Level 63, 25-97101 Shanghai Tang, 25-97105 Shanghai Tang v2. Lighting-only relatives (no audio): 22-97114 HoH, 24-97101/24-97104 HoH Dubai (these add "IT Network Configuration" to exclusions and use "Emergency Lighting Systems"), 24-97105 MAF MiCC, 25-97104 MAF MiCC Redesign (these add "Shell and Core tenanted spaces").

**C3 - All-systems experiential venue scope (FoH guest areas).** Used for FEC / themed-attraction venues where one inclusion ("All FoH Guest areas" or "All Systems: Public areas") covers every package, with BoH and life-safety excluded.

> The proposed scope includes the following areas:
> - All FoH / Guest Areas
> - Area Development spaces
>
> The proposed scope excludes the following areas:
> - Attractions and games
> - Emergency and Life Safety Systems

Used by: 23-96604 DockX, 23-96605 Jumpoline (adds BoH Areas), 23-96607 HoH KSA (FoH guest areas; excludes BoH, PAVA, Emergency Lighting, Digital Signage), 23-97108 / 23-97109 Ciel (all-systems public lobby / rooms-and-suites). Trivial differences: which specific exclusions are appended.

**C4 - Multi-package (Lighting + Sound) area scope.** Lighting and Sound enumerated separately on both inclusion and exclusion sides; used on larger entertainment/leisure developments.

> The proposed scope includes the following areas:
>
> • Lighting:
>   - Interior Architectural Lighting
>   - Facade Lighting
>   - Lighting Control Systems
> • Sound:
>   - General BGM playback / paging system
>   - Localised Immersive Speaker Locations as required
>   - Audio Control Systems
>
> The proposed scope excludes the following areas:
> • Lighting:
>   - Emergency Lighting Systems
>   - BoH Areas
>   - Shell and Core tenanted spaces
>   - Car Park area Lighting
>   - Street Lighting requiring a Statutory Authority's approval
> • Sound:
>   - Emergency / Alarm / PAVA System Integration
>   - BoH Areas
>   - Shell and Core tenanted spaces
>   - Acoustic Calculations
>   - Exterior Areas

Used by: 23-96601 Dammam Adv World. Related multi-package area blocks: 23-97111 Dubai Island Promenade (Audio + Lighting, promenade-specific), 23-97102 Wynn (Acoustics + AV, resort-specific key-activities form).

**C5 - Theatrical lighting scope.** Specialised inclusion/exclusion for theatre venues.

> The proposed scope includes the following areas:
> - Theatrical Lighting Systems
> - Theatrical Lighting Control Systems
>
> The proposed scope excludes the following areas:
> - FoH areas outside theatre spaces
> - BoH areas outside theatre spaces
> - Lighting Systems provided by Others [co-ordination is included as required]

Used by: 24-96606 JOH (FP-01, FP-02, comments, comments-response - identical core, bullet glyph only difference).

---

### D. Stages / services division statement

**D1 - "Divided into Stages 1-N" services statement (canonical).** The standard sentence introducing the staged services breakdown. Only the stage count (N) and an occasional trailing qualifier vary.

> This proposal includes the services described below and is divided into Stages 1-3.
> Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
> The deliverables outlined in this document are typical for a wide range of projects.
> Specific Deliverables will be agreed with the Client during Preliminaries.

Used by: 22-97115, 23-96604, 24-96601, 26-96801, 25-97106, 23-97110 (Stages 1-3); 23-97105, 24-97107, 25-97108, 24-97113 (Stages 1-4); 24-97102, 24-97110 (Stages 1-5); 23-97111 (Stages 1-6); 23-96606, 23-97101 (Stages 1-7); 22-97114, 23-96607, 24-97101 (Stages 1-8); 23-97109 (Stages 1-2); 25-97103 (Stages 2-6) +more. Trivial differences: the stage range (`1-3` … `1-8`, `2-6`), one/multi-line formatting, and trailing qualifiers - e.g. "typical for similar projects of this type and scale… within the time frames allowed for in this proposal" (22-97114, 24-97101); "…based on the relevant package items listed in Volume 4" (22-96601); a stray "xxx" placeholder (23-96606 template). 23-96607 appends two Revit / Design-and-Build acknowledgement sentences. 25-97102 uses a shortened form ("Specific Deliverables will be reviewed and agreed with the Client during each stage").

---

### E. Stage-deliverable narratives

These recur verbatim (bullet glyph aside) and describe individual design-stage deliverables. Catalogued together because they are templated and reused across lighting/audio proposals.

**E1 - Concept Design Report.** "We will prepare a [discipline] Concept Design Report… guidelines for the Client and other Designers… We have allowed for up to one revision of the Concept Design Report." Discipline name is interpolated (Specialist Lighting, Audio System, External Lighting, Specialist Lighting (Façade)).

> We will prepare a Specialist Lighting Concept Design Report for the Project which will provide guidelines for the Client and other Designers to ensure that the Specialist Lighting concepts are in line with the overall Project aspirations. This report shall be based on coordination with other Designers / Consultants as well as requirements of the Client.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings
> • Briefing and presentation meetings with the Client
> • Carrying out conceptual design studies
> • Undertake research to establish current / future trends and establish design baselines
> • Provide recommendations on package requirements [such as lux levels, uniformities, resolution, sound levels, and intelligibility as required]
> • High level Package Master Plans
> • Produce presentation materials to convey the overall design intention
>
> We have allowed for up to one revision of the Concept Design Report.

Used by: 24-96603 Marasi Gate, 25-97107 Cove Boulevard, 25-97108 RAK Beach District, 22-97111 TPF (Audio variant - adds a "Concept Design Report:" sub-list and budget-outline line).

**E2 - Schematic Design (preliminary development).** "In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase… up to one revision of the Schematic Design Report."

> In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings
> • Carry out further development of package design
> • Carry out preliminary development of package specific 3D layouts based on linked files to LOD 200 standards
> • Prepare preliminary 3D package calculations [as required]
> • Prepare preliminary package layouts
> • Prepare preliminary package detail sketches
> • Prepare preliminary package equipment schedule
> • Prepare preliminary package control intent
> • Prepare preliminary logical control channel schedule
> • Prepare preliminary package scene information
> • Prepare preliminary package power requirements
> • Assist others in checking that the package schemes are within the approved budgets
> • Identify integration points with other systems as required
>
> We have allowed for up to one revision of the Schematic Design Report.

Used by: 24-96603 Marasi Gate, 25-97108 RAK Beach District, 25-97107 Cove Boulevard (2D-layouts variant), 22-97111 TPF (shorter list, no LOD/scene/channel lines). Trivial differences: LOD 200 vs 2D-layout wording, presence of the scene/channel-schedule bullets.

**E3 - Detailed Design ("Basis of Design").** "In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender… from a 'Basis of Design' perspective."

> In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender and finalised by the awarded contractor for construction from a "Basis of Design" perspective.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings / mock-ups
> - Carry out further development of package specific 3D layouts based on linked files to LOD 300 standards
> - Prepare final package layouts
> - Assist others with the coordination of the package design
> - Prepare final package equipment schedule
> - Prepare final package details where applicable for incorporation into Architect's details
> - Prepare final 3D package calculations
> - Prepare final package power requirements
> - Specify integration points with other systems as required

Used by: 24-96603 Marasi Gate, 25-97107 Cove Boulevard, 25-97108 RAK Beach District (appends the four "Open Tender" calc/spec/costing lines), 22-97111 TPF (shorter, no LOD line). Trivial differences: LOD 300 wording, the appended tender-documentation lines on 25-97108.

**E4 - Tender Documentation ("Open Tender").** "This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from an 'Open Tender' perspective."

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.
>
> Typical deliverables include:
> • Following approval provide documents for final tender and assistance in co-ordinating system design with architectural, electrical, structural and mechanical design.
> • Final system calculations and compliance with relevant Building Regulations.
> • Final specification and schedule of devices, accessories and associated equipment.
> • The above information will allow detailed costing by others - the Project's Cost Estimator / Quantity Surveyor / Tendering Contractor / other.
> • Final specification for the operational requirements of the control system and control schedule.

Used by: 24-96603 Marasi Gate, 25-97107 Cove Boulevard, 25-97108 RAK Beach District (abbreviated to the first bullet only).

**E5 - Post-Tender review / submittal scoring.** Review and analysis of substitute devices and tender submittals.

> Typical deliverables include:
> - Review and checking of proposed substitute devices, as selected by others, for compliance with specification.
> - Review and analyse of submittals based on technical compliance, company experience and other factors as required.
> - Prepare scoring matrix comparing submitted package tenders.

Used by: 25-97107 Cove Boulevard.

**E6 - Pre-design / project initiation.** Information-gathering deliverables plus the "emittiv requires the following information in order to commence" list.

> Typical deliverables include:
> - Assistance in defining project aspirations
> - Assistance in project planning and brief development as appropriate
> - Review of site and project information
>
> emittiv requires the following information in order to commence:
> - Client's rules and regulations applying to the site and its application
> - Operational and functional requirements
> - Design criteria and standards
> - Program and Area allocations
> - Any established package budget
> - Existing project drawings, sketches, models and reports

Used by: 22-97111 TPF Sound System (signed).

**E7 - Sitewide review / lighting-coordination scope (Wynn-specific).** Bespoke deliverables for a lighting-coordination/review appointment (review existing designs, control-system mapping, scene coordination). Highly project-specific - not a reusable template.

> Specified deliverables include:
> - Review all lighting designs (drawings, specifications, reports) within the Wynn resort building, external areas and Island 3 infrastructure and identify any additional information required.
> - Identify the lighting control systems specified for each area and suggest appropriate technologies where not specified, and advise suitability for the application.
> - Identify where lighting scenes have been specified and highlight relevant areas where scenes have not been specified.
> - Identify locations of lighting control panels, centralised workstation and connectivity requirements.
> - Prepare and present initial report.

Used by: 25-97102 WAMI (FP-01, plus a Schematic-phase sitewide-mapping continuation).

---

**Recommended standard wording:**

For a typical lighting-led proposal, the canonical Scope & Services clause set is:

1. **Appointment confirmation (A1):**
   > emittiv ["Consultant", "We"] have received a Request for Proposal from [Client] ["Client"].
   >
   > emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
   >
   > emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.

2. **Areas included / excluded (C1):**
   > The proposed scope includes the following areas:
   > - Lighting:
   >   - Public Realm Lighting
   >   - Interior Architectural Lighting
   >   - Facade Lighting
   >   - Landscape / Water Feature Lighting
   >   - Lighting Control Systems
   >
   > The proposed scope excludes the following areas:
   > - Lighting:
   >   - BoH Areas
   >   - Shell and Core tenanted spaces
   >   - Street Lighting requiring a Statutory Authority's approval

3. **Services division (D1):**
   > This proposal includes the services described below and is divided into Stages 1-[N].
   > Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
   > The deliverables outlined in this document are typical for a wide range of projects.
   > Specific Deliverables will be agreed with the Client during Preliminaries.

Standardisation notes: pick one bullet glyph (recommend `-` for consistency); fix the grammatical slips that recur verbatim ("from a 'Open Tender'" → "from an 'Open Tender'"; "Review and analyse of submittals" → "Review and analysis of submittals"). Stage-deliverable narratives (E1-E5) are stable enough to template with the discipline name and LOD level as the only interpolated fields.

---

## Stages

Stage clauses fall into two families: **stage-list overviews** (the "This proposal includes the following project stages:" preamble that enumerates the stages, often with scope inclusions/exclusions) and **individual stage deliverables** (the per-stage "Typical deliverables include:" body for Concept, Schematic, Detailed, Tender, Construction, etc.). Bullet style (`•`) and dash style (`-`) of the same clause are treated as one variant where wording is otherwise identical; the difference is purely a markup convention and is noted where relevant.

---

### Stage-list overviews

#### Stage list — Concept / Schematic / Detailed / Tender (4-stage Design Phase, most common)

The most frequent design-phase enumeration. Many trivial variants exist differing only in: bullet vs dash markup, em-dash vs hyphen vs colon stage labels, presence/absence of a "Preliminaries" line, and "Schematic"/"Developed"/"Technical" naming for stage 2/3. Representative:

> This proposal includes the following project stages:
> • Design Phase
>   - Preliminaries
>   - Stage 1   Concept Design
>   - Stage 2   Schematic Design
>   - Stage 3   Detailed Design
>   - Stage 4   Tender Documentation

Naming and structure vary by project: "Developed Design" / "Technical Design" replace "Schematic" / "Detailed" (Tape Theatre, Reserve Cut, Shanghai Tang); "Pre-Concept" prepends the list (DMC 2A); some add Stage 4a/5 "Tender Return Review" / "IFC Documentation" (Cove Boulevard, Military Museum, Aljada); some drop Preliminaries entirely.

Used by: 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-96604 DockX, 24-96602, 24-96603 Marasi Gate, 24-97102 DMC 2A, 24-97107 Reserve Cut, 25-97101 Shanghai Tang, 25-97105 Shanghai Tang v2, 25-97107 Cove Boulevard, 25-97108 RAK Beach District, 26-96801 Rozana Muscat, 23-97105 Tape Theatre, +8 more

#### Stage list — Concept / Detailed / IFT / IFC / Procurement (FEC-style)

> This proposal includes the following project stages:
>
> Design Phase
> • Stage 1 — Concept Design
> • Stage 2 — Detailed Design
> • Stage 3 — IFT Documents
> • Stage 4 — IFC Documents
> • Stage 5 — Procurement
> ...

Used by: 22-97113 MAF FEC, 25-97106 MAF MOE FEC v3 (Concept Design Review variant), 24-97109 MOE FEC v2

#### Stage list — Design + Post Contract Phase (full lifecycle)

Adds a Post Contract Phase (Construction Supervision, Focussing/Aiming and Scene Setting, Commissioning and Handover, Post Completion / DLP) after the design stages.

> This proposal includes the following project stages:
> • Design Phase
>   - Preliminaries
>   - Stage 1   Concept Design
>   - Stage 2   Schematic Design
>   - Stage 3   Detailed Design
>   - Stage 4   Tender Documentation
>   - Stage 5   Tender Return Review
> • Post Contract Phase
>   - Stage 6   Construction Supervision
>   - Stage 6a  Focussing / Aiming and Scene Setting
>   - Stage 6b  Commissioning and Handover
>   - Stage 7   Post Completion / DLP

Used by: 23-97101 Marina Island Financial District, 23-96607 HOH KSA, 23-97111 Dubai Island Promenade (FP-01/FP-02), 24-97106 RAK Sled, 24-97110 Masdar B-02, 25-96601, 25-97103 Aljada FG, 23-96606 KSA Pav Osaka

#### Stage list — Post Design / Supervision only (no design phase)

Supervision-only engagements beginning at tender return.

> This proposal includes the following project stages:
> • Post Design Phase
>   - Stage 1: Tender Return Review
>   - Stage 2: Contractor / Procurement Management
>   - Stage 3: Shop Drawing Review
>   - Stage 4: Construction Supervision
>   - Stage 5: Commissioning Validation
>   - Stage 6: Programming Supervision
>   - Stage 7: Handover / Training
>   - Stage 8: Post Completion / DLP [Optional]

Used by: 22-97114 HoH, 24-97101 HoH Supervision (FP-01/FP-02), 24-97104 HoH Programming (2-stage off-site/on-site variant)

#### Stage list — Review & Report / Ongoing Consultancy (WAMI-style audit)

> This proposal includes the following project stages:
> - Review and Report Phase
>   - Stage 1: Information Gathering and Initial Report
>   - Stage 2: Report and Recommendations
> - Ongoing Consultancy
>   - Stage 3: On-going Support and Validation
> ...

Used by: 25-97102 WAMI (FP-01, FP-02, WDD Lighting notes)

#### Stage list — System Tuning (2-stage audio)

> This proposal includes the following project stages:
>
> System Tuning
> - Stage 1: Review & Report of current sound system
> - Stage 2: Calibrate, EQ, setup the audio experiential experience

Used by: 22-97112 The View Exp Ctr

#### Stage list — generic "divided into Stages 1-N" preamble (no enumeration)

> This proposal includes the services described below and is divided into Stages 1-4.
> Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
> The deliverables outlined in this document are typical for a wide range of projects.
> Specific Deliverables will be agreed with the Client during Preliminaries.

Used by: 23-96605 Jumpoline (Stages 1-3), 24-96605 Observatory (Stages 1-4), 23-96601 Dammam Adv World (Stage 1 only, with inline SD body)

#### Stage list — single-stage / partial-scope variants

Short bespoke lists: single Detailed Design stage (24-97105 MAF MiCC Kids), Detailed + Tender (25-97105 Shanghai Tang v2 FP-01), Detailed + Issue for Tender (25-97104 MAF MiCC Kids Redesign), 50% Concept/Schematic variants (22-97115 Kids Activity Park), Concept Design Review + Detailed + Tender (24-97109 MOE FEC v2, 25-97106 MAF MOE FEC v3).

Used by: 24-97105, 25-97104, 25-97105, 22-97115, 24-97109, 24-97113 Level 63, 25-97106

---

### Individual stage deliverables

#### Preliminaries / Preparation and Brief Development

> Typical deliverables include:
> • Assistance in defining project aspirations
> • Assistance in project planning and brief development as appropriate
> • Review of site and project information
>
> emittiv requires the following information in order to commence:
> • Any particular requirements or aspirations
> • Any rules and regulations applying to the site
> • Design criteria and standards
> • Operational and functional requirements
> • Program and Area allocations
> • Existing project drawings, sketches, models and reports
> • Any established package budgets

(The TPF variant lists slightly different "requires" items — Client's rules and regulations, operational/functional requirements, design criteria, program/area, package budget, existing drawings. The HOH KSA variant appends a note about a tight programme to opening and streamlined deliverables.)

Used by: 24-97102 DMC 2A, 25-97103 Aljada Block F + G, 22-97111 TPF, 23-96607 HOH KSA

#### Concept Design Report

The most varied clause family — the package descriptor changes per discipline ("Specialist Lighting", "Lighting and Audio", "Audio", "Sensory Design", "Sensory Systems", "Control System", "Specialist Sensory Systems", "Specialist Video"), but the deliverables list is otherwise stable. Representative (Specialist Lighting, most common):

> We will prepare a Specialist Lighting Concept Design Report for the Project which will provide guidelines for the Client and other Designers to ensure that the Specialist Lighting concepts are in line with the overall Project aspirations. This report shall be based on coordination with other Designers / Consultants as well as requirements of the Client.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings
> • Briefing and presentation meetings with the Client
> • Carrying out conceptual design studies
> • Undertake research to establish current / future trends and establish design baselines
> • Provide recommendations on package requirements [such as lux levels, uniformities, resolution, sound levels, and intelligibility as required]
> • High level Package Master Plans
> • Produce presentation materials to convey the overall design intention
>
> We have allowed for up to one revision of the Concept Design Report.

Discipline-specific recommendation lines differ: audio versions use "[such as sound levels, intelligibility, and spatialisation as required]" and may add a "Content recommendations" bullet (TPF, Tape Theatre). Some prepend a review of existing site/concept (Sensory Design "shall review the existing concept design"; Rozana adds "Undertake a review of the existing site conditions and lighting devices"). The 23-97110 HoH variant ("for specific new areas") drops the revision line.

Used by: 23-97102 Wynn (AA & EL), 24-97102 DMC 2A, 24-97106 RAK Sled, 24-97108 Tamani Hotel, 24-97110 Masdar B-02, 24-97111 Masdar OS-48, 25-97103 Aljada, 25-97109 Mapletree, 24-97107 Reserve Cut, 25-97105 Shanghai Tang v2, 25-97106 MAF MOE FEC v3, +6 more

#### Schematic / Developed Design Report

Standard "developed from the Concept Design phase" body. Two sub-variants by deliverable depth: the **full** list (includes "package specific 3D layouts to LOD 200", "control intent", "logical control channel schedule", "scene information") and a **reduced** list that omits 3D layouts and/or scene information. Representative (full):

> In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings
> • Carry out further development of package design
> • Carry out preliminary development of package specific 3D layouts based on linked files to LOD 200 standards
> • Prepare preliminary 3D package calculations [as required]
> • Prepare preliminary package layouts
> • Prepare preliminary package detail sketches
> • Prepare preliminary package equipment schedule
> • Prepare preliminary package control intent
> • Prepare preliminary logical control channel schedule
> • Prepare preliminary package scene information
> • Prepare preliminary package power requirements
> • Assist others in checking that the package schemes are within the approved budgets
> • Identify integration points with other systems as required
>
> We have allowed for up to one revision of the Schematic Design Report.

The 23-97110 HoH variant notes "compressed time frames ... will prevent a formal SD package"; the 23-96607 HOH KSA variant ("locate key equipment in the working model") recommends excluding LED tape from Revit. Bullet and dash forms are otherwise identical.

Used by: 23-97102 Wynn (AA & EL), 24-97102 DMC 2A, 24-97108 Tamani Hotel, 24-97110 Masdar B-02, 24-97106 RAK Sled, 24-97107 Reserve Cut, 24-97111 Masdar OS-48, 24-97112 Military Museum, 25-97103 Aljada, 25-97109 Mapletree, 23-96606 KSA Pav Osaka, +10 more

#### Detailed / Developed Design ("Basis of Design")

Standard "issued for tender ... from a Basis of Design perspective" body. Sub-variants differ by LOD (200 / 300 / 200-300 TBC), whether 2D vs 3D layouts, and "and their integration with other design disciplines" phrasing (Reserve Cut / Shanghai Tang). Representative (LOD 300, most common):

> In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender and finalised by the awarded contractor for construction from a "Basis of Design" perspective.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings / mock-ups
> • Carry out further development of package specific 3D layouts based on linked files to LOD 300 standards
> • Prepare final package layouts
> • Assist others with the coordination of the package design
> • Prepare final package equipment schedule
> • Prepare final package details where applicable for incorporation into Architect's details
> • Prepare final 3D package calculations
> • Prepare final package power requirements
> • Specify integration points with other systems as required

Some projects (Masdar OS-48, Mapletree, Wynn AA) **merge** the Tender-Documentation deliverables into this stage (final system calculations, specification/schedule, costing-by-others, submittal review, scoring matrix). The 25-97106/24-97109 variant uses "2D layouts based on linked files".

Used by: 24-97102 DMC 2A, 24-97108 Tamani Hotel, 24-97110 Masdar B-02, 23-96606 KSA Pav Osaka, 24-96605 Observatory, 23-97102 Wynn (AA & EL), 24-96601 Dammam Water Park, 24-97112 Military Museum, 25-97103 Aljada, 25-97106 MAF MOE FEC v3, 24-97109 MOE FEC v2, +6 more

#### Tender Documentation ("Open Tender")

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.
>
> Typical deliverables include:
> • Following approval provide documents for final tender and assistance in co-ordinating system design with architectural, electrical, structural and mechanical design.
> • Final system calculations and compliance with relevant Building Regulations.
> • Final specification and schedule of devices, accessories and associated equipment.
> • The above information will allow detailed costing by others - the Project's Cost Estimator / Quantity Surveyor / Tendering Contractor / other.
> • Final specification for the operational requirements of the control system and control schedule.

Variants: some drop the "Open Tender" qualifier (Reserve Cut, Shanghai Tang — "...for construction."); the IFC variant (Masdar OS-48) reads "issued for construction to the awarded Contractor"; Tape Theatre / Wynn EL / Masdar B-02 / Aljada append the Tender-Return-Evaluation deliverables (substitute device review, submittal analysis, scoring matrix). Em-dash/hyphen differences in "by others -/–" are trivial.

Used by: 23-97107 DAFZA Refurb, 24-97102 DMC 2A, 24-97106 RAK Sled, 24-97108 Tamani Hotel, 25-97106 MAF MOE FEC v3, 23-96606 KSA Pav Osaka, 23-97106 Fountain Control, 24-97112 Military Museum, 24-96601 Dammam Water Park, 24-97110 Masdar B-02, 24-97111 Masdar OS-48, +6 more

#### Tender Return Review / Evaluation

> Typical deliverables include:
> • Review and checking of proposed substitute devices, as selected by others, for compliance with specification.
> • Review and analyse of submittals based on technical compliance, company experience and other factors as required.
> • Prepare scoring matrix comparing submitted package tenders.

(The HOH KSA variant replaces the scoring-matrix bullet with "Initial reviews and comments on Shop Drawings developed by the awarded contractor.")

Used by: 23-97107 DAFZA Refurb, 24-97102 DMC 2A, 24-97106 RAK Sled, 24-97108 Tamani Hotel, 23-96606 KSA Pav Osaka, 23-96607 HOH KSA

#### Construction Supervision

> Typical deliverables include:
> • Contract meetings with the Construction and Design Team as considered necessary by the design team (the number of site meetings will be quantified).
> • Review Contractors production information
> • Resolution of site queries
> • Final review of package installation.
> • The Package Contractor shall supply personnel to perform this work in compliance with the local legislation and union agreements.

(The HOH KSA / Fountain Control variants add ongoing reviews, programme maintenance, regular Client reporting, and pre-commissioning supervision. Wynn AA folds focussing/aiming, scene-setting and defects deliverables into this clause.)

Used by: 23-97105 Tape Theatre, 25-97103 Aljada, 24-97106 RAK Sled, 23-97111 Dubai Island Promenade, 23-96606 KSA Pav Osaka, 23-96607 HOH KSA, 23-97106 Fountain Control, 23-97102 Wynn AA

#### Focussing / Aiming and Scene Setting

> Typical deliverables include:
> • Supervision of focusing / aiming of adjustable device.
> • Oversee the programming of installed control equipment where applicable.
>
> Important Notes:
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> • Any content required for the system operation shall be installed / loaded by the contractor prior to programming activities commencing.
> • emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Contractor].

(The JOH variant — "square one" layouts + patch/config per venue, Client-provided engineer, "3 weeks on-site" — is a distinct multi-venue rework.)

Used by: 23-96607 HOH KSA, 23-97111 Dubai Island Promenade, 23-96606 KSA Pav Osaka, 24-97106 RAK Sled, 24-96606 JOH

#### Commissioning / Handover and Defects

> Typical deliverables include:
> • Assistance with producing the package defects report to responsible areas.
> • Assistance in checking completion / making good of any defects noted in our system report and any other relevant reports.

(The HOH KSA variant adds "Review and witness training and handover sessions provided to the Client by the awarded contractor.")

Used by: 23-97111 Dubai Island Promenade, 24-97106 RAK Sled, 23-96606 KSA Pav Osaka, 23-97105 Tape Theatre, 23-96607 HOH KSA

#### Post Completion / DLP

> Typical deliverables include:
> • Assistance with providing advice on any defects reported by the contract administrator during the defects liability period up to a maximum of 12 months from practical completion.
> • Conduct operational reviews and assessments as appropriate.

Used by: 24-97106 RAK Sled, 23-96606 KSA Pav Osaka, 23-97105 Tape Theatre, 25-97103 Aljada

#### MPME Coordination (stage-lighting bespoke — JOH)

Two project-specific clauses for the JOH stage-lighting engagement: "develop and deliver approved system drawings ... to LOD 400" and "update the systems drawings ... 3.5 days per month in this phase". Distinct enough not to merge with the standard Detailed Design clause.

Used by: 24-96606 JOH (FP-01, FP-01 comments, FP-01 comments response, FP-02)

#### Programming / Show Design / Training (Fountain Control bespoke)

Project-specific deliverables for show programming ("develop and program an agreed number of shows [currently 20]"), operational-staff training, and Material/Shop-Drawing submittal review — unique to 23-97106 Fountain Control.

Used by: 23-97106 Fountain Control

---

**Recommended standard wording:**

For the **stage-list overview**, the cleanest canonical form is the full-lifecycle dash list with explicit Design and Post Contract phases, dropping em-dashes in favour of plain stage labels:

> This proposal includes the following project stages:
> - Design Phase
>   - Preliminaries
>   - Stage 1: Concept Design
>   - Stage 2: Schematic Design
>   - Stage 3: Detailed Design
>   - Stage 4: Tender Documentation
> - Post Contract Phase [Optional]
>   - Stage 5: Construction Supervision
>   - Stage 6: Focussing / Aiming and Scene Setting
>   - Stage 7: Commissioning and Handover
>   - Stage 8: Post Completion / DLP

Projects then add/remove stages and scope inclusion/exclusion blocks as needed.

For the **individual stage deliverables**, adopt one canonical body per stage with the package descriptor as the only variable token (`<Package>` = Specialist Lighting / Audio / Sensory Design / etc.):

- **Concept Design Report** — the Specialist Lighting body above, with the recommendations bullet `[such as lux levels, uniformities, resolution, sound levels, and intelligibility as required]` for visual packages or `[such as sound levels, intelligibility, and spatialisation as required]` for audio, always closing with "We have allowed for up to one revision of the Concept Design Report."
- **Schematic Design Report** — the full LOD 200 body above (it is the superset; reduced variants are subsets), closing with the one-revision line.
- **Detailed Design** — the LOD 300 "Basis of Design" body above, keeping Tender Documentation as a *separate* stage rather than merging.
- **Tender Documentation** — the "Open Tender" five-bullet body above; append the three Tender-Return-Review bullets only when there is no separate Tender Return Review stage.
- **Construction Supervision / Focussing & Aiming / Commissioning / DLP** — the standalone bodies above, kept as discrete stages rather than folded together (the merged Wynn-AA form harms readability).

Standardise on a single bullet character (dash) and plain colons in stage labels to eliminate the bullet-vs-dash and em-dash-vs-hyphen variant churn that produced most of the near-duplicates above.

---

## Design Phase Notes

This clause type ("Design Phase Notes") spans two distinct content families that appear under it across the FP corpus:

1. **Assumptions / disclaimer notes** - the standard "Design Phase Notes" boilerplate (architect coordination, CAD-file readiness, engineering-specialist disclaimers). This is the dominant family.
2. **Stage deliverable blocks** - Concept, Schematic, Detailed Design and Tender/IFT stage descriptions that were filed under this clause type in many proposals.

Variants are grouped by family below. Trivial wording differences (bullet glyph `•` vs `-`, en-dash vs hyphen, "Lighting/Audio/Sensory" discipline name swaps) are merged with the difference noted.

---

### Family A - Design Phase Notes (assumptions / disclaimers)

#### A1. Standard 8-point notes (canonical, Architect + Electrical Engineer)

The core boilerplate. Two near-identical variants differ only in bullet glyph: `•` (21 uses) and `-` (17 uses). Same eight points verbatim.

> • This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> • Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> • emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> • It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

**Difference between the two:** bullet glyph only (`•` vs `-`); text is otherwise identical.

Used by (`•` variant, 21): 23-96601 Dammam Adv World, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-96607 HOH KSA, 23-97101 Marina Island, 23-97102 Wynn (AA), 23-97107 DAFZA Refurb, 23-97108 Ciel Lobby, 23-97111 Dubai Island Promenade, 24-97102 DMC 2A, 24-97106 RAK Sled, 24-97107 Reserve Cut +9 more (24-97108 Tamani, 25-97105 Shanghai Tang v2, 25-97106 MAF MOE FEC v3, 23-96606 KSA Pav Osaka, 23-97110 HoH Dxb Update, 24-97109 MOE FEC v2, 25-97101 Shanghai Tang, 25-97104 MAF MiCC Kids FEC Redesign)

Used by (`-` variant, 17): 22-97111 TPF Sound System, 22-97115 Kids Activity Park AUH, 23-96604 DockX, 23-96605 Jumpoline, 23-97102 Wynn (EL), 23-97105 Tape Theatre, 23-97106 Fountain Control, 23-97109 Ciel VIP, 23-97111 Dubai Island Promenade, 24-96601 Dammam Water Park, 24-96602, 24-97105 MAF MiCC Kids +5 more (24-97113 Level 63, 25-97101 Shanghai Tang)

#### A2. Standard notes + "illustrative renders" point (9 points)

Identical to A1 but adds a point: *"Any images prepared by emittiv shall be illustrative in nature, and intended to convey the overall design intent. If realistic renders are required, these shall be prepared by the Architect with inputs and comments by emittiv."* Appears with both glyphs (`-` 10 uses, `•` 2 uses).

> - This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> - Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> - emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> - It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> - Any images prepared by emittiv shall be illustrative in nature, and intended to convey the overall design intent. If realistic renders are required, these shall be prepared by the Architect with inputs and comments by emittiv.
> - emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> - All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> - emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> - The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

Used by (`-` variant, 10): 24-97111 Masdar OS-48, 24-97112 Military Museum, 25-97103 Aljada FG, 25-97107 Cove Boulevard, 25-97109 Mapletree Warehouse, 26-96801 Rozana Muscat, 25-97102 WAMI (+ 3 doc revisions of the above)

Used by (`•` variant, 2): 24-97110 Masdar B-02, 25-97108 RAK Beach District

#### A3. Notes + "client provides standards/codes" point

A2 plus one extra point: *"Client shall provide emittiv with digital [vectorised] copies of all relevant standards, codes and regulations upon request."* (`•` glyph).

> • This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> ... [A1/A2 points] ...
> • Client shall provide emittiv with digital [vectorised] copies of all relevant standards, codes and regulations upon request.
> • emittiv is not an engineering specialist. ...

Used by: 24-96603 Marasi Gate

#### A4. Notes - "Client digital copies / unless noted" wording

A2 with two phrasing tweaks: CAD line reads *"CAD files / drawings / models provided by the Client"* and the renders point ends *"...with inputs and comments by emittiv unless specifically noted otherwise."* (`•` glyph).

> • ... It is assumed that CAD files / drawings / models provided by the Client will be in a ready to use format and require no work by emittiv to prepare them.
> • Any images prepared by emittiv shall be illustrative in nature ... unless specifically noted otherwise.
> ...

Used by: 24-96605 Observatory

#### A5. Notes - Lead Contractor variant (no Electrical Engineer / no "not issued to Contractor")

A materially different assumptions set: addresses the **Lead Contractor** (not Architect), drops the "drawings not to be issued to the Contractor" and "information for use by Architect/EE" points, and adds a site-visit additional-charges clause. Used on a contractor-led (D&B) engagement.

> - This proposal assumes that the Lead Contractor and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> - It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> - emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> - All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> - emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> - The exact method of fastening or connection must be verified by the Site team or relevant Specialist Contractor.
> - In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Used by: 24-96606 JOH

#### A6. Notes wrapped with a 3-stage preamble

A1 (`•` glyph) prefixed with a stage-structure preamble. The notes block itself is identical to A1.

> This proposal includes the services described below and is divided into Stages 1-3. Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work. The deliverables outlined in this document are typical for a wide range of projects. Specific Deliverables will be agreed with the Client during Preliminaries.
> • This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer ... [A1 eight points]

Used by: 25-97106 MAF MOE FEC v3 (FP-01)

---

### Family B - Concept Design stage

The standard Concept Design Report deliverables block. The only meaningful variation is the **discipline name** in the report title and minor tweaks to the "recommendations" point. Treat these as one clause with a discipline placeholder.

#### B1. Concept Design - Specialist Lighting (canonical)

> We will prepare a Specialist Lighting Concept Design Report for the Project which will provide guidelines for the Client and other Designers to ensure that the Specialist Lighting concepts are in line with the overall Project aspirations. This report shall be based on coordination with other Designers / Consultants as well as requirements of the Client.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings
> - Briefing and presentation meetings with the Client
> - Carrying out conceptual design studies
> - Undertake research to establish current / future trends and establish design baselines
> - Provide recommendations on package requirements [such as lux levels, uniformities, resolution, sound levels, and intelligibility as required]
> - High level Package Master Plans
> - Produce presentation materials to convey the overall design intention
>
> We have allowed for up to one revision of the Concept Design Report.

Used by (4 + 2 glyph-only variant): 23-96604 DockX, 24-96602, 25-97103 Aljada FG, 22-97113 MAF FEC (FP-02); plus `•`-glyph variant on 23-97101 Marina Island, 22-97113 MAF FEC (FP-01)

#### B2. Concept Design - discipline-renamed variants

Same block as B1 with the discipline name swapped. Otherwise identical (some use `•`, some have minor "recommendations" tweaks noted):

- **Sensory Technology** - 23-96602 Shoot the Chute (`•`)
- **Sensory Design** - 23-97108 Ciel Lobby (FP-01, `•`)
- **Sensory Systems** - 23-97108 Ciel Lobby (FP-02, `•`; adds "reverberation times" to recommendations)
- **Audio** - 23-97105 Tape Theatre (`-`; recommendations reworded to "sound levels, intelligibility, and spatialisation"; adds a content-recommendations point)
- **Lighting and Audio** - 24-97113 Level 63 (`-`)
- **Specialist Lighting (screen resolution / "as required")** - 22-96601 Dammam Waterpark (`•`; "screen resolution" + trailing "as required")

---

### Family C - Schematic Design stage

"In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase." Differences are: bullet glyph, presence/absence of the **LOD 200 3D-layouts** point, "further/preliminary development" wording, and report-name ("Schematic Design" vs "50% Detailed Design").

#### C1. Schematic Design - with LOD 200 point (canonical, fuller)

> In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings
> - Carry out further development of package design
> - Carry out preliminary development of package specific 3D layouts based on linked files to LOD 200 standards
> - Prepare preliminary 3D package calculations [as required]
> - Prepare preliminary package layouts
> - Prepare preliminary package detail sketches
> - Prepare preliminary package equipment schedule
> - Prepare preliminary package control intent
> - Prepare preliminary logical control channel schedule
> - Prepare preliminary package scene information
> - Prepare preliminary package power requirements
> - Assist others in checking that the package schemes are within the approved budgets
> - Identify integration points with other systems as required
>
> We have allowed for up to one revision of the Schematic Design Report.

Used by: 23-97101 Marina Island, 23-97108 Ciel Lobby (FP-01), 24-96602, 25-97103 Aljada FG, 22-96601 Dammam Waterpark (uses "preliminary development of package design")

#### C2. Schematic Design - without the LOD 200 point

Same as C1 minus the "LOD 200 standards" line; "further development of package design". Appears with both glyphs.

Used by: 23-96604 DockX, 24-97113 Level 63, 23-96602 Shoot the Chute, 23-97108 Ciel Lobby (FP-02, adds "Identify and recommend appropriate acoustic treatments"), 23-97105 Tape Theatre ("3D package calculations as appropriate"), 25-97106 MAF MOE FEC v3

#### C3. Schematic Design - "50% Detailed Design Report" revision wording

Identical body to C2 but the revision line reads *"We have allowed for up to one revision of the 50% Detailed Design Report."*

Used by: 22-97113 MAF FEC (FP-01 `•`, FP-02 `-`)

---

### Family D - Detailed Design stage ("Basis of Design")

"In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender ... from a 'Basis of Design' perspective." Variation: bullet glyph, presence of **LOD 300 3D-layouts** point, and tail additions (final spec/schedule/costing lines pulled in from the tender stage).

#### D1. Detailed Design - core (no LOD 300, no tail)

> In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender and finalised by the awarded contractor for construction from a "Basis of Design" perspective.
>
> Typical deliverables include:
> • Attend design workshops / Design Team meetings / mock-ups
> • Prepare final package layouts
> • Assist others with the coordination of the package design
> • Prepare final package equipment schedule
> • Prepare final package details where applicable for incorporation into Architect's details
> • Prepare final 3D package calculations
> • Prepare final package power requirements
> • Specify integration points with other systems as required

Used by: 23-96602 Shoot the Chute, 23-97108 Ciel Lobby (FP-02), 24-97105 MAF MiCC Kids and similar

#### D2. Detailed Design - with LOD 300 point

D1 plus *"Carry out further development of package specific 3D layouts based on linked files to LOD 300 standards"*.

Used by: 22-96601 Dammam Waterpark, 23-97101 Marina Island, 23-97108 Ciel Lobby (FP-01), 25-97103 Aljada FG, 25-96601 (within combined block)

#### D3. Detailed Design - with appended final-spec / costing tail

D1/D2 body plus tail lines: *"Final system calculations and compliance... Final specification and schedule of devices... allow detailed costing by others... Final specification for the operational requirements of the control system and control schedule."* (Effectively merges Detailed Design + Tender deliverables.)

Used by: 24-96602 (with LOD 300), 22-97113 MAF FEC (both revisions)

#### D4. Detailed Design - minor wording variants

- "with indicative pricing" on the equipment-schedule line - 23-96604 DockX
- "system control programming specification for operations of the venue" added - 23-97105 Tape Theatre
- "and their integration with other design disciplines" preamble - 24-97113 Level 63
- "Prepare layout and elevation drawings / lighting calculations / lighting control schematic drawings" (lighting-specific deliverables) - 25-97104 MAF MiCC Kids FEC Redesign

---

### Family E - Tender / IFT stage

"This stage will prepare the design documents so they can be issued for tender..." Variation: tender basis (**"Open Tender"** vs **"Design and Build"**), and whether deliverables are listed.

#### E1. Tender stage - "Open Tender" with full deliverables (canonical)

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.
>
> Typical deliverables include:
> - Following approval provide documents for final tender and assistance in co-ordinating system design with architectural, electrical, structural and mechanical design.
> - Final system calculations and compliance with relevant Building Regulations.
> - Final specification and schedule of devices, accessories and associated equipment.
> - The above information will allow detailed costing by others - the Project's Cost Estimator / Quantity Surveyor / Tendering Contractor / other.
> - Final specification for the operational requirements of the control system and control schedule.

Used by: 23-96602 Shoot the Chute, 23-97101 Marina Island (en-dash variant), 24-97113 Level 63 ("...construction." no basis phrase)

#### E2. Tender stage - "Design and Build" basis

Same as E1 but *"...from a 'Design and Build' perspective."* Some are short (2 deliverables only).

Used by: 22-96601 Dammam Waterpark (full), 22-97113 MAF FEC (both revisions; short 2-line: tender docs + final calculations only)

#### E3. Tender stage - one-line (no deliverables)

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.

Used by: 22-96601 Dammam Waterpark

#### E4. Tender stage - "Revised IFT documentation pack"

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.
>
> Typical deliverables include:
> - Revised IFT documentation pack addressing any questions or issues that arise after design works.

Used by: 22-97113 MAF FEC (both revisions)

#### E5. Tender stage - with appended Tender Return Evaluation

E1 body plus a "Stage 4b - Tender Return Evaluation" block (substitute-device review, submittal analysis, scoring matrix). Also seen merged into the Tender deliverables on 25-97103.

Used by: 23-97105 Tape Theatre, 25-97103 Aljada FG (merged)

---

### Family F - Tender Return Evaluation (standalone)

> Typical deliverables include:
> - Review and checking of proposed substitute devices, as selected by others, for compliance with specification.
> - Review and analyse of submittals based on technical compliance, company experience and other factors as required.
> - Prepare scoring matrix comparing submitted package tenders.

Used by: 23-97101 Marina Island, 22-97113 MAF FEC (both revisions)

---

### Family G - Project-specific blocks (do not generalise)

These are bespoke to a single proposal and should be kept as project-specific clauses, not promoted to standard wording:

- **22-97112 The View Exp Ctr - audio system review (2-stage).** "Stage 1 - Review and Report of Current Sound System" + "Stage 2 - Calibrate, EQ, Setup the Audio Experiential Experience", with the standard disclaimer notes appended. Also stored as separate split fragments (Stage-1 content, Stage-2 deliverables).
- **25-97102 WAMI - lighting control system review/coordination.** Two bespoke deliverable blocks (site-wide lighting control review report; schematic-stage sitewide maps/matrix with "Recommended additional scope elements").
- **25-96601 - full Stages 1-7 combined block.** A single clause concatenating the Preliminaries, Concept (Lighting/Video/SFX), Schematic (LOD 200), Detailed (LOD 300), and the A2 Design Phase Notes. This is a whole-proposal scaffold, not a reusable note.

---

**Recommended standard wording:**

For the **Design Phase Notes** clause type proper (Family A), adopt **variant A2** (9 points, including the illustrative-renders clause) as the canonical default, using the `•` bullet glyph for consistency with the dominant proposal set. A2 is the most complete general-purpose set: it covers everything in A1 plus the renders responsibility, which is the single most common addition across the corpus. Keep two parameterised options:

- **Lead designer:** `Architect [or other Lead Designer]` (default) vs `Lead Contractor` (Family A5, for Design-and-Build / contractor-led engagements - which also swaps the final-fastening point to "Site team or relevant Specialist Contractor" and drops the "not issued direct to the Contractor" point).
- **Optional add-on points** (include as needed, not by default): the "Client shall provide... digital [vectorised] copies of all relevant standards, codes and regulations" point (A3), and the site-visit additional-charges point (A5).

Canonical A2 text:

> • This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> • Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> • emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> • It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> • Any images prepared by emittiv shall be illustrative in nature, and intended to convey the overall design intent. If realistic renders are required, these shall be prepared by the Architect with inputs and comments by emittiv.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

For the **stage deliverable families (B-F)** that were filed under this clause type, the recommended canonical versions are the fuller, glyph-normalised forms: **B1** (Concept, discipline-parameterised), **C1** (Schematic, with LOD 200), **D2** (Detailed Design, with LOD 300), **E1** (Tender, "Open Tender" with full deliverables), and **F** (Tender Return Evaluation). These should ideally be migrated to their own clause types (Concept Design / Schematic Design / Detailed Design / Tender) rather than remaining under "Design Phase Notes".

---

## Post Contract Phase Notes

This section consolidates the distinct variants of the "Post Contract Phase Notes" clause type found across fee proposals. Variants fall into two broad families: (A) **scope assumptions / coordination caveats** (the boilerplate notes about who maintains coordination roles, emittiv's limits as a non-engineering specialist, and additional-charge triggers), and (B) **stage deliverable blocks** (typical deliverables for individual post-contract stages such as design management, supervision, commissioning, programming, hand-over/training, and DLP). The most-used variants are the coordination caveats.

---

### A. Coordination & assumptions caveats

#### A1. Standard coordination + abortive-visit charge (most common)

The canonical post-contract assumption pair. Two bullet content variants exist that are identical apart from the bullet glyph (hyphen `-` vs. typographic `•`).

> - This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> - In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Differences: hyphen-bullet version (count 9) vs. `•`-bullet version (count 8); wording otherwise identical.

Used by: 23-97102 Wynn, 23-97105 Tape Theatre, 23-97106 Fountain Control, 23-97111 Dubai Island Promenade, 25-97103 Aljada FG, 25-97103 Aljada Block F+G, 25-97102 WAMI (x3), 23-96607 HOH KSA, 23-97101 Marina Island, 24-97106 RAK Sled, 24-97110 Masdar B-02, 25-97108 RAK Beach District, 23-96606 KSA Pav Osaka, 25-96601 +2 more

#### A2. Extended coordination + non-engineering-specialist disclaimers (Lead Contractor variant)

Expands A1 with CAD-readiness assumption, the "emittiv is not an engineering specialist" disclaimer set, and structural/safety verification caveats. Names the **Lead Contractor** (not Architect) as coordinator. Hyphen and `•` bullet variants exist.

> • This proposal assumes that the Lead Contractor and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> • It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Site team or relevant Specialist Contractor.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Differences: `•`-bullet version (count 2) vs. hyphen-bullet version (count 1, 24-96606 JOH FP-02); wording otherwise identical.

Used by: 24-96606 JOH (x3, incl. FP-01 comments, FP-01, FP-02)

#### A3. Architect/Lead-Designer tender-documentation variant

Names the **Architect [or other Lead Designer]** and adds tender-documentation handling clauses: information is for the design team's tender prep, drawings not to be issued direct to the Contractor, plus the non-engineering-specialist disclaimers. Final verification routed to Architect / Structural Engineer.

> • This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> • Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> • emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> • It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

Used by: 22-96601 Dammam Waterpark

#### A4. Architect coordination + structural disclaimers + abortive-visit (hybrid)

A1's Architect coordination line and abortive-visit line, with the structural/safety disclaimer set (A2/A3) inserted between. No CAD-readiness or tender-documentation lines.

> • This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Architect / Structural Engineer.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Used by: 24-97101 HoH Supervision FP-02

#### A5. Coordination + commissioning/programming preconditions + content caveats

A1's two lines plus the commissioning-precondition notes (system focused/aimed/programmed only once fully installed and commissioned by the contractor/manufacturer) and content-provision caveats, including a project-specific Pixel Mapping content line.

> • This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> • Pixel Mapping content shall be provided by the Client.
> • Any content required for the system operation shall be installed / loaded by the contractor prior to programming activities commencing.

Used by: 24-97104 HoH Programming

---

### B. Stage deliverable blocks

These variants document the "Typical deliverables include:" content for individual post-contract stages. They are organised by stage function.

#### B1. Site supervision / contract administration deliverables

Contract meetings, review of contractor production information, site-query resolution, final review of installation, with the union-compliance personnel note. A standalone short form (count 3) and an expanded form merging programming, defects and important-notes content into a single block (count 1) both occur.

> Typical deliverables include:
> • Contract meetings with the Construction and Design Team as considered necessary by the design team (the number of site meetings will be quantified).
> • Review Contractors production information
> • Resolution of site queries
> • Final review of package installation.
> • The Package Contractor shall supply personnel to perform this work in compliance with the local legislation and union agreements.

Differences: hyphen-bullet equivalents exist (25-97103 Aljada FG); 23-97102 Wynn EL-FP-01 uses an expanded single-block form adding programming oversight, defects assistance, focusing/aiming and commissioning important-notes.

Used by: 23-97101 Marina Island, 24-97110 Masdar B-02, 22-97113 MAF FEC, 25-97103 Aljada FG (incl. expanded 23-97102 Wynn EL-FP-01)

#### B2. Focusing / aiming & programming supervision (short form)

Supervision of focusing/aiming and oversight of programming, plus the standard commissioning Important Notes. Some variants append the scene-setting and content-loading lines; bullet glyph varies (• vs -).

> Typical deliverables include:
> • Supervision of focusing / aiming of adjustable device.
> • Oversee the programming of installed control equipment where applicable.
>
> Important Notes:
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> • Any content required for the system operation shall be installed / loaded by the contractor prior to programming activities commencing.
> • emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Contractor].

Differences: 23-97101 / 22-97113 form omits the content-loading and scene-setting lines; 25-97103 Aljada FG uses hyphen bullets and includes all lines.

Used by: 23-97101 Marina Island, 22-97113 MAF FEC, 25-97103 Aljada FG

#### B3. Programming (undertaken, not supervised) — full Important Notes

emittiv undertakes the programming and establishes connectivity, with the fuller Important Notes set (adds the "other systems must be in operational states" line). Includes a day-allowance paragraph.

> Typical deliverables include:
> • Undertake the programming of installed control equipment.
> • Establish connectivity with site wide control devices.
>
> Important Notes:
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the Contractor and/or Manufacturer.
> • Other systems that impact or rely on the lighting scenes [screens, projection, cameras, signage etc.] must be in their operational states during the lighting programming.
> • Any content required for the system operation shall be installed / loaded by the Contractor prior to programming activities commencing.
>
> This proposal allows for 7 days of programming time in this stage. Typical staffing would include the Designer and 1 Programmer. Some works may be undertaken off site, prior to commencement, in order to facilitate a streamlined workflow.

Used by: 22-97114 HoH

#### B4. Commissioning validation deliverables

Review/feedback on commissioning procedures, addressing/grouping confirmation, functionality and grouping test witnessing, and direct aiming/focussing. Includes day-allowance paragraph.

> Typical deliverables include:
> • Review and provide feedback on commissioning procedures developed by the Contractor.
> • Confirm addressing and grouping structures are appropriate for intended use.
> • Confirm control system is operational and ready for programming.
> • Witness functionality and grouping tests to confirm the system is ready for programming.
> • Coordinate addressing of fixtures with Programmers to streamline the works.
> • Direct aiming and focussing of adjustable devices.
> - The Package Contractor shall supply personnel to perform this work in compliance with the local legislation and union agreements.
>
> This proposal allows for 8 days on site for this stage. Typical staffing would include the Designer and 1 Assistant. Given the number of spaces in this project, multiple rooms should be ready for inspection on each visit.

Used by: 22-97114 HoH

#### B5. Installation supervision deliverables

Attend meetings/site inspections, provide client feedback, resolve queries, final review. Includes day-allowance/roll-over paragraph.

> Typical deliverables include:
> • Attend meetings with the Installation and Design Teams as considered necessary by the Design Team (the number of site meetings will be quantified).
> • Attend site inspections as agreed to review the progress and workmanship of the Contractor on site.
> • Provide feedback to the Client following site inspections.
> • Resolution of site queries.
> • Final review of package installation.
>
> Installation Supervision is estimated at 1 day per month over a 6 month installation period. This time can be broken up into 2 x 1/2 day visits or sessions as required, and spread over multiple months. Any unused time shall roll over into following months. Delays in construction timelines may require additional visits.

Used by: 22-97114 HoH

#### B6. Commissioning programming supervision (focus/aim + program oversight)

Witness-style supervision of focusing/aiming and programming oversight, with full commissioning Important Notes. Lighting-control-system focused variant.

> Typical deliverables include:
> • Supervision of focusing / aiming of adjustable device.
> • Oversee the programming of installed control equipment where applicable.
>
> Important Notes:
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> • Any content required for the system operation shall be installed / loaded by the contractor prior to programming activities commencing.
> • emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Contractor].

(See B2 — this is the variant including the content-loading and scene-setting lines.)

Used by: 23-97101 Marina Island, 22-97113 MAF FEC

#### B7. Contractor return / submittal evaluation deliverables

Review of substitute devices, submittal analysis, client recommendations, and tender scoring matrix.

> Typical deliverables include:
> • Review and checking of proposed substitute devices, as selected by others, for compliance with specification.
> • Review and analyse submittals based on technical compliance, company experience, project understanding, and other factors as required.
> • Provide recommendations to the Client based on previous experiences and local knowledge.
> • Prepare scoring matrix comparing submitted package tenders.

Used by: 22-97114 HoH

#### B8. Contractor / procurement management deliverables

Regular contractor/client meetings and procurement-pathway support, with day-allowance paragraph.

> Typical deliverables include:
> • Attend regular meetings with Contractors and Client to ensure ongoing progress and development of the system.
> • Provide support to Contractor to develop installation programmes and timelines.
> • Provide support to Contractor and Client for procurement pathways.
> • Provide support to Contractor and Client to minimise lead times.
>
> This proposal allows for 9 days [envisaged to occur over 6 weeks] of work in this stage. This time will be split between Technical, Design, and Project Management staff as required.

Used by: 22-97114 HoH

#### B9. Shop drawing review deliverables

Pre-commencement workshop, contractor production-information review, design-query resolution, with the "Substantial Completion" invoicing note.

> Typical deliverables include:
> • Attend workshop / meetings with Contractor to clarify questions from the system engineering team prior to commencement.
> • Review Contractors production information.
> • Ongoing resolution of design queries.
> • Provide recommendations to Client in order to resolve coordination issues as required.
>
> For clarity, Shop Drawing reviews are dependent on submittals by the Contractor. An agreement shall be reached with the Client on what shall constitute "Substantial Completion" and allow invoicing of the final payment for this stage. Any outstanding drawings shall be reviewed as they are submitted.

Used by: 22-97114 HoH

#### B10. Hand-over & training deliverables

Defects-report assistance, defect making-good checks, training-session development, and hands-on operator training, with day-allowance paragraph.

> Typical deliverables include:
> • Assistance with producing the package defects report to responsible areas.
> • Assistance in checking completion / making good of any defects noted in our system report and any other relevant reports.
> • Develop training session[s] for operations staff to explain the functionality of the Lighting Control Systems.
> • Conduct some "Hands On" training with selected operations staff to familiarise them with required actions.
>
> This proposal allows for 2 days of work in this stage. This is intended to be completed by the Designer and will be broken down to allow for session planning and delivery for the Lighting Control System.
> The Contractor shall provide training for the operation and maintenance of the installed fittings and devices. This session shall be attended by emittiv to confirm sufficient information is provided to the operations staff.

Used by: 22-97114 HoH

#### B11. Defects-report / making-good assistance (short form)

Standalone defects assistance pair, without the training lines.

> Typical deliverables include:
> • Assistance with producing the package defects report to responsible areas.
> • Assistance in checking completion / making good of any defects noted in our system report and any other relevant reports.

Used by: 23-97101 Marina Island, 22-97113 MAF FEC

#### B12. Defects Liability Period (DLP) deliverables — short form

Advice on reported defects up to 12 months from practical completion plus operational reviews. Bullet glyph varies (• vs -).

> Typical deliverables include:
> • Assistance with providing advice on any defects reported by the contract administrator during the defects liability period up to a maximum of 12 months from practical completion.
> • Conduct operational reviews and assessments as appropriate.

Differences: hyphen-bullet equivalent at 25-97103 Aljada FG.

Used by: 23-97101 Marina Island, 22-97113 MAF FEC, 25-97103 Aljada FG

#### B13. DLP inspection deliverables — extended form

DLP variant adding site inspections, a client report, and the quarterly inspection schedule paragraph.

> Typical deliverables include:
> • Attend site inspections to review the functionality of the installed systems.
> • Provide advice on any defects reported by the contract administrator during the defects liability period up to a maximum of 12 months from practical completion.
> • Prepare a report for the Client outlining observations and findings during the inspection.
> • Conduct operational reviews and assessments as appropriate.
>
> DLP inspections typically consist of a visit every 3 months to inspect and review the installed system in an operational state. These usually commence after 3 months of operation.

Used by: 22-97114 HoH

#### B14. Site-wide commissioning validation (specified + recommended)

Specified deliverables for site-wide lighting-control commissioning (review information, develop commissioning plan, verify on site, interim and final reports), plus recommended additional scope and the union-compliance note. Two near-identical variants differ only in the on-site verification wording: "scene activation and lighting levels" vs. "zones and control group activation and lighting control functionality".

> Specified deliverables include:
> - Review new lighting information and updated report and supporting documents as appropriate.
> - Review and comment on lighting control commissioning documentation.
> - Develop sitewide lighting control commissioning plan and lead testing with support of areas lighting control contractors.
> - Verify commission on site including scene activation and lighting levels for each lighting control system.
> - Prepare interim reports on testing and commissioning progress.
> - Prepare and present final report.
>
> Recommended additional scope elements:
> - Review Contractors production information as required.
> - Resolution of site queries.
> - Final review of package installation.
>
> note: The Package Contractor shall supply personnel to perform aiming and focussing work in compliance with the local legislation and union agreements.

Differences: 25-97102 FP-02 substitutes "Verify commission on site including zones and control group activation and lighting control functionality" and capitalises "Note:".

Used by: 25-97102 WAMI FP-01 (incl. WDD notes), 25-97102 WAMI FP-02

#### B15. JOH bespoke programming deliverables (Client-commissioned)

Project-specific JOH variant: "square one" layouts and patch/config per venue, focusing/aiming and programming oversight, with Important Notes referencing **Client** commissioning (not Contractor/Manufacturer) and an on-site duration note.

> Deliverables include:
> - Prepare "square one" layouts for each venue
> - Prepare patch and configuration information for each venue
> - Supervision of focusing / aiming of adjustable device
> - Oversee the programming of installed control equipment
>
> Important Notes:
> - This will be carried out following the Client's commissioning of the system and after confirmation of all circuits / cables being correctly installed, labelled and operational.
> - The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> - emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Client].
>
> It is anticipated that preparatory works for this stage shall be completed off-site and comprise part of Stage 2.
> We have allowed for 3 weeks on-site.

Used by: 24-96606 JOH (comments response)

---

### C. Multi-stage composite blocks (whole post-contract proposals)

Some documents store the entire multi-stage post-contract scope as a single block. These are full-proposal compositions, not reusable clauses, and are catalogued here for traceability rather than as standardisation targets.

#### C1. Tait internal project — Stages 4-7 with "No Offer" on PM

Composite covering Design Management (Stage 4), a **Stage 5 Project Management "No Offer"** decline, Site Supervision (Stage 6, optional), and Programming (Stage 7, optional/recommended), closing with the A1 coordination + abortive-visit notes under a "Post Contract Phase Notes:" heading.

> Stage 4 - Design Management - Required:
> There is a significant time frame between having the designs completed and beginning the procurement process… [Stages 4-7 with Stage 5 "No Offer" decline; full text retained in source 25-96601-FP-01]
> …
> Post Contract Phase Notes:
> • This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Used by: 25-96601

#### C2. HoH Supervision — full Stages 1-7 post-contract proposal

Complete seven-stage supervision proposal (Contractor Return Evaluation through Hand Over and Training) with embedded day allowances per stage. This is a full-proposal composite, not a reusable clause.

> This proposal includes the services described below and is divided into Stages 1-7… [Stages 1-7: Contractor Return Evaluation, Contractor/Procurement Management, Shop Drawing Review, Installation Supervision, Commissioning Validation, Programming Supervision, Hand Over and Training; full text retained in source 24-97101-FP-01]

Used by: 24-97101 HoH Supervision FP-01

---

### D. Adjacent stage block (not strictly post-contract)

#### D1. Tender / construction documentation stage deliverables

A pre-construction "issue for tender" stage block that appears in the corpus alongside post-contract notes. Listed for completeness; it is a documentation stage, not a post-contract clause.

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction from a "Open Tender" perspective.
>
> Typical deliverables include:
> • Following approval provide documents for final tender and assistance in co-ordinating system design with architectural, electrical, structural and mechanical design.
> • Final system calculations and compliance with relevant Building Regulations.
> • Final specification and schedule of devices, accessories and associated equipment.
> • The above information will allow detailed costing by others - the Project's Cost Estimator / Quantity Surveyor / Tendering Contractor / other.
> • Final specification for the operational requirements of the control system and control schedule.

Used by: 25-97104 MAF MiCC Kids FEC Redesign

---

**Recommended standard wording:**

For the headline **Post Contract Phase Notes** clause itself, A1 is the clear canonical form (17 documents combined across both bullet glyphs). Standardise on the typographic-bullet version, with project-specific party names substituted as needed:

> • This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Where the engagement involves emittiv assessing structural/rigging elements or providing tender documentation, extend with the A2/A3 disclaimer block (non-engineering-specialist, structural verification, fastening verification) and swap "Architect" for "Lead Contractor" or "Architect [or other Lead Designer]" to match the project's lead party. The stage deliverable blocks (Section B) and composites (Section C) are scope content rather than notes, and should be templated per stage rather than merged into the headline clause.

---

## Design Phase Fees

This clause type presents the staged fee schedule for the design phase (and, in some proposals, review/supervision/post-contract fees). Across 67 distinct variants the structure is highly consistent: a standard **applicability preamble** ("These rates are applicable if the complete scope... is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees."), a **per-stage fee table** carrying project-specific figures, and a closing **fee-variation notification clause** ("Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval."). The variants below are grouped by what actually differs: the boilerplate wording, the table layout, the currency, and any added qualifier clauses. Project-specific fee figures are reproduced verbatim but are not themselves the basis for grouping.

---

### A. Standard staged design fee (preamble + stage table + variation notice)

The dominant pattern. The two fixed clauses bracket a stage-by-stage fee list. Wording of the two clauses is essentially identical across these projects; the only real differences are (a) whether the stage table appears before or after the variation notice, (b) em-dash vs hyphen vs slash vs pipe as the stage/fee separator, and (c) the number and naming of stages. These are trivial layout/punctuation differences over the same clause.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 — Concept Design: AED 40,000
> Stage 2 — Schematic Design: AED 35,000
> Stage 3 — Detailed Design: AED 35,000
> Stage 4 — Tender Documentation: AED 10,000
> Total Fee: AED 120,000
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Layout variants observed (same clause, cosmetic differences):
- **Colon/dash list** (`Stage 1 — Concept Design: AED ...`) — most common.
- **Pipe-delimited table** (`Stage | Milestone | Fee (AED)` with `1 | Concept Design | 43,400`).
- **Markdown table** (`| Milestone | Stage | Fee (AED) |`).
- **Slash-delimited** (`Stage 1 / Concept Design: AED ...`).
- **"Preliminaries: [included]"** prefix row where preliminaries are bundled into the staged fee.

Used by: 23-96602 Shoot the Chute /FP-01, 23-96603 Khobar Grand Mosque /FP-01, 23-96607 HOH KSA /FP-01, 23-97101 Marina Island /FP-01, 23-97105 Tape Theatre /FP-01 (+ AH-comments variant), 23-97106 Fountain Control /FP-01, 23-97107 DAFZA Refurb /FP-01, 23-97108 Ciel Lobby /FP-01 + /FP-02, 23-97109 Ciel VIP /FP-01 + /FP-02, 23-97111 Dubai Island Promenade /FP-01 + /FP-02, 24-96602 /FP-01, 24-96603 Marasi Gate /FP-01, 24-97102 DMC 2A /FP-01, 24-97106 RAK Sled /FP-01, 24-97108 Tamani Hotel /FP-01, 24-97110 Masdar B-02 /FP-01, 24-97111 Masdar OS-48 /FP-01, 25-97109 Mapletree Warehouse /FP-01, 26-96801 Rozana Muscat /FP-01 +25 more

---

### B. Staged design fee with EUR/USD currency

Identical clause structure to Group A, with the currency expressed in EUR or USD rather than AED. Some USD proposals add prefatory lines ("All prices shown are in USD. ... All rates are exclusive of VAT and expenses.") and/or "Fees are provided on a Lump Sum basis." / "Please consider this our best and final offer."

Representative text (EUR):

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 - Concept Design: EUR 12,500
> Stage 2 - Schematic Design: EUR 25,000
> Stage 3 - Detailed Design: EUR 25,000
> Total: EUR 62,500
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

USD prefatory wording (Wynn EL variant) adds before the table:

> emittiv operates a policy of fair pricing on a fee for service basis across all of our projects.
> Please consider this our best and final offer.
>
> All prices shown are in USD.
> Fees are provided on a Lump Sum basis.
> All rates are exclusive of VAT and expenses.

Used by: 23-96601 Dammam Adv World /FP-01 (EUR, single Schematic stage), 23-96604 DockX /FP-01 (EUR), 23-96605 Jumpoline /FP-01 (EUR), 23-97102 Wynn /AA-FP-01 (USD, A/V + Acoustics columns), 23-97102 Wynn /EL-FP-01 (USD)

---

### C. Staged design fee with department/discipline breakdown

Group A clause plus an additional per-discipline fee matrix (Lighting / Video / Audio / SFX / Control / Acoustics, etc.) broken down by stage. The two standard clauses are unchanged; the breakdown table is appended.

Representative text (excerpt — full discipline matrix retained in source):

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.
>
> Stage 1 — Concept Design: AED 219,375
> Stage 2 — Schematic Design: AED 219,375
> Stage 3 — Detailed Design: AED 263,250
> Stage 4 — Tender Documentation: AED 87,750
> Stage 5 — IFC Documentation: AED 87,750
> Total: AED 877,500
>
> Design Phase - Department Fees
> Stage 1 — Concept Design: Lighting 59,063 | Video 50,625 | Audio 42,188 | SFX 8,438 | Control 16,875 | Acoustics 42,188 | Stage Fee 219,375
> [... per-stage discipline rows ...]
> Totals: Lighting 236,250 | Video 202,500 | Audio 168,750 | SFX 33,750 | Control 67,500 | Acoustics 168,750 | Grand Total 877,500

Used by: 22-96601 Dammam Waterpark /FP-01 (7-department: Lighting/Video/Audio/SFX/Show Ctrl/IPTV/Ani.), 22-97113 MAF FEC /FP-01 + /FP-02 (5-discipline: Lighting/Video/Sound/Technical/Acoustics), 24-97112 Military Museum /FP-01 (6-discipline + Acoustics)

---

### D. Package/department fee (no numbered stages)

Fee expressed by discipline package rather than by design stage. Standard preamble and variation notice retained.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Package | Fee (AED)
> Lighting | 48,000
> Video | 24,000
> Audio | 30,000
> SFX | 6,000
> Control Systems | 12,000
> Total | 120,000
>
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

A second revision of the same project adds a discount line:

> Total: AED 120,000
> Project Discount: AED 20,000
> Grand Total: AED 100,000

Used by: 22-97115 Kids Activity Park AUH /FP-01 (package list, colon form), /FP-02 (with discount/grand total)

---

### E. Multi-block design fee (per-building or per-section blocks)

The fee schedule is split into named blocks (e.g. Block F, Block G), each with its own staged table. Some blocks carry the full standard clauses; the second block in a document often omits them (they apply once for the whole proposal). Also adds post-contract/supervision qualifier lines.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
> Construction Supervision is estimated on 1 staff member being on site per month.
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.
>
> Block F: [Specialist Lighting] — Design Phase - Lump Sum Fees (AED)
> Stage 3 — Schematic Design: 192,500
> Stage 4 — Detailed Design: 220,000
> Stage 4T — Tender Documentation: 82,500
> Stage 5 — IFC Documentation: 55,000
> Total: 550,000

The companion block (Block G) appears as a separate variant carrying only the block heading + table (clauses inherited from the document).

Used by: 25-97103 Aljada FG /FP-01 (Block F + Block G), 25-97103 Aljada Block F + G /FP-02 (two block tables)

---

### F. Review / supervision / engineering fee variants

The same applicability + variation clauses applied to non-standard stage sets: review reports, ongoing management, programming, or unit-priced supervision. The variation-notice "after Stage X" reference adjusts to the relevant stage (Stage 1 where there is no Stage 2). These also tend to carry "Supervision Fees are estimates... should be used for budgetary purposes only" qualifier.

Representative text (supervision, unit-priced):

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Supervision Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Stage | Milestone | Unit Price | Qty | Fee (AED)
> - | Preliminaries | Included | 1 | Included
> 1 | System Engineering | - | 1 | 420,000
> 2 | Ongoing Management | 15,000 | 18 | 270,000
> 3 | Programming | 30,000 | 3 | 90,000
> Total | | | | 780,000
>
> Before exceeding the mentioned fee... after Stage 1 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Used by: 24-96606 JOH /FP-01 (3 layout variants: pipe table, slash list, slash-colon list), 24-96605 Observatory /FP-01 (Review + Optional Renders per-unit + Design Phase tables combined), 23-97110 HoH Dxb Update /FP-01 (single "Stage 1-3 Project Redesign" line)

---

### G. Information-gathering / report fee with contingency note (WAMI)

A distinct two-stage "Information Gathering / Report and Recommendations" schedule that appends a contingency caveat not seen elsewhere. One revision swaps in the "Post Contract Fees are estimates..." sentence.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 - Information Gathering: AED 483,360
> Stage 2 - Report and Recommendations: AED 586,080
> Total: AED 1,069,440
>
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.
>
> Note: As previously noted, we have very little information from which to base these fee estimates. Contingency amounts [at least 20%] should be discussed and included in the project budgets.

Used by: 25-97102 WAMI /FP-01, 25-97102 WAMI /FP-02, 25-97102 WAMI /FP-01 WDD Lighting notes (Post-Contract-estimates wording instead of variation-only)

---

### H. Design fee with optional acoustic scope add-on

Group A clause with a trailing optional-scope line offering acoustic design for a fixed lump sum. Otherwise identical to the standard clause.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 — Concept Design: AED 13,750
> Stage 2 — Schematic Design: AED 16,500
> Stage 3 — Detailed Design: AED 19,250
> Stage 4 — Tender Documentation: AED 5,500
> Total: AED 55,000
>
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.
>
> Optional Scope: Acoustic Design and Consultancy for a lump sum fee of AED 27,500.

Used by: 24-97107 Reserve Cut /FP-01, 24-97113 Level 63 /FP-01, 25-97101 Shanghai Tang /FP-01 + /FP-02, 25-97105 Shanghai Tang v2 /25-97101-FP-01

---

### I. Post-contract-estimate qualifier variants

Group A clause with the added sentence: "Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage." Some also add "Construction Supervision is estimated on N visits per month." Stage tables are otherwise standard.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Stage | Milestone | Fee (AED)
> 1 | Concept Design | 46,250
> 2 | Schematic Design | 55,500
> 3 | Detailed Design | 64,750
> 4 | Tender Documentation | 18,500
> Total | | 185,000
>
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Used by: 25-96601 /FP-01, 25-97107 Cove Boulevard /FP-01, 25-97108 RAK Beach District /FP-01

---

### J. Template/unpriced placeholder

A template proposal where stage names are present but fees read "[fee not stated in template]". Carries the standard variation notice. Useful as the blank starting form; not a client-facing priced clause.

Representative text:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 - Preliminaries: [fee not stated in template]
> Stage 2 - Concept Design: [fee not stated in template]
> [...]
> Before exceeding the above fee... emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Used by: 23-96606 KSA Pav Osaka /e-yy-cccnn-FP-01

---

**Recommended standard wording:**

The canonical clause is the Group A form: the applicability preamble, a clean colon-delimited stage list, then the variation notice. Use em-dash between stage label and milestone, colon before the fee, and the variation notice **after** the table. Reference the last client-approved stage in the notice (Stage 2 for full 4-5 stage proposals; Stage 1 for short or supervision-only proposals).

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 — Concept Design: AED 0
> Stage 2 — Schematic Design: AED 0
> Stage 3 — Detailed Design: AED 0
> Stage 4 — Tender Documentation: AED 0
> Total: AED 0
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Append, only when applicable:
- `Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.` (placed immediately after the preamble) when the proposal includes post-contract/supervision fees.
- `Optional Scope: Acoustic Design and Consultancy for a lump sum fee of AED 27,500.` (placed after the variation notice) when acoustics is offered as an add-on.
- A per-discipline breakdown matrix (Group C) for multi-discipline projects, after the staged total.

---

## Post Contract Fees

The "Post Contract Fees" clause type covers fees for the construction / execution phase of a project (the stages following design). Across the corpus it appears in three broad shapes: (A) a **boilerplate disclaimer** about estimates that nearly always opens the section, often paired with a "before exceeding" change-notification paragraph; (B) a **fee schedule** of post-contract stages with quantities, unit rates and an estimated total; and (C) **per-week involvement-tier rate cards** for projects where the post-contract scope cannot yet be quantified. Most documents combine the disclaimer (A) with a schedule (B). The variants below are grouped by these shapes; stage numbering, stage names, units, rates and totals are project-specific and reproduced verbatim.

---

### A. Standard estimate disclaimer (boilerplate)

The most common opening sentence. Two near-identical wordings exist that differ only by the noun "Post Contract Fees" vs "Post Design Fees" (the latter typically appears alongside the "These rates are applicable…" awarding clause). This boilerplate is embedded in almost every schedule variant below.

> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.

Variant noun "Post Design Fees" (otherwise identical):

> Post Design Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.

**Used by:** present (verbatim or as the opening line) in nearly every variant in this section — see the schedules below.

---

### B. "These rates are applicable…" awarding clause

Frequently prepended to the disclaimer. Establishes that quoted rates assume full scope award.

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.

One longer variant extends this with the estimate-disclaimer sentence folded in:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees. Stage quantities are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.

**Used by:** 22-97114 HoH (FP-01), 24-97104 HoH Programming (FP-01), 24-97101 HoH Supervision (FP-01, FP-02), 24-96606 JOH (FP-02)

---

### C. "Before exceeding the above fee…" change-notification clause

A recurring paragraph requiring written approval before exceeding the quoted fee. The approval-trigger stage number varies (after Stage 1, Stage 2 or Stage 3 has been approved).

> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

**Used by:** 22-97114 HoH (FP-01, "after Stage 2"), 23-97105 Tape Theatre (FP-01, "after Stage 2"), 24-97104 HoH Programming (FP-01, "after Stage 2"), 25-97103 Aljada (FP-02, "after Stage 2"), 24-96606 JOH (FP-02, "after Stage 1"), 24-97101 HoH Supervision (FP-01 "after Stage 3", FP-02 "after Stage 3")

---

### D. Full 8-stage Post-Design supervision schedule (HoH-family, task/day units)

The most detailed schedule format: numbered stages 1–8 covering tender review through DLP, each priced by task/day/visit unit. Used in the House-of-Hospitality (HoH) supervision proposals. Quantities and rates differ between projects.

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Post Design Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Stage 1 — Tender Return Review: task x 1 @ 25,000 = 25,000
> Stage 2 — C / P Management: day x 9 @ 4,500 = 40,500
> Stage 3 — Shop Drawing Review: task x 1 @ 65,000 = 65,000
> Stage 4 — Installation Supervision: day x 6 @ 7,500 = 45,000
> Stage 5 — Commissioning Validation: day x 8 @ 7,500 = 60,000
> Stage 6 — Programming: day x 7 @ 7,500 = 52,500
> Stage 7 — Handover / Training: day x 2 @ 4,500 = 9,000
> Stage 8 — Post Completion / DLP [Optional]: visit x 0 @ 10,500 = 0
> Estimated Total: 297,000
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Two sibling variants (24-97101 HoH Supervision) use the same 7-stage structure (no Stage 8) with higher unit rates and "after Stage 3" approval triggers — one with Estimated Total AED 628,500, one with AED 385,500. They differ only in quantities/rates and table layout (one uses `Stage N | … | Qty | Price | Est. Fee`, the other `Stage N - … : Qty @ rate = total`).

**Used by:** 22-97114 HoH (FP-01), 24-97101 HoH Supervision (FP-01: total AED 628,500), 24-97101 HoH Supervision (FP-02: total AED 385,500)

---

### E. Construction-supervision schedule (Stage 5/6 onward, month/visit units)

The dominant working format for most projects: a short schedule of construction-phase stages (Construction Supervision, Scene Setting, Handover, DLP), priced by month/visit/day, with an estimated total. Each project sets its own stage numbers, units, quantities and rates; an optional one-line note describes the supervision cadence (e.g. "1 visit per month", "2 visits per month", "48 hours per week"). Representative instance:

> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Construction Supervision is estimated on 1 visit per month with regular meetings and document reviews included.
>
> Stage 6 Construction Supervision: 36 months × AED 22,000 = AED 792,000
> Stage 6a Scene Setting: 10 visits × AED 10,000 = AED 100,000
> Stage 6b Handover: 3 weeks × AED 52,000 = AED 156,000
> Stage 7 DLP: 3 visits × AED 16,800 = AED 50,400
> Estimated Total: AED 1,098,400

This family is large; the members differ only in stage labels, cadence note, quantities, rates and total (and minor markup: `x` vs `×` vs `@`, dash style). Notable members:

- **23-96607 HOH KSA (FP-01):** Stages 5–8 (Submittal Review / Construction / Programming / Handover Supervision); total AED 1,582,000.
- **23-97106 Fountain Control (FP-01):** Stage 6 Construction / Stage 7 Show Programming / Stage 8 Training; "48 hours per week" cadence with a reduced-rate note for continuous blocks; total AED 1,174,400.
- **23-97111 Dubai Island Promenade (FP-01):** Stages 4–6 @ AED 8,400/visit; total AED 58,800. **(FP-02):** same stages @ AED 4,500; total AED 36,000.
- **24-97106 RAK Sled (FP-01):** Shop Drawing Review lot + Stages 5/5a/6/7; total AED 273,500.
- **24-97110 Masdar B-02 (FP-01):** Stage 5 Construction Supervision only, 12 visits @ AED 9,000 = AED 108,000; "2 visits per month by 2 consultants".
- **22-97113 MAF FEC (FP-01):** Stages 6/6a/6b/7 with optional DLP; total AED 193,200.
- **23-96606 KSA Pav Osaka (FP-01):** Stages 5/5a/6/7; unit rate for Stage 5 and total "not stated in template".
- **23-97105 Tape Theatre (FP-01):** Stages 5/5a/6/7 with all quantities/prices marked "TBD" (placeholder template).

**Used by:** 23-96607 HOH KSA (FP-01), 23-97101 Marina Island Financial District (FP-01), 23-97105 Tape Theatre (FP-01), 23-97106 Fountain Control (FP-01), 23-97111 Dubai Island Promenade (FP-01, FP-02), 24-97106 RAK Sled (FP-01), 24-97110 Masdar B-02 (FP-01), 22-97113 MAF FEC (FP-01), 23-96606 KSA Pav Osaka (FP-01) +1 more

---

### F. Aljada FG/G time-based block schedule (Stage 6/6a/7)

A distinct sub-family unique to project 25-97103 (Aljada). Four exact-dedup variants exist but they are the **same schedule** with identical figures (Stage 6 AED 2,196,480 / Stage 6a AED 2,310,000 / Stage 7 AED 140,000 / Total AED 4,646,480); they differ only in the header line ("Block F: [Specialist Lighting]" vs "Block G [Specialist Lighting and Home Automation]" vs no block header) and whether the disclaimer/change-notification boilerplate is prepended.

> Stage 6 Construction Supervision: 24 months x AED 91,520 = AED 2,196,480
> Stage 6a Scene Setting: 3 months x AED 770,000 = AED 2,310,000
> Stage 7 DLP: 4 visits x AED 35,000 = AED 140,000
> Estimated Total: AED 4,646,480

The fullest instance (FP-02) prepends the disclaimer + change-notification clause:

> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage. Construction Supervision is estimated on 1 staff member being on site per month.
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.
>
> Stage 6 Construction Supervision: 24 months x AED 91,520 = AED 2,196,480
> Stage 6a Scene Setting: 3 months x AED 770,000 = AED 2,310,000
> Stage 7 DLP: 4 visits x AED 35,000 = AED 140,000
> Estimated Total: AED 4,646,480

**Used by:** 25-97103 Aljada FG (FP-01, "Block F" + "Block G" headers), 25-97103 Aljada Block F + G (FP-02, with/without boilerplate)

---

### G. Programming / integration day-rate schedule (HoH Programming)

A short post-contract schedule scoped to off-site and on-site programming days, not general construction supervision.

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Stage 1 — Off Site Pre-Programming: 22 days @ AED 4,500/day = AED 99,000
> Stage 2 — On Site Programming / Integration: 28 days @ AED 9,000/day = AED 252,000
> Estimated Total: AED 351,000
>
> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage 2 has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

**Used by:** 24-97104 HoH Programming (FP-01)

---

### H. Multi-discipline / large monthly-retainer schedules

Project-specific schedules priced by month over long durations, often spanning multiple disciplines. These are bespoke and not part of a reusable family.

**24-96606 JOH (FP-02)** — system-engineering + ongoing-management + programming schedule (total AED 780,000), with the awarding clause and an "after Stage 1" change-notification clause:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees. Stage quantities are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Fee Schedule:
> - Preliminaries: Qty 1 — Included
> - Stage 1 - System Engineering: Qty 1 — AED 420,000
> - Stage 2 - Ongoing Management: Unit Price AED 15,000 x 18 months — AED 270,000
> - Stage 3 - Programming: Unit Price AED 30,000 x 3 — AED 90,000
> - Total: AED 780,000
>
> Before exceeding the mentioned fee … after Stage 1 has been approved …

**25-96601 (FP-01)** — Design Management / Project Management / Site Supervision / Programming over 13–29 months; total AED 2,500,000 (Stage 5 "No offer — N/A"):

> Stage 4 - Design Management: 13 months @ AED 40,000/month = AED 520,000
> Stage 5 - Project Management: 29 months - No offer - N/A
> Stage 6 - Site Supervision: 13 months @ AED 110,000/month = AED 1,430,000
> Stage 7 - Programming: 1 month @ AED 550,000/month = AED 550,000
>
> Estimated Total: AED 2,500,000

**Used by:** 24-96606 JOH (FP-02), 25-96601 (FP-01)

---

### I. Wynn A/V + Acoustics construction-administration (Stage 5)

A two-document pair for project 23-97102: one combines A/V and Acoustics in a single Stage 5 line, the other quotes the electrical (EL) discipline alone. These are the same clause split by discipline package.

> Stage | Milestone | A/V | Acoustics | Total Fee
> 5 | Construction Administration | 145,000 | 120,000 | 265,000

Single-discipline (EL) variant:

> Stage 5 — Construction Administration: USD 145,000
> Total Fee: USD 145,000

**Used by:** 23-97102 Wynn (AA-FP-01: A/V + Acoustics, total 265,000), 23-97102 Wynn (EL-FP-01: USD 145,000)

---

### J. Involvement-tier per-week rate card (WAMI, no total)

Used where post-contract scope cannot yet be sized: instead of a stage schedule, six involvement tiers (light/medium/heavy × off-site/on-site) are priced per week, with an explicit "insufficient information to estimate" note. Three exact-dedup variants exist for project 25-97102 (WAMI) that share identical tier rates and differ only in stage-prefix ("3a…" vs "Stage 3a…") and the placement/wording of the disclaimer and insufficient-information note.

> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Stage 3a - Light Off-Site Involvement: AED 10,680 per week
> Stage 3b - Medium Off-Site Involvement: AED 26,160 per week
> Stage 3c - Heavy Off-Site Involvement: AED 84,960 per week
> Stage 3d - Light On-Site Involvement: AED 85,360 per week
> Stage 3e - Medium On-Site Involvement: AED 135,560 per week
> Stage 3f - Heavy On-Site Involvement: AED 185,760 per week
>
> Please note that we do not currently have sufficient information to estimate these fees. Refer to Proposed Deployment section for indicative staffing of these configurations.

**Used by:** 25-97102 WAMI (FP-01, FP-01 WDD notes, FP-02)

---

### K. Deferred / placeholder

A single-line clause used when post-contract fees are not yet quantifiable at proposal time.

> TBC During Design Stages

**Used by:** 24-97111 Masdar OS-48 (FP-01)

---

**Recommended standard wording:**

For the common construction-supervision case, the canonical opening boilerplate is the estimate disclaimer plus (when full scope is being quoted) the awarding clause and the change-notification clause. Use this fixed template, then append the project-specific stage schedule:

> These rates are applicable if the complete scope described in this proposal is awarded to emittiv. Variations to the proposed scope may result in adjustments to the nominated fees.
>
> Post Contract Fees are estimates based on previous experiences with similar projects. These numbers should be used for budgetary purposes only and may alter during the execution stage.
>
> Construction Supervision is estimated on [cadence — e.g. 1 visit per month].
>
> Stage 6 — Construction Supervision: [qty] [unit] @ AED [rate] = AED [total]
> Stage 6a — Scene Setting: [qty] [unit] @ AED [rate] = AED [total]
> Stage 6b — Handover: [qty] [unit] @ AED [rate] = AED [total]
> Stage 7 — DLP [Optional]: [qty] [unit] @ AED [rate] = AED [total]
> Estimated Total: AED [sum]
>
> Before exceeding the above fee due to an increase in the scope of the project, increased attendance requirements, or due to changes and / or revisions after Stage [N] has been approved, emittiv will notify the Client in writing in order to secure written approval. In this event, the additional fee to be charged will be agreed at that time.

Notes for standardisation: prefer the noun **"Post Contract Fees"** over "Post Design Fees" (more common and clearer); standardise the multiplication glyph to `x` and the unit-rate separator to `@`; always include an **Estimated Total** line. Where scope cannot be quantified, use the WAMI involvement-tier rate card (variant J) or the "TBC During Design Stages" placeholder (variant K) rather than omitting the section.

---

## Payment Terms

The Payment Terms clause is the most heavily reused clause family in the corpus. Almost every proposal opens with the same "currency / VAT / mobilisation / payment window / back-to-back / retention" preamble, then optionally appends an "Additional Payment Terms" block (responsibility, bank account, stable-coin discount, daily billing rate, additional expenses, stage gating). Variants below are grouped by the meaningful structural pattern; trivial differences (bullet glyphs, line-break spacing, "Please let us know..." trailer, discount percentage, daily rate amount) are noted but not split out separately.

---

### 1. Short core preamble — 14-day terms, Post Contract Fees monthly

The base clause with monthly Post Contract Fee invoicing and time-sheets on request.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Post Contract Fees will be invoiced monthly. Internal time sheets will be provided on request.
> Back to back payments are not acceptable.
> Retention is not acceptable.

Used by: 23-97101 Marina Island Financial District (FP-01), 23-97105 Tape Theatre (FP-01-AH comments), 22-97113 MAF FEC (FP-01)

---

### 2. Short core preamble — 14-day terms, no Post Contract line

Same as variant 1 but omitting the "Post Contract Fees will be invoiced monthly" line.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.

Used by: 24-97112 Military Museum (FP-01), 25-97107 Cove Boulevard (FP-01), 26-96801 Rozana Muscat (FP-01), 24-97107 Reserve Cut (FP-01 — adds an optional-acoustic-fees line)

---

### 3. Short core preamble — 30-day terms

Identical to variant 2 except the payment window is 30 calendar days instead of 14. (Minor: one instance runs the VAT sentence onto the same line as "expenses".)

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 30 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.

Used by: 25-97102 WAMI (FP-01 WDD notes, FP-02), 25-97101 Shanghai Tang (FP-02)

---

### 4. Full clause — core preamble + Additional Payment Terms block (14-day)

The dominant "complete" pattern: the 14-day core preamble followed by the standard Additional Payment Terms block (client responsibility, bank account, stable-coin discount, daily rate, additional expenses, stage gating, revision charging). Discount percentage (1.5% / 2.5%), daily rate (AED 4,200 / 4,500 / 5,000 / 5,200 / 5,880), bullet glyph, and presence of "Post Contract Fees will be invoiced monthly" vary across instances; the "Please let us know if you are interested..." trailer is present in some. These are the same clause.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> Additional Payment Terms:
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED 4,500 per person per day, excluding travel, accommodation and ancillary costs.
> - Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 22-97115 Kids Activity Park AUH (FP-01, FP-02), 23-97105 Tape Theatre (FP-01), 23-97107 DAFZA Refurb (FP-01), 23-97108 Ciel Lobby (FP-01, FP-02), 23-97109 Ciel VIP (FP-01, FP-02), 23-97110 HoH Dxb Update (FP-01), 23-97111 Dubai Island Promenade (FP-01, FP-02), 24-97102 DMC 2A (FP-01), 24-97111 Masdar OS-48 (FP-01), 25-96601 (FP-01), 25-97103 Aljada FG (FP-01, FP-02), 25-97108 RAK Beach District (FP-01), 25-97109 Mapletree Warehouse (FP-01), 23-96606 KSA Pav Osaka (placeholder, has "xxx" edit markers), 24-97104 HoH Programming (FP-01, "On and Off Site Programming Fees"), 23-96602 Shoot the Chute (FP-01), 24-97108 Tamani Hotel (FP-01), 24-97113 Level 63 (FP-01) +6 more

---

### 5. Full clause — core preamble + Additional Payment Terms block (30-day)

Same complete pattern as variant 4 but with a 30-day payment window. Some instances add a "Post Contract Fees will be invoiced monthly on a pro-rata basis" or "Optional Acoustic Design..." line.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 30 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> Additional Payment Terms:
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be as indicated in the included rate card, excluding travel, accommodation and ancillary costs.
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting, review, or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 25-97102 WAMI (FP-01), 25-97105 Shanghai Tang v2 (FP-01, FP-05 with AED 5,200 rate), 25-97101 Shanghai Tang (FP-01)

---

### 6. Full clause with "Late or non-payment may result in suspension" line

Variant 4/5 pattern plus a suspension-of-works clause and "Post Contract Fees will be invoiced at the end of each month" (30-day terms). The HoH Supervision proposals are the clearest examples.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 30 calendar days after receipt of invoice.
> Post Contract Fees will be invoiced at the end of each month. Internal time sheets can be provided on request.
> Late or non-payment of invoices may result in the suspension of works, at the sole discretion of the Consultant.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount.
>
> Daily billing rates applicable to this project will be AED 4,500 per person per day, excluding travel, accommodation and ancillary costs.
> Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 22-97114 HoH (FP-01), 24-97101 HoH Supervision (FP-01, FP-02)

---

### 7. "Ongoing / Post Contract Management Fees invoiced monthly" + higher daily rate (JOH family)

The JOH proposals use the full clause with "Ongoing Management Fees will be invoiced monthly", an AED 4,800 daily rate, and (in FP-02) the variant phrasing "For any additional works beyond the agreed scope, daily billing rates..." with "Substantial revision of the current designs". A short "Ongoing Management Fees" preamble-only version also exists.

> All prices shown are in AED. All rates are exclusive of VAT and expenses. VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate. Mobilisation payments are to be received prior to any works commencing.
>
> Other payments will be required within 14 calendar days after receipt of invoice. Ongoing Management Fees will be invoiced monthly. Internal time sheets will be provided on request. Back to back payments are not acceptable. Retention is not acceptable.
>
> Additional Payment Terms:
>
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice. All payments shall be deposited by the Client into the bank account nominated on each invoice.
>
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED 4,800 per person per day, excluding travel, accommodation and ancillary costs.
> - Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately. Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin. Further revision of the designs and resubmissions, if required, will be charged at the nominated rates. Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 24-96606 JOH (FP-01, FP-01 comments, FP-02 [reworded "beyond agreed scope"], FP-01 comments response [preamble only])

---

### 8. EUR currency — VAT NOT applicable

For European clients: prices in EUR, "UAE VAT is NOT applicable" (or "VAT is NOT applicable"). Otherwise the same 14-day core preamble; the DockX/Jumpoline pair is the short form, Dammam Adventure World is the full form with EUR 1,050 rate and 1% discount.

> All prices shown are in EUR.
> UAE VAT is NOT applicable to this proposal.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.

Used by: 23-96604 DockX (FP-01), 23-96605 Jumpoline (FP-01), 23-96601 Dammam Adventure World (FP-01 — full form, EUR 1,050, 1% discount)

---

### 9. KSA / Saudi tax variants

Saudi-Arabia proposals swap UAE VAT for KSA Withholding Tax (or a "VAT / Saudi Withholding Tax" hybrid, or "VAT assumed applicable... if Saudi Withholding tax applies, calculated in place of UAE VAT"). Structure otherwise follows the full clause. Marasi Gate uses SAR currency and a submittal-triggered invoicing note.

> All prices shown are in AED.
> All rates are exclusive of taxes and expenses.
> KSA Withholding tax is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> Additional Payment Terms:
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice. All payments shall be deposited by the Client into the bank account nominated on each invoice.
>
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 1.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED 4,200 per person per day, excluding travel, accommodation and ancillary costs.
> • Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately. Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin. Further revision of the designs and resubmissions, if required, will be charged at the nominated rates. Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 23-96603 Khobar Grand Mosque (FP-01), 23-96607 HOH KSA (FP-01 — "VAT assumed... Saudi Withholding in place of VAT", monthly-in-advance), 24-96602 (FP-01 — "VAT / Saudi Withholding Tax"), 24-96603 Marasi Gate (FP-01 — SAR, submittal-triggered, draft/50% invoicing)

---

### 10. Conditional VAT — "Depending on final contractual arrangements" (Wynn family)

Wynn proposals open with conditional VAT wording and reference an appendix-driven remuneration schedule, monthly invoicing against work completed, and rates from a "Staff Rates section". The EL-FP-01 instance adds a 10% mobilisation deducted pro-rata and 45-day terms.

> Depending on final contractual arrangements, if VAT is applicable to this proposal it shall be added to all invoices at the prevailing rate.
>
> Mobilisation payments are to be received prior to any works commencing.
> Other payments will be required within 30 calendar days after receipt of invoice.
> Fees will be invoiced monthly against work completed. Internal time sheets will be provided on request as required.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> Additional Payment Terms:
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 1.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be as indicated in the following Staff Rates section, per person per day, excluding travel, accommodation and ancillary costs.
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 23-97102 Wynn (AA-FP-01), 23-97102 Wynn (EL-FP-01 — 45-day terms, 10% mobilisation pro-rata, 1% discount)

---

### 11. Pro-rata / monthly stage-completion invoicing variants

Several proposals replace the "Mobilisation + other payments" cadence with monthly pro-rata invoicing against stage completion.

> All prices shown are in AED. All rates are exclusive of VAT and expenses. VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
>
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 30 calendar days after receipt of invoice.
>
> Invoices shall be issued monthly, based on pro rata completion of each stage and department. Back to back payments are not acceptable. Retention is not acceptable.
>
> [followed by the standard Additional Payment Terms block — here with a 1.5% stable-coin discount and AED 4,500 rate]

Used by: 22-96601 Dammam Waterpark (FP-01 — monthly pro-rata by stage/department, 1.5% discount), 22-97113 MAF FEC (FP-02 — "Invoices shall be issued every month for works completed", 30-day, no back-to-back line), 23-97106 Fountain Control (FP-01 — "Post Contract Fees invoiced monthly on a pro-rata basis"), 23-97102 Wynn EL (see variant 10)

---

### 12. Submittal-triggered invoicing

Invoicing/payment is triggered by submittal of stage deliverables rather than a fixed cadence.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Submittal of stage deliverables shall be the trigger invoicing and payment for the relevant works.
> Back to back payments are not acceptable.
> Retention is not acceptable.

Used by: 24-96605 Observatory (FP-01), 24-96603 Marasi Gate (FP-01 — SAR, see variant 9)

---

### 13. Explicit milestone / phase payment schedule (lump-sum breakdown)

Three one-off proposals lead with a concrete payment-structure table or split before the standard Additional Payment Terms block. These are project-specific and should NOT be treated as boilerplate.

- **TPF Sound System (22-97111-FP-01):** a per-stage payment breakdown (Mobilisation 30% = AED 50,000; Stage 1–4 splits; Total AED 166,667), then "Any Post Contract Fees will be invoiced monthly", then the standard Additional Payment Terms (AED 4,200 daily rate, "approved in writing" without "notified and"). A signed sibling (22-97111-FP-01 - signed) carries only the Additional Payment Terms block without the milestone table.
- **The View Experience Centre (22-97112-FP-01):** "50% upon confirmation of order / 50% upon completion of works", then Additional Payment Terms with a graded staff rate card (Senior Engineer AED 5,000 / Assistant AED 2,250 / Wireman AED 1,500) and "A senior engineer is required to be on site with any other staff." The unsigned and signed copies differ only in bullet glyph (• vs —).

Used by: 22-97111 TPF Sound System (FP-01, FP-01 signed), 22-97112 The View Exp Ctr (FP-01, FP-01 signed)

---

### 14. Back-to-back payment basis accepted (Dammam Water Park)

Unusually, this proposal *accepts* a back-to-back basis ("It is understood that this project shall be undertaken on a Back to Back payment basis") with monthly invoicing for work to date — the opposite of the standard "Back to back payments are not acceptable" line. Project-specific.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> It is anticipated that invoices will be issued on a monthly basis for work completed to date.
> It is understood that this project shall be undertaken on a Back to Back payment basis.
> Retention is not acceptable.
> [followed by the standard Additional Payment Terms block, 1.5% discount, AED 4,500 rate]

Used by: 24-96601 Dammam Water Park (FP-01)

---

### 15. "Monthly pro-rata payments open for discussion" (MAF MOE FEC family)

The MAF MOE FEC proposals keep the 14-day core but drop the back-to-back line and add a soft note that monthly pro-rata payments are open for discussion based on the previous project.

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Retention is not acceptable.
>
> Note: Based on the previous project, monthly pro-rata payments will be open for discussion.

Used by: 25-97106 MAF MOE FEC v3 (FP-01 [+ full Additional Payment Terms block], FP-02 [preamble only]), 24-97109 MOE FEC v2 (FP-01 — adds the full block)

---

### 16. Additional-Payment-Terms-only fragments (no preamble)

Some documents store only the trailing "Additional Payment Terms" block — the responsibility/bank-account/stable-coin/daily-rate paragraph without the currency/VAT/mobilisation preamble. These are continuation fragments of the full clause (variant 4/5), not standalone clauses.

> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
> Daily billing rates applicable to this project will be [rate], excluding travel, accommodation and ancillary costs.
> [...standard expenses / stage-gating / revision lines...]

Used by: 25-97102 WAMI (FP-01 WDD notes, FP-02 — EUR 1,250 / "as indicated in rate card"), 23-96604 DockX (FP-01 — EUR 1,250), 24-97107 Reserve Cut (FP-01 — no stable-coin line), 26-96801 Rozana Muscat (FP-01 — "nominated stable coin currencies", AED 5,200)

---

**Recommended standard wording:**

The clear canonical version is the **full clause with the 14-day core preamble + Additional Payment Terms block** (variant 4) — it is by far the most-used pattern and covers the standard AED/VAT case. Recommended baseline:

> All prices shown are in AED.
> All rates are exclusive of VAT and expenses.
> VAT is applicable to this proposal and shall be added to all invoices at the prevailing rate.
> Mobilisation payments are to be received prior to any works commencing. Other payments will be required within 14 calendar days after receipt of invoice.
> Post Contract Fees will be invoiced monthly. Internal time sheets will be provided on request.
> Back to back payments are not acceptable.
> Retention is not acceptable.
>
> Additional Payment Terms:
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED 4,500 per person per day, excluding travel, accommodation and ancillary costs.
> - Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Parameterise these fields per project rather than maintaining separate clauses:
- **Currency** — AED (default), EUR (European clients, with "VAT is NOT applicable"), SAR (KSA).
- **Tax line** — UAE VAT (default), KSA Withholding Tax, or the hybrid "VAT / Saudi Withholding Tax".
- **Payment window** — 14 days (default) or 30 days (45 for Wynn-style appendix contracts).
- **Stable-coin discount** — 1% / 1.5% / 2.5% (2.5% is most common).
- **Daily rate** — AED 4,200 / 4,500 / 5,000 / 5,200 / 5,880 (or graded staff rate card, or "see rate card").
- **Invoicing cadence line** — "Post Contract Fees invoiced monthly", "Ongoing Management Fees invoiced monthly", monthly pro-rata by stage, or submittal-triggered.

Keep the milestone/lump-sum schedules (variant 13), the back-to-back-accepted case (variant 14), and the conditional-VAT appendix contracts (variant 10) as project-specific overrides, not part of the standard library.

---

## Payment Schedule

The Payment Schedule clause sets out the staged fee breakdown and the amount payable at each milestone. Across the corpus the same underlying structure recurs with cosmetic variation: a Mobilisation advance (a percentage of the design-stage fees, most often 30%), followed by per-stage Fee and Payment columns, then a Total. The variation is almost entirely formatting (bullet list vs. pipe table vs. inline "Fee X / Payment Y") and the specific stage names/amounts per project. Below the variants are grouped by structural pattern rather than by literal text, since the amounts differ on every project.

---

### Variant A — Mobilisation + design stages, "Fee / Payment" per stage (the dominant form)

The most common pattern. Mobilisation as a percentage of design-stage fees (30% in most, 25%/20%/15%/14%/10% in others), then each stage lists its Fee and the Payment due, ending with a Total. Delimiters vary trivially (`—`, `/`, `|`, `:`, `()`); these are the same clause.

> Mobilisation: 30% — AED 57,000
> Stage 1 — Preliminaries: AED 19,000 (payment AED 4,750)
> Stage 2 — Schematic Design: AED 66,500 (payment AED 52,250)
> Stage 3 — Detailed Design: AED 85,500 (payment AED 71,250)
> Stage 4 — Tender Documentation: AED 19,000 (payment AED 4,750)
> Total: AED 190,000

Used by: 22-96601 Dammam Waterpark, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-96604 DockX, 23-96607 HOH KSA, 23-97101 Marina Island, 23-97105 Tape Theatre (FP-01 + AH-comments), 23-97108 Ciel Lobby (FP-01 + FP-02), 23-97111 Dubai Island Promenade (FP-01 + FP-02), 24-96601 Dammam Water Park, 24-96602, 24-97102 DMC 2A, 24-97110 Masdar B-02, 24-97111 Masdar OS-48, 24-97112 Military Museum, 25-96601, 25-97101 Shanghai Tang, 25-97104 MAF MiCC Kids Redesign, 25-97105 Shanghai Tang v2 (both), 25-97106 MAF MOE FEC v3, 25-97109 Mapletree, 26-96801 Rozana Muscat, 24-97107 Reserve Cut, 24-97113 Level 63 +6 more

---

### Variant B — Same clause rendered as a pipe / markdown table

Identical structure to Variant A (Mobilisation %, per-stage Fee + Payment, Total) but laid out as a table with columns `Stage | Milestone | Fee | Payment`. Mobilisation often appears in the dash row with the % in the Payment column. The only difference from Variant A is table formatting.

> Stage | Milestone | Fee (AED) | Payment (AED)
> — | Mobilisation | — | 30% = 55,500
> 1 | Concept Design | 46,250 | 32,375
> 2 | Schematic Design | 55,500 | 41,625
> 3 | Detailed Design | 64,750 | 50,875
> 4 | Tender Documentation | 18,500 | 4,625
> Total | | 185,000 |

Used by: 23-97102 Wynn (AA-FP-01), 23-97107 DAFZA Refurb, 24-97108 Tamani Hotel, 24-97109 MOE FEC v2, 25-97103 Aljada FG (both table forms), 25-97107 Cove Boulevard, 25-97108 RAK Beach District, 23-97109 Ciel VIP (FP-02), 24-97106 RAK Sled, 25-97101 Shanghai Tang (FP-02), 25-97106 MAF MOE FEC v3 (FP-02)

---

### Variant C — Design phase + optional/post-contract works split

Variant A's design-phase table followed by a separately-totalled "Optional Works" or "Post Contract" block, or a "Review" phase plus a "Design Phase" block. Same per-stage Fee/Payment mechanics; the distinguishing feature is the second totalled section.

> Design Phase:
> Mobilisation: 30% = AED 412,500
> Stage 1 — Concept Design: AED 425,000
> Stage 2 — Schematic Design: AED 987,500
> Stage 3 — IFT Documents: AED 237,500
> Total: AED 2,062,500
>
> Optional Works:
> Stage 4 — Detailed Design: Fee AED 1,312,500 | Payment AED 1,312,500
> Stage 5 — IFC Documents: Fee AED 375,000 | Payment AED 375,000
> Total: AED 1,687,500

Used by: 22-96601 Dammam Waterpark (e-22-96601-FP-01), 24-96605 Observatory (Review + Design Phase), 22-97113 MAF FEC (Preliminaries/Mobilisation form)

---

### Variant D — Supervision / construction-phase schedule (50/50 + invoiced monthly)

A distinct supervision-services schedule. No design-stage mobilisation %; instead each stage carries a payment term of either "50/50" (50% advance, 50% on substantial completion) or "invoiced monthly". Always closes with the explanatory note on 50/50 terms.

> Stage 1 — Tender Return Review: AED 35,000 — 50/50
> Stage 2 — C / P Management: AED 40,500 — invoiced monthly
> Stage 3 — Shop Drawing Review: AED 85,000 — 50/50
> Stage 4 — Installation Supervision: AED 81,000 — invoiced monthly
> Stage 5 — Commissioning Validation: AED 90,000 — invoiced monthly
> Stage 6 — Programming Supervision: AED 45,000 — invoiced monthly
> Stage 7 — Handover / Training: AED 9,000 — invoiced monthly
> Total: AED 628,500
>
> note: 50/50 payment terms indicate 50% in advance and 50% upon substantial completion. Invoices for advance payments will be generated ahead of time to allow for processing by the Client.

Used by: 22-97114 HoH, 24-97101 HoH Supervision (FP-01 + FP-02), 23-97110 HoH Dxb Update (Mobilisation 50% + submittal milestones variant)

---

### Variant E — Information-gathering / validation phase + ongoing T&M support

A two-stage gathering phase (Information Gathering + Report) plus an open-ended "On-going Support and Validation" phase invoiced monthly on Time & Material, with the minimum-weekly-involvement note. Recurs verbatim across the WAMI proposals.

> Mobilisation: 30% = AED 320,832
> Stage 1 - Information Gathering: Fee AED 483,360 | Payment AED 322,944
> Stage 2 - Report and Recommendations: Fee AED 586,080 | Payment AED 425,664
> Total: AED 1,069,440
>
> On-going Support and Validation Phase Fees: To be invoiced monthly on a Time and Material basis. Internal time sheets will be provided on request.
> Note: In order to maintain an active involvement in the Project, a minimum of the 3a [Light Off-Site Involvement] must be confirmed for each week. This is intended to ensure continuity of the developed knowledge base and prevent the need to re-mobilise after extended breaks.

Used by: 25-97102 WAMI (FP-01 "WDD Lighting notes", FP-01, FP-02) — count 4 across variants. Trivial differences: one rendering punctuates "= AED 320,832 (due prior to works commencing)" and splits the on-going block onto separate lines; otherwise identical.

---

### Variant F — Engineering + ongoing management + programming (JOH multi-phase)

A bespoke multi-phase schedule: Stage 1 engineering mobilisation, four monthly system-engineering sub-stages, Stage 2 "Ongoing Management - invoiced monthly" (18 × 15,000), then a separate programming mobilisation + completion. Appears in four near-identical renderings (bullet, pipe table, "Stage X.Y" inline) — same clause, formatting only.

> Stage 1 — Engineering Mobilisation: 30% = AED 126,000
> Stage 1.1 — System Engineering month 1: AED 105,000 billed / AED 73,500 payment
> Stage 1.2 — System Engineering month 2: AED 105,000 billed / AED 73,500 payment
> Stage 1.3 — System Engineering month 3: AED 105,000 billed / AED 73,500 payment
> Stage 1.4 — System Engineering month 4: AED 105,000 billed / AED 73,500 payment
> Stage 2 — Ongoing Management - invoiced monthly: 18 x AED 15,000 = AED 270,000
> Stage 3 — Programming Mobilisation: 30% = AED 30,000
> Stage 3.1 — Programming Substantial Completion: AED 90,000 billed / AED 60,000 payment
> Total: AED 780,000

Used by: 24-96606 JOH (FP-01 base, FP-01 "comments", FP-01 "comments response", FP-02) — all four are the same schedule with different delimiters.

---

### Variant G — Discount-conditional programming schedule

A short programming schedule (mobilisation + off-site pre-programming + on-site programming) carrying a conditional discount tied to a companion supervision proposal. The discount paragraph is the distinguishing content.

> Mobilisation: 14% = AED 49,000
> Stage 1 — Off Site Pre-Programming: Fee AED 99,000 / Payment AED 50,000
> Stage 2 — On Site Programming / Integration: Fee AED 252,000 / Payment AED 252,000
> Total: AED 351,000
>
> A Discount of 45,000 AED can be applied to this fee if our Construction Supervision Proposal [e-24-97101-FP-02] is approved and ordered. This equates to the Programming Supervision fees shown in that proposal, which would not be required if emittiv are undertaking the Programming as well. If the Construction Supervision proposal has been ordered and a PO has been issued, a PO for this Proposal in the amount of 306,000 AED [excl. VAT] will be considered complete and acceptable.

Used by: 24-97104 HoH Programming (e-24-97104-FP-01)

---

### Variant H — Simple milestone table (no per-stage fee/payment split)

A reduced form: just Milestone → Payment, no separate Fee column. Used on smaller/fixed-price proposals.

> Milestone | Payment (AED)
> 30% Mobilisation | 36,000
> Concept Design Submittal | 48,000
> 50% Schematic Design Submittal | 36,000
> Total | 120,000

Used by: 22-97115 Kids Activity Park AUH (FP-01 milestone table, FP-02 mobilisation+submittal), 23-96601 Dammam Adv World (Stage 1 mobilisation/balance form), 24-97105 MAF MiCC Kids

---

### Variant I — Extended terms appended to the schedule (Jumpoline)

Variant A/B's pipe-table schedule followed by an extensive "Additional Payment Terms" block: client payment responsibility, nominated bank account, stable-coin (USDT/USDC) 1.5% discount offer, daily billing rates, expense handling, and stage-gating language. The appended terms — not the table — are what set this apart.

> Mobilisation | 30% | EUR 12,000
> Stage 1 | Concept Design | EUR 8,000 | EUR 4,000
> Stage 2 | Schematic Design | EUR 16,000 | EUR 12,000
> Stage 3 | Detailed Design | EUR 16,000 | EUR 12,000
> Total | | EUR 40,000
>
> Additional Payment Terms
> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice.
> All payments shall be deposited by the Client into the bank account nominated on each invoice.
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 1.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be EUR 1,250 per person per day, excluding travel, accommodation and ancillary costs.
> - Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin.
> Further revision of the designs and resubmissions, if required, will be charged at the nominated rates.
> Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

Used by: 23-96605 Jumpoline (e-23-96605-FP-01)

---

### Variant J — Blank template (placeholder, no amounts)

The empty boilerplate used as a starting template, with stage names but "[payment not stated]" / "[not stated in template]" placeholders. Not a real clause instance — listed for completeness.

> Mobilisation: 30%
> Stage 1 - Preliminaries: [payment not stated]
> Stage 2 - Concept Design: [payment not stated]
> Stage 3 - Developed Design: [payment not stated]
> Stage 4 - Technical Design: [payment not stated]
> Stage 4a - Tender Documentation: [payment not stated]
> Stage 4b - Tender Return Review: [payment not stated]
>
> Total: [not stated in template]

Used by: 23-96606 KSA Pav Osaka (e-yy-cccnn-FP-01 — placeholder/template file)

---

**Recommended standard wording:**

Variant A is the canonical design-phase payment schedule and should be the default. Present it as a clean per-stage table so the Fee and Payment columns read consistently regardless of project:

> | Stage | Milestone | Fee (AED) | Payment (AED) |
> |---|---|---|---|
> | — | Mobilisation | — | 30% = {mobilisation_amount} |
> | 1 | Concept Design | {fee} | {payment} |
> | 2 | Schematic Design | {fee} | {payment} |
> | 3 | Detailed Design | {fee} | {payment} |
> | 4 | Tender Documentation | {fee} | {payment} |
> | **Total** | | **{total_fee}** | |
>
> Documentation and payment approvals are required before work on the next stage will begin.

Notes for the standard:
- **Mobilisation** is a percentage of total design-stage fees (default 30%; use 25%/20%/15%/10% where the project warrants a lower advance), due prior to works commencing.
- Keep the **"Documentation and payment approvals are required before work on the next stage will begin"** sentence — it recurs across many variants and gates stage payments.
- For **supervision/construction** engagements use Variant D's structure instead (per-stage "50/50" or "invoiced monthly" terms plus the 50/50 explanatory note), not the design-phase table.
- For **ongoing/retainer** phases (validation, ongoing management, post-contract) append a single line: "{Phase} to be invoiced monthly" (Variant E/F), and only add the T&M time-sheet note where a Time & Material basis genuinely applies.
- Currency placeholder ({AED}/{EUR}/{USD}/{SAR}) follows the project; the structure is currency-agnostic.
- Treat the stable-coin discount, daily-rate card, and expense-handling paragraphs (Variant I) as **optional add-on terms**, not part of the base schedule — include only when the project requires them.

---

## Reimbursables & Provisional Sums

This section consolidates clauses covering daily billing rates, additional-expense handling, payment terms, reimbursable cost projections (travel / accommodation / meals), and staff rate cards. The corpus splits into two broad families: (A) the **standard daily-rate + payment-terms paragraph** that appears across most proposals with only the headline daily rate and discount percentage varying, and (B) **tabular reimbursable projections and rate cards** that are project-specific.

---

### Variant A1 — Standard daily-rate + payment-terms clause (full form)

The most common clause. Combines payment responsibility, stable-coin discount, headline daily rate, additional-expense handling, and stage-gate/revision terms. The only material differences between instances are the **headline daily rate** (AED 4,200 / 4,500 / 4,750 / 4,800 / 5,000) and the **stable-coin discount** (1.5% or 2.5%), plus trivial paragraph ordering and bullet-glyph differences (`-` vs `•`).

> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice. All payments shall be deposited by the Client into the bank account nominated on each invoice.
>
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED 4,500 per person per day, excluding travel, accommodation and ancillary costs.
> - Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately. Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin. Further revision of the designs and resubmissions, if required, will be charged at the nominated rates. Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

**Rate / discount variations observed within this clause family:**
- AED 4,200 / 2.5% — 23-97105-FP-01, 22-97113-FP-01
- AED 4,200 / 1.5% — 23-97106-FP-01
- AED 4,200 (no payment-terms para) — 23-97101-FP-01
- AED 4,500 / 2.5% — 25-97107-FP-01, 25-97101-FP-02, 24-97107-FP-01, 24-96605-FP-01
- AED 4,500 / 1.5% — 24-96602-FP-01
- AED 4,750 / 2.5% — 24-97112-FP-01
- AED 4,800 / 2.5% — 24-96606-FP-01
- AED 5,000 / 2.5% — 25-97106-FP-02

Used by: 25-97107-FP-01, 25-97101-FP-02, 23-97101-FP-01, 23-97105-FP-01, 23-97106-FP-01, 24-96602-FP-01, 24-97107-FP-01, 24-97112-FP-01, 24-96606-FP-01, 25-97106-FP-02, 22-97113-FP-01, 24-96605-FP-01

---

### Variant A2 — Daily-rate clause, abbreviated (no stage-gate / payment-responsibility paragraphs)

A shortened version of A1 keeping the daily rate, additional-expense handling, and stable-coin discount, but dropping the payment-responsibility and stage-gate/revision paragraphs.

> Daily billing rates applicable to this project will be AED 4,500 per person per day, excluding travel, accommodation and ancillary costs.
> • Daily rates for in house staff are available on our rate card. Please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.

Used by: 24-97107-FP-01

---

### Variant A3 — Daily-rate clause with out-of-country travel note (no payment terms)

A minimal daily-rate variant that omits payment terms and rate-card reference, and adds an explicit out-of-country travel exclusion.

> Daily billing rates applicable to this project will be AED 5,880 per person per day, excluding travel, accommodation and ancillary costs.
> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.

Used by: 25-97103-FP-02

---

### Variant A4 — Rate-card-referenced daily-rate clause (travel-time + per-diem terms)

A daily-rate clause that defers to an included rate card rather than naming a headline rate, and adds travel-time and per-diem conditions.

> Additional expenses related to the Project will be charged and invoiced separately.
> Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
> Daily billing rates applicable to this project will be as indicated in the included rate card, excluding travel, accommodation and ancillary costs.
> Travel time shall be charged outside of Dubai city. Consecutive multi-day visits may require additional accommodation and per diem fees. This shall be agreed with the Client upon request.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.

Used by: 25-97102-FP-01

---

### Variant B1 — Reimbursable cost projection table (travel / accommodation / meals)

Project-specific projections of reimbursable expenses, broken down per named team member across flights, weekly accommodation allowance, and daily meals. Two instances differ in scope and totals; both share the same preamble and line-item structure.

> Prices are in AED. Where applicable, VAT shall be added at the prevailing rate.
> This projection allows for 4 x 1 week site visits / workshops.
>
> Item | Martin | Andrew H | Michael | Neeskens | John | Andrew N | Total Qty | Unit Price | Total Price
> Flights | 0 | 0 | 0 | 0 | 4 | 0 | 4 | 13,600 | 54,400
> Accom [weekly allowance] | 4 | 4 | 4 | 4 | 4 | 4 | 24 | 3,600 | 86,400
> Meals [Daily] | 20 | 20 | 20 | 20 | 20 | 20 | 120 | 300 | 36,000
> Total | | | | | | | | | 176,800

A second instance (23-97102-EL-FP-01) extends the same structure with an additional assistant-designer visit allowance, a stated acceptance of the Employer's Travel Reimbursable policy, a different team roster (Total AED 186,000), and an appended staff rate card (see B3).

Used by: 23-97102-AA-FP-01, 23-97102-EL-FP-01

---

### Variant B2 — Current staff rate card (Jan 2025, full grade matrix)

The standard detailed rate card as of January 2025: per-grade hourly/daily/weekly/monthly rates across 16 roles, with time definitions and validity terms. Two near-identical instances differ only in trivial punctuation ("Prices shown in AED" vs "Prices are shown in AED"; em-/parenthetical phrasing of the week definition).

> Current as of January 2025. Prices shown in AED. Where applicable, VAT shall be added at the prevailing rate. Site Visits incur a minimum full day fee. Travel time shall be charged outside of Dubai city. Consecutive multi-day visits may require additional accommodation and per diem fees. This shall be agreed with the Client upon request.
>
> Time definitions: 1 day = 8 hrs | 1 week = 5 working days - Monday to Friday | 1 month = 22 working days.
>
> Partner: AED 1,235/hr | 9,880/day | 49,400/week | 217,360/month
> Project Director: AED 880/hr | 7,040/day | 35,200/week | 154,880/month
> Design Manager: AED 770/hr | 6,160/day | 30,800/week | 135,520/month
> Project Manager: AED 795/hr | 6,360/day | 31,800/week | 139,920/month
> Senior Consultant: AED 680/hr | 5,440/day | 27,200/week | 119,680/month
> Consultant: AED 520/hr | 4,160/day | 20,800/week | 91,520/month
> BIM Manager: AED 685/hr | 5,480/day | 27,400/week | 120,560/month
> BIM Operator: AED 420/hr | 3,360/day | 16,800/week | 73,920/month
> 3D Digital Modeller: AED 455/hr | 3,640/day | 18,200/week | 80,080/month
> 2D Drafting: AED 390/hr | 3,120/day | 15,600/week | 68,640/month
> 2D Visualisation: AED 655/hr | 5,240/day | 26,200/week | 115,280/month
> Site Manager: AED 825/hr | 6,600/day | 33,000/week | 145,200/month
> Site Consultant: AED 735/hr | 5,880/day | 29,400/week | 129,360/month
> Site Technician: AED 520/hr | 4,160/day | 20,800/week | 91,520/month
> Coordinator: AED 405/hr | 3,240/day | 16,200/week | 71,280/month
> Administrative Support: AED 270/hr | 2,160/day | 10,800/week | 47,520/month
>
> emittiv confirms that these prices shall remain in effect until the advertised project completion date of Q2 2027 ["early 2027" as listed on the official website]. Any extension of time beyond this may be subject to revised fees.

Used by: 25-97102-FP-01 (WDD Lighting notes), 25-97102-FP-02

---

### Variant B3 — Legacy staff rate card (Jan 2023, alternate grade set)

An earlier rate card embedded in the Wynn EL projection. Different role taxonomy (Design Manager, Package Designer, Design Assistant, 2D/3D/BIM/CAD operators, Admin, Technical Manager/Supervisor) and 2023 pricing. Retained for historical reference only.

> Staff Rates (Current as of January 2023, prices in AED; VAT applicable at prevailing rate; rates applicable for the duration of the project):
> Design Manager: Half Day 2,600 | Day 4,200 | Week 19,350
> Package Designer: Half Day 2,100 | Day 3,500 | Week 15,750 | Month 55,150
> Design Assistant: Hour 400 | Half Day 1,500 | Day 2,500 | Week 11,250 | Month 39,400
> 2D Visualiser: Hour 250 | Half Day 900 | Day 1,500 | Week 6,750 | Month 23,650
> 3D Operator: Hour 300 | Half Day 1,050 | Day 1,750 | Week 7,900 | Month 27,650
> BIM Operator: Hour 300 | Half Day 1,000 | Day 1,600 | Week 6,800 | Month 23,700
> CAD Operator: Hour 250 | Half Day 750 | Day 1,250 | Week 5,650 | Month 19,800
> Admin: Hour 150 | Half Day 450 | Day 750 | Week 3,400 | Month 11,900
> Technical Manager: Half Day 2,600 | Day 4,200 | Week 19,350 | Month 59,150
> Technical Supervisor: Half Day 1,650 | Day 3,750 | Week 12,400 | Month 43,400

Used by: 23-97102-EL-FP-01

---

**Recommended standard wording:**

For the daily-rate + payment-terms clause (Family A), Variant **A1** is the canonical full form. Use it with the project's actual headline daily rate and the current standard stable-coin discount (the 2.5% rate is the modern default; the 1.5% instances are older). Recommended canonical text:

> It shall be the responsibility of the Client to deliver payment to the Consultant on or before the due date shown on each invoice. All payments shall be deposited by the Client into the bank account nominated on each invoice.
>
> emittiv accepts payment in stable coin currencies [USDT and USDC.] Payments made by this method are eligible for a 2.5% discount. Please let us know if you are interested in pursuing this option.
>
> Daily billing rates applicable to this project will be AED [RATE] per person per day, excluding travel, accommodation and ancillary costs. Daily rates for in-house staff are available on our rate card; please request a copy of this document if required.
>
> Additional expenses related to the Project will be charged and invoiced separately. Any additional fees or expenses shall be notified and approved in writing by the Client prior to emittiv proceeding.
>
> Documentation and payment approvals are required before work on the next stage will begin. Further revision of the designs and resubmissions, if required, will be charged at the nominated rates. Any additional consulting or design services performed beyond the scope of this proposal will be invoiced on the basis of pre-determined daily billing rates or on the basis of a mutually agreed fixed fee.

For the rate card (Family B), Variant **B2** (the January 2025 16-grade matrix) is the current standard; B3 is superseded. Reimbursable projection tables (B1) are inherently project-specific and should be generated per proposal rather than standardised, though the preamble ("Prices are in AED. Where applicable, VAT shall be added at the prevailing rate.") and the Flights / Accommodation / Meals line-item structure are the reusable skeleton.

---

## Assumptions

The "Assumptions" clause type opens nearly every proposal with the same framing sentence and then a project-specific bulleted list. The standard preamble appears in two interchangeable forms (one-line vs. two-line break after the first sentence) and with either `-` hyphen or `•` bullet markers; these are typographic variants of the same wording. Below, variants are grouped by the substantive content of their assumption lists.

---

### 1. Standard preamble only (placeholder / empty body)

> The following assumptions have been made in the preparation of this proposal.
> If any of the following are incorrect, please advise us and we will prepare an updated version.
> • xxx

The bare template with a placeholder line (`xxx` or `- xxx`). Two docs carry it unfilled.

Used by: 23-96606 KSA Pav Osaka (e-yy-cccnn-FP-01), 23-97105 Tape Theatre (e-23-97105-FP-01-AH comments)

---

### 2. Designer/engineer coordination boundary (long form)

> - This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> - Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> - emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> - It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> - emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> - All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> - emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> - The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

The fullest disclaimer block establishing emittiv's role and limits of liability. Appears with `-` and with `•` markers (otherwise identical).

Used by: 23-97105 Tape Theatre (e-23-97105-FP-01), 22-97113 MAF FEC (22-97113-FP-02), 23-97108 Ciel Lobby (e-23-97108-FP-01), 22-97113 MAF FEC (22-97113-FP-01 — `•` variant)

---

### 3. Designer/engineer coordination boundary (short form, with site-visit charge)

> • This proposal assumes that the Architect and Electrical Engineer are to maintain their usual role of co-ordination of the design and supervision of the system installation on site.
> • emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> • All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> • emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> • The exact method of fastening or connection must be verified by the Architect / Structural Engineer.
> • In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

A condensed sibling of variant 2: drops the "information provided is for use by…", "not to be issued direct to the Contractor", and "CAD files ready to use" lines, and adds the site-visit-additional-charge line. A two-line minimal version (just the first and last bullets) was used on one MAF FEC doc.

Used by: 22-97114 HoH (e-22-97114-FP-01), 24-97101 HoH Supervision (e-24-97101-FP-01), 22-97113 MAF FEC (22-97113-FP-01 — first + last bullet only)

---

### 4. Existing cable infrastructure (retrofit/upgrade projects)

> The following assumptions have been made in the preparation of this proposal. If any of the following are incorrect, please advise us and we will prepare an updated version.
> - It is assumed that all cable infrastructure is functional and fit for purpose
> - This proposal does not provide for replacing or re-pulling of any cables
> - The Client will provide complete as built system drawings and access to all appropriate equipment as required

For projects working over existing installations. One View Experience Centre doc merges the first two points onto a single bulleted line (`• … fit for purpose — This proposal does not provide for replacing…`); content is otherwise identical.

Used by: 22-97112 The View Exp Ctr (e-22-97112-FP-01 - signed), 22-97112 The View Exp Ctr (e-22-97112-FP-01 — merged-bullet variant)

---

### 5. Infrastructure and structural support provided by Client

> The following assumptions have been made in the preparation of this proposal. If any of the following are incorrect, please advise us and we will prepare an updated version.
>
> - The Client will provide appropriate infrastructure to each installation location
> - The Client will provide structural guidance for proposed mounting locations as required

Used by: 22-97111 TPF Sound System (e-22-97111-FP-01 - signed), 22-97111 TPF Sound System (e-22-97111-FP-01)

---

### 6. No interaction with attractions

> The following assumptions have been made in the preparation of this proposal.
> If any of the following are incorrect, please advise us and we will prepare an updated version.
> - No interfaces or interaction shall be required with attractions.

Used by: 23-96604 DockX (e-23-96604-FP-01), 23-96605 Jumpoline (e-23-96605-FP-01)

---

### 7. Tie-in to site-wide BMS/GRMS (scope limited to Concept/Schematic)

> The following assumptions have been made in the preparation of this proposal.
>
> If any of the following are incorrect, please advise us and we will prepare an updated version.
>
> - Given that these Suites form part of a larger hotel operation, it is assumed that the sensory systems shall be required to tie in to the site wide BMS / GRMS. As a result, our proposal is limited to Concept and Schematic design stages, to allow the systems to be incorporated into the master system by others.

Used by: 23-97109 Ciel VIP (e-23-97109-FP-01), 23-97109 Ciel VIP (e-23-97109-FP-02)

---

### 8. MPME / specialist-contractor engagement (JOH)

> The following assumptions have been made in the preparation of this proposal. If any of the following are incorrect, please advise us and we will prepare an updated version.
>
> - The End Client has or shall engage a specialist contractor for rigging systems who will be responsible for all required mounting positions - movable and static.
> - MPME has or shall engage a certified ETC systems specialist for the project.
> - MPME has or shall engage suitably capable MA programmers for the project.
> - MPME has staff capable of undertaking 2D works in Revit to assist in timely delivery of shop drawings. emittiv can provide with basic training in this.
> - emittiv shall coordinate and provide any required information to these specialists as part of our works, however they will ultimately be responsible for their specific systems.

Project-specific to JOH. Identical content across four docs, varying only by `-`/`•` markers and one/two-line preamble.

Used by: 24-96606 JOH (24-96606-FP-01 comments response), 24-96606 JOH (24-96606-FP-02), 24-96606 JOH (24-96606-FP-01 comments), 24-96606 JOH (24-96606-FP-01)

---

### 9. Promenade poles — Client completes aesthetic/structural and detailed design

> The following assumptions have been made in the preparation of this proposal.
> If any of the following are incorrect, please advise us and we will prepare an updated version.
>
> - Aesthetic and Structural design of Promenade Poles shall be completed by the Client
>   - emittiv to advise on technical requirements and locations
> - Landscape, Carpark and Water Feature Lighting Design shall be completed by the Client
>   - emittiv to advise on control system requirements to be implemented by others
> - emittiv's design scope shall be concept and schematic, with final detailed design to be completed by the Client
>   - emittiv shall review and provide comments on the detailed design documents prior to submittal
>   - emittiv shall provide technical details as noted at detailed design stages

The FP-02 revision drops the "Landscape, Carpark and Water Feature" middle item and uses `•` markers; otherwise identical.

Used by: 23-97111 Dubai Island Promenade (e-23-97111-FP-01), 23-97111 Dubai Island Promenade (e-23-97111-FP-02 — Landscape item removed)

---

### 10. Concept design excluded (Block F — Aljada)

> The following assumptions have been made in the preparation of this proposal. If any of the following are incorrect, please advise us and we will prepare an updated version.
> - Block F excludes the Concept Design Stage from the scope of works. It is assumed that a fully developed Lighting Concept Design shall be provided allowing us to commence Schematic Design immediately.
> - It is assumed that the required high quality renders mentioned in the scope documents shall be prepared by others, with input from emittiv to guide the night time and lighting images.

FP-02 keeps these two points and adds three building-configuration assumptions specific to the email of 09 May 2025 (G007/G006/G005 similarity, 5 building configurations).

Used by: 25-97103 Aljada FG (25-97103-FP-01), 25-97103 Aljada Block F + G (25-97103-FP-02 — adds building-config assumptions)

---

### 11. "Project shall conform with information provided by the Client"

> The following assumptions have been made in the preparation of this proposal.
>
> If any of the following are incorrect, please advise us and we will prepare an updated version.
> • The project shall conform with the information provided by the Client.

Single-line body. Appears with `-` and `•` markers.

Used by: 24-96602 (e-24-96602-FP-01), 24-97102 DMC 2A (e-24-97102-FP-01)

---

### 12. WAMI — "Incomplete Data / Expended Focus / Back of House"

> The following assumptions have been made in the preparation of this proposal. If any of the following are incorrect, please advise us and we will prepare an updated version.
>
> - Incomplete Data: The information provided by the Client for this proposal does not include any lighting or programme data for us to evaluate. As such, scope, staffing, and durations mentioned in this document are educated estimates at best. We will undertake regular and ongoing reviews in conjunction with the Client to ensure appropriate scope and staffing levels are provided for the effective delivery of the works.
>
> - Expended Focus: The scope document and requested tasks are very much focused on the various lighting control systems on site. In our experience, this needs to encompass the actual lighting fixtures as well, since the two elements are fundamentally linked. It is not our intent to override any design choices or ongoing involvement of the existing lighting design teams in any way. However the overall success of any lighting scheme does depend on the tight integration of fixture selection, installation, and aiming, with scene programming, scheduling, and transitions.
>
> - Back of House: Back of House areas will be reviewed on a case by case basis. [Suggested addition: BOH is to be part of the scope.]

Three WAMI revisions carry the same three-topic body with minor differences: FP-01 (WDD notes) includes the bracketed suggested addition; FP-01 drops the bracket and adds "We have provided some suggested additions to support this."; FP-02 restructures the three topics as bold headers (no bullets) and omits the Back of House topic.

Used by: 25-97102 WAMI (25-97102-FP-01 WDD Lighting notes), 25-97102 WAMI (25-97102-FP-01), 25-97102 WAMI (25-97102-FP-02 — header form, no BOH)

---

### 13. Single-point and project-specific bodies (one doc each)

Each of the following uses the standard preamble with a project-specific assumption list. Listed by project for traceability.

- **22-96601 Dammam Waterpark (e-22-96601-FP-01)** — Client provides Concept design for all areas; Mechanical Effects assumed similar to the Globe; Sim Leisure provides story boards/scripts/renders, emittiv provides technical designs.
- **23-96601 Dammam Adv World (e-23-96601-FP-01)** — Client to provide all required drawings to be marked up by emittiv.
- **23-96603 Khobar Grand Mosque (e-23-96603-FP-01)** — Concept developed to a level allowing direct move to schematic/detailed; Architectural drawings and Revit models provided by Client.
- **23-96607 HOH KSA (e-23-96607-FP-01)** — Existing working drawings/specs provided in editable formats; Client supports visa procurement for site supervision.
- **23-97101 Marina Island Financial District (e-23-97101-FP-01)** — Street Lighting excluded (confirmed by email); project developed using BIM systems.
- **23-97102 Wynn (e-23-97102-AA-FP-01 and e-23-97102-EL-FP-01)** — No travel outside the U.A.E. required; FOUR [4] design workshops on site with core team allowed. (Two docs; `•` vs `-` markers.)
- **23-97106 Fountain Control (23-97106-FP-01)** — 3D model/offline show file/software setup by Awarded Contractor; working samples of existing components made available; suitable working facilities and internet/file-server access on site; project-mandated software provided by Client.
- **23-97110 HoH Dxb Update (e-23-97110-FP-01)** — Information on lighting elements designed by others provided in a ready-to-incorporate format for drawings and BoQs.
- **24-96601 Dammam Water Park (e-24-96601-FP-01)** — Client provides Schematic Design packages (LOD 200 Revit models); Concept design for Show Systems provided by Client.
- **24-96605 Observatory (24-96605-FP-01)** — Client provides all relevant existing design documentation and models.
- **24-97104 HoH Programming (e-24-97104-FP-01)** — Lighting/control and master [AV] control systems fully installed/commissioned/operational before on-site programming; programming narrative agreed before commencing.
- **24-97105 MAF MiCC Kids (24-97105-FP-01) and 25-97104 MAF MiCC Kids FEC Redesign (25-97104-FP-01)** — Only Detailed Design stage requested, so only the final report is produced; any changes to delivered design/equipment treated as variations. (Two docs; `-` vs `•` markers.)
- **24-97106 RAK Sled (24-97106-FP-01)** — Electrical system design by others.
- **24-97108 Tamani Hotel (24-97108-FP-01)** — Floors 6-25, 27-46 and 48-50 are duplicated sets.
- **24-97111 Masdar OS-48 (24-97111-FP-01)** — Project must meet LEED V4.0 Platinum, Estidama PBRS 05 Pearl, WELL Platinum, Masdar City's Energy Design Guidelines; vectorised copies of standards provided by Client.
- **25-97108 RAK Beach District (25-97108-FP-01)** — Facade lighting based on traditional lighting devices, not pixel-based video systems (pixel systems require different fee/time structures).
- **25-97109 Mapletree Warehouse (25-97109-FP-01)** — Copies of all relevant codes/standards/regulations provided to emittiv by the Client.
- **26-96801 Rozana Muscat (26-96801-FP-01)** — Existing lighting plans/as-builts/specs provided (else on-site audit may be required); Client provides hotel/brand/local standards before design commences.

---

**Recommended standard wording:**

Preamble (use the two-line form and `-` hyphen markers for consistency):

> The following assumptions have been made in the preparation of this proposal.
> If any of the following are incorrect, please advise us and we will prepare an updated version.

Where the designer/engineer coordination boundary is needed, the full block from variant 2 is the most complete and defensible canonical version:

> - This proposal assumes that the Architect [or other Lead Designer] and Electrical Engineer, maintain their usual role and responsibility for the co-ordination of the design and supervision of the installation on site.
> - Information provided is for use by the Architect and Electrical Engineer in the preparation of the construction / tender documentation.
> - emittiv's drawings, specifications and other information is not to be issued direct to the Contractor as tender or construction information.
> - It is assumed that CAD files / drawings provided for our use will be in a ready to use format and require no work by emittiv to prepare them.
> - emittiv is not an engineering specialist. All details provided are intended to assist appropriate specialists in making professional assessments.
> - All information provided regarding trusses, rigging, lifts or other structural or mechanical elements must be verified by suitably qualified professionals.
> - emittiv is not qualified to determine whether the designs meet structural, safety and / or building code requirements.
> - The exact method of fastening or connection must be verified by the Architect / Structural Engineer.

For site-visit cost protection, append the variant 3 line:

> - In the event of emittiv visiting site for meetings or inspections and being unable to complete their work, additional charges will be made as outlined in the Fees section of this Proposal.

Project-specific assumptions (scope exclusions, Client-provided inputs, standards to meet) are then added per project beneath this standard block.

---

## Exclusions

Clauses defining what is outside the proposed scope. The dominant pattern is a discipline-organised list (Lighting / Audio / Control Systems), with two project-type outliers (theme-park ride works, and a museum project that leads with an inclusion).

### Discipline-organised exclusions (the standard pattern)

The most common form. A heading line ("The proposed scope excludes the following areas:") followed by a per-discipline breakdown. The richest representative is the Dammam Water Park version, which covers all four disciplines plus rides:

> The proposed scope excludes the following areas:
>
> - Lighting:
>   - Emergency Lighting Systems
>   - BoH Areas
>   - Shell and Core tenanted spaces
>   - Street Lighting requiring a Statutory Authority's approval
> - Audio:
>   - PAVA Systems [though appropriate tie-in points shall be provided in the main system design]
> - Control Systems:
>   - Site wide IT networking [connectivity and designs shall be provided for the systems in scope]
> - Rides and Slides:
>   - Technical equipment integral to rides shall be developed by the ride vendors, however control and general integration of these components within the wider systems shall be provided.

Used by: 24-96601 Dammam Water Park

The remaining projects use trimmed subsets of the same clause, varying only in which disciplines and bullets appear:

- **Lighting + Audio (Emergency / BoH / PAVA)** - the most common reduced form:
  > The proposed scope excludes the following areas:
  > - Lighting:
  >   - Emergency Lighting
  >   - BoH Areas
  > - Audio:
  >   - PAVA / Emergency systems

  Used by: 24-97113 Level 63, 25-97105 Shanghai Tang v2 (Shanghai Tang is identical apart from bullet markers and blank-line spacing).

- **Lighting only (full four-bullet list)**:
  > The proposed scope excludes the following areas:
  > • Lighting:
  >   - BoH Areas
  >   - Shell and Core tenanted spaces
  >   - Street Lighting requiring a Statutory Authority's approval

  Used by: 24-97102 DMC 2A. KSA Pav Osaka uses the same list but with a placeholder first bullet ("xxx") and an added "Emergency Lighting"-style slot left blank - clearly an unfinished draft of this clause (23-96606 KSA Pav Osaka).

Note: differences across this group are trivial wording/formatting only - "Emergency Lighting" vs "Emergency Lighting Systems", bullet glyph ("-" vs "•"), and blank-line spacing. The disciplines covered (Lighting, Audio, Control Systems, Rides) and the recurring excluded items (Emergency/PAVA, BoH, Shell & Core, Street Lighting, IT networking) are stable.

### Inclusion-then-exclusion variant

One project frames scope by stating what IS included first, then the exclusions. Same exclusion vocabulary, different framing:

> The proposed scope includes the following areas:
> - All FoH and Offices as indicated in the supplied Area Table
>
> The proposed scope excludes the following areas:
> - All Specialist Museum Exhibit units
> - BoH / Storage Areas
> - Lighting: Shell and Core tenanted spaces; Street Lighting requiring a Statutory Authority's approval
> - Audio: PA/VA Systems

Used by: 24-97112 Military Museum

### Theme-park / ride-works exclusions

A distinct clause for attraction projects, excluding ride engineering and content/media deliverables rather than building disciplines:

> The following items are specifically excluded from this proposal:
> • Ride structure and operation - any direct interfaces with these elements are to be confirmed by specialist ride engineers
> • Content / Soundtrack / Special Effects - can be proposed and quoted at a later date
> • DD Information for any animatronics - this typically needs to be developed by contractor / supplier
> • Realistic Renders or Fly-throughs - it is assumed that the theming contractor would provide these if required. emittiv to advise and make recommendations throughout the development process.
> • Structural calculations
> • MEP calculations
>
> The following areas are also excluded:
> • Back of House areas
> • Emergency Lighting
> • Detailed connectivity to external sources [e.g. BMS, Fire Alarms, PAVA, Digital Signage], however typical inputs to our designed systems shall be provided.

Used by: 23-96602 Shoot the Chute

### Minimal / project-specific exclusion

A short bespoke exclusion that does not follow the discipline pattern:

> The proposed scope excludes the following areas:
> - Content by others
> - Show control system installed on audio processors

Used by: 22-97112 The View Exp Ctr

---

**Recommended standard wording:**

Use the discipline-organised clause as the canonical base, including only the disciplines relevant to the project. The Dammam Water Park version is the most complete template; drop the Rides section for non-attraction work:

> The proposed scope excludes the following areas:
>
> - Lighting:
>   - Emergency Lighting Systems
>   - BoH Areas
>   - Shell and Core tenanted spaces
>   - Street Lighting requiring a Statutory Authority's approval
> - Audio:
>   - PAVA Systems [though appropriate tie-in points shall be provided in the main system design]
> - Control Systems:
>   - Site wide IT networking [connectivity and designs shall be provided for the systems in scope]

Add a "Rides and Slides" section for attraction/water-park projects, or substitute the theme-park clause (Shoot the Chute) where ride engineering, content, and media deliverables need to be excluded. The parenthetical tie-in notes ("appropriate tie-in points shall be provided", "connectivity and designs shall be provided for the systems in scope") should be retained - they pre-empt the most common client objection that exclusion means total disconnection.

---

## Proposal Validity

This clause sets the period for which a fee proposal remains binding from its date of issue. Across the corpus the wording is highly standardised; the only meaningful variation is the validity period itself (60 / 90 / 120 days) and whether the clause appears standalone or embedded inside a larger preamble block.

### Variant A - 60 days (dominant)

The overwhelming default. Used as a standalone clause in 62 documents, and also embedded inside every composite preamble block below.

> This proposal shall remain valid for a period of sixty [60] days from the date of issue.

Used by: 22-96601, 22-97111, 22-97112, 22-97113, 22-97114, 22-97115, 23-96601, 23-96602, 23-96603, 23-96604, 23-96605, 23-96607, +50 more (62 documents total)

### Variant B - 90 days

> This proposal shall remain valid for a period of ninety [90] days from the date of issue.

Used by: 23-97102 Wynn (both AA-FP-01 and EL-FP-01)

### Variant C - 120 days

Same clause, longer period. Note: written as a plain numeral `120` without the bracketed-word form (`one hundred and twenty [120]`) used by the 60/90 variants.

> This proposal shall remain valid for a period of 120 days from the date of issue.

Used by: 23-97106 Fountain Control

### Variant D - embedded in preamble block (60 days)

In four documents the validity clause is not standalone but appears inside a larger introductory block that also covers the RFP acknowledgement, reference documents, project details, and packages. The validity wording within these is identical to Variant A (60 days); only the surrounding structure differs (some use a `Validity` heading, some inline it as `Validity:`).

Representative (23-97101 Marina Island Financial District):

> emittiv ["Consultant", "We"] have received a Request for Proposal from P&T Group ["Client"].
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.
>
> Validity
> This proposal shall remain valid for a period of sixty [60] days from the date of issue.
>
> Project Details
> The Project is known as Marina Island Financial District and is located in Kasssir Island, Abu Dhabi, U.A.E.
> [...project details, reference documents, and packages follow...]

Used by: 23-97101 Marina Island Financial District, 24-96603 Marasi Gate, 25-97108 RAK Beach District (each bundles the 60-day validity line into a full preamble block)

### Recommended standard wording

The 60-day form is the company default by a wide margin (62 standalone + 4 embedded uses) and should be the canonical clause. Keep the bracketed numeral convention (`sixty [60]`) and apply it consistently to the longer periods when used:

> **This proposal shall remain valid for a period of sixty [60] days from the date of issue.**

For longer-validity proposals, mirror the bracketed-word format - `ninety [90]` or `one hundred and twenty [120]` - rather than the bare-numeral `120` seen in 23-97106. The validity clause should stand alone as its own clause/heading; folding it into a preamble block (Variant D) is an artefact of older proposals and is not recommended for the standard template.

---

## Limitation of Liability

All 69 fee proposals across the corpus use a single Limitation of Liability clause. The only differences between variants are trivial rewordings of the liability-cap sentence; the consequential-loss exclusion and the force-majeure sentence are identical everywhere.

### Standard clause (consequential-loss exclusion + liability cap + force majeure)

> In no event shall emittiv be liable in tort, contract or otherwise [including negligence] to compensate the Client for any business interruption, loss of [anticipated] profits, revenue, business, contracts or [anticipated] savings, costs of procurement of substitute goods or services or any special, indirect or consequential loss or any punitive damages.
>
> In no event shall emittiv's liability to the Client under any circumstances exceed the amount of compensation actually received by emittiv from the Client under any related Agreement as of a certain date.
>
> emittiv shall not be liable for delays or performance failures due to circumstances beyond our control.

Used by: 22-96601, 22-97111, 22-97112, 22-97113, 22-97114, 22-97115, 23-96601, 23-96602, 23-96603, 23-96604, 23-96605, 23-96606 +55 more (67 documents total)

### Cap-sentence wording variants (otherwise identical)

The liability-cap sentence appears in three interchangeable phrasings. Paragraphs 1 (consequential loss) and 3 (force majeure) are byte-identical across all three.

- **Variant A - "In no event shall... exceed"** (the dominant form, 67 docs): `In no event shall emittiv's liability to the Client under any circumstances exceed the amount...`
- **Variant B - "In all events... shall not exceed"** (1 doc): `In all events, emittiv's liability to the Client under any circumstances shall not exceed the amount...`
  - Used by: 22-97111 TPF Sound System (signed copy, e-22-97111-FP-01)
- **Variant C - "...shall not exceed"** (1 doc): `emittiv's liability to the Client under any circumstances shall not exceed the amount...`
  - Used by: 22-97112 The View Exp Ctr (signed copy, e-22-97112-FP-01)

Note: Variants B and C are the two *signed* copies of proposals that, in their draft form, used Variant A (22-97111 and 22-97112 both also appear in the Variant A list). The wording change is cosmetic and does not alter the cap's meaning - the cap is always the total compensation actually received by emittiv under the related agreement.

**Recommended standard wording:**

> In no event shall emittiv be liable in tort, contract or otherwise [including negligence] to compensate the Client for any business interruption, loss of [anticipated] profits, revenue, business, contracts or [anticipated] savings, costs of procurement of substitute goods or services or any special, indirect or consequential loss or any punitive damages.
>
> In no event shall emittiv's liability to the Client under any circumstances exceed the amount of compensation actually received by emittiv from the Client under any related Agreement as of a certain date.
>
> emittiv shall not be liable for delays or performance failures due to circumstances beyond our control.

This is Variant A, used in 67 of 69 documents. One drafting note worth resolving before standardising: "as of a certain date" is vague and should be replaced with a defined reference point (e.g. "as at the date the claim arises") for legal clarity - but the verbatim text above reflects the corpus as written.

---

## Basis of Appointment

This clause establishes the governing professional-services agreement, the precedence of the proposal over any client-supplied contract, and the dispute-resolution jurisdiction. Three governing agreements appear across the corpus: the ACE Professional Services Agreement 2017 (the dominant standard), the ACE Short Form Agreement 2015, and the FIDIC Client/Consultant Model Services Agreement (White Book). All variants share the same closing two paragraphs (governing-document protection + DIFC Courts jurisdiction).

---

### Variant A - ACE Professional Services Agreement 2017 (standard)

The overwhelmingly dominant form. Two near-identical sub-variants differ only by trivial wording: A1 uses "form part of this proposal and **be** read as contained herein" (61 uses across the corpus), A2 drops the "be" ("form part of this proposal and read as contained herein", 4 uses). Same meaning; A1 is the cleaner reading.

> Our appointment shall be subject to and in accordance with the Association for Consultancy and Engineering [ACE] Professional Services Agreement 2017. The terms and conditions laid out in the ACE Professional Services Agreement shall form part of this proposal and be read as contained herein.
>
> Any contracts provided by the Client to the Consultant must not substantially or materially alter the scope or terms outlined in this Proposal and must include reference to this Proposal as the governing document.
>
> Any dispute, difference, controversy or claim arising out of or in connection with this contract, including [but not limited to] any question regarding its existence, validity, interpretation, performance, discharge and applicable remedies, shall be subject to the exclusive jurisdiction of the Courts of the Dubai International Financial Centre ["the DIFC Courts"].

Used by: 22-96601, 22-97114, 22-97115, 23-96601, 23-96602, 23-96603, 23-96604, 23-96605, 23-96607, 23-97101, 23-97102 (Wynn AA + EL), 24-97104, 25-97103, 25-97106 (MAF MOE FEC), 22-97113 (MAF FEC), 24-97101 (HoH Supervision) +44 more (61 total, A1+A2 combined; includes 22-97111 TPF, 22-97112 The View as the A2 sub-variant)

---

### Variant B - ACE PSA 2017 with acceptance preamble

Same ACE PSA 2017 body as Variant A, but prefixed with an explicit appointment-acceptance statement naming the client and confirming qualification/capacity. Single use.

> emittiv ["Consultant", "We"] have received a Request for Proposal from Stickman Tribe Ltd. ["Client"].
>
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.
>
> Our appointment shall be subject to and in accordance with the Association for Consultancy and Engineering [ACE] Professional Services Agreement 2017.
> The terms and conditions laid out in the ACE Professional Services Agreement shall form part of this proposal and be read as contained herein.
> Any contracts provided by the Client to the Consultant must not substantially or materially alter the scope or terms outlined in this Proposal and must include reference to this Proposal as the governing document.
> Any dispute, difference, controversy or claim arising out of or in connection with this contract, including [but not limited to] any question regarding its existence, validity, interpretation, performance, discharge and applicable remedies, shall be subject to the exclusive jurisdiction of the Courts of the Dubai International Financial Centre ["the DIFC Courts"].

Used by: 23-97109 (Ciel VIP, FP-01)

---

### Variant C - ACE Short Form Agreement 2015

Identical structure to Variant A but invokes the ACE **Short Form** Agreement 2015 instead of the full PSA 2017. Used for smaller/simpler engagements.

> Our appointment shall be subject to and in accordance with the Association for Consultancy and Engineering [ACE] Short Form Agreement 2015.
> The terms and conditions laid out in the ACE Short Form Agreement shall form part of this proposal and be read as contained herein.
> Any contracts provided by the Client to the Consultant must not substantially or materially alter the scope or terms outlined in this Proposal and must include reference to this Proposal as the governing document.
> Any dispute, difference, controversy or claim arising out of or in connection with this contract, including [but not limited to] any question regarding its existence, validity, interpretation, performance, discharge and applicable remedies, shall be subject to the exclusive jurisdiction of the Courts of the Dubai International Financial Centre ["the DIFC Courts"].

Used by: 24-96606 (JOH, FP-01 and FP-02 plus comment rounds)

---

### Variant D - FIDIC White Book (2017, 5th Ed)

Used where the client mandates a FIDIC Client/Consultant Model Services Agreement. The core appointment paragraph mirrors Variants A/C but names the 2017 White Book. Several sub-forms appear, all on the single WAMI project (25-97102), reflecting an iterative negotiation:

- **D1 (negotiation note + appointment):** opens with a reservation noting the client's proposed contract is the outdated 2006 version, flags non-standard amendments to be reviewed, and notes the proposal is missing as a governing document - then states the standard appointment paragraph.
- **D2 (appointment + "Consultant Agreement" note):** the standard appointment paragraph first, followed by the same negotiation reservations under a "Consultant Agreement:" heading.
- **D3 (note only):** just the negotiation reservation paragraph, no appointment clause.
- **D4 (appointment only):** just the standard appointment paragraph, no reservations.

Representative appointment text (D4):

> Our appointment shall be subject to and in accordance with the FIDIC Client / Consultant Model Services Agreement - 5th Ed (2017 White Book).
> The terms and general conditions laid out in the 2017 White Book shall form part of this proposal and be read as contained herein.
> Any contracts provided by the Client to the Consultant must not substantially or materially alter the scope or terms outlined in this Proposal and must include reference to this Proposal as the governing document.
> Any dispute, difference, controversy or claim arising out of or in connection with this contract, including [but not limited to] any question regarding its existence, validity, interpretation, performance, discharge and applicable remedies, shall be subject to the exclusive jurisdiction of the Courts of the Dubai International Financial Centre ["the DIFC Courts"].

Representative reservation text (D1/D3):

> In principle we have no objection to working under the FIDIC Client / Consultant Model Services Agreement [White Book], though we note that the proposed contract is based on the 2006 version rather than the current 2017 edition. We further note that there are a number of specific conditions which alter the standard contract. These amendments will be reviewed and discussed prior to engagement. We also make mention of the fact that the proposed contract does not include the offer of the Consultant as a governing document, which will need to be remedied prior to proceeding.

Used by: 25-97102 (WAMI; FP-01 and FP-02, across the D1-D4 sub-forms)

---

**Recommended standard wording:**

For the default case (no client-mandated framework), use Variant A1 - the ACE PSA 2017 form, which accounts for the large majority of the corpus:

> Our appointment shall be subject to and in accordance with the Association for Consultancy and Engineering [ACE] Professional Services Agreement 2017. The terms and conditions laid out in the ACE Professional Services Agreement shall form part of this proposal and be read as contained herein.
>
> Any contracts provided by the Client to the Consultant must not substantially or materially alter the scope or terms outlined in this Proposal and must include reference to this Proposal as the governing document.
>
> Any dispute, difference, controversy or claim arising out of or in connection with this contract, including [but not limited to] any question regarding its existence, validity, interpretation, performance, discharge and applicable remedies, shall be subject to the exclusive jurisdiction of the Courts of the Dubai International Financial Centre ["the DIFC Courts"].

Swap the agreement name for the **ACE Short Form Agreement 2015** (Variant C) on smaller engagements, or the **FIDIC 2017 White Book** (Variant D) where the client mandates FIDIC - retaining the final two paragraphs unchanged. The last two paragraphs (governing-document precedence + DIFC Courts jurisdiction) are invariant across every variant and should always be included.

---

## Defined Role

Defines the boundary of emittiv's appointment: what the scope covers (design + equipment specification for the named packages), the disciplines explicitly excluded (architectural, electrical engineering, acoustic/fire-rated coordination), and the client's duty to disclose applicable regulations/standards/landlord guidelines at appointment.

---

### Variant A - Standard (lighting, video & sound)

The dominant wording, used in 61 documents. Full discipline exclusions plus the "other than Part L or CIBSE" carve-out on the regulations clause.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
>
> emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
>
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
>
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
>
> emittiv is not responsible for the coordination of the package with any acoustic or fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.
>
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Used by: 22-97111, 22-97112, 22-97113, 22-97114, 22-97115, 23-96601, 23-96602, 23-96603, 23-96604, 23-96605, 23-96606, 23-96607 +49 more (61 documents total; minor whitespace/em-dash differences in 22-97112-FP-01 are folded in here as the same clause).

---

### Variant B - Standard, no Part L/CIBSE carve-out

Identical to Variant A in structure, but the regulations clause omits "other than Part L or CIBSE" (broader, unqualified obligation on the client to disclose all standards). Used on the Wynn AA and EL fee proposals.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
> emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
> emittiv is not responsible for the coordination of the package with any fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Differences from Variant A: drops "other than Part L or CIBSE"; the coordination exclusion says "any fire rated treatments" rather than "any acoustic or fire rated treatments" (acoustic omitted).

Used by: 23-97102 (Wynn) - AA-FP-01, EL-FP-01.

---

### Variant C - Audio & acoustics scope

Same clause family, but the scope line is for an audio/acoustics package rather than lighting/video/sound. Used on the Tape Theatre proposal.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
> emittiv's scope includes the design of the audio and acoustics [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
> emittiv is not responsible for the coordination of the package with any acoustic or fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Differences from Variant A: scope is "audio and acoustics" instead of "lighting, video, and sound"; regulations clause has no "other than Part L or CIBSE" carve-out (as Variant B).

Used by: 23-97105 (Tape Theatre) - FP-01, FP-01-AH comments.

---

### Variant D - Extended scope (lighting, video, sound & other elements)

Broadest scope wording and the only variant that drops the architectural-services exclusion entirely. Used on the Dammam Waterpark proposal.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
>
> emittiv's scope includes the design of the lighting, video, sound, and other elements [as indicated in the Packages section above] and the specification of the equipment that produces them in conjunction with the Client and other members of the Design Team.
>
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
>
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Differences from Variant A: scope adds "and other elements"; omits both the architectural-services exclusion and the acoustic/fire-rated coordination exclusion. Keeps the Part L/CIBSE carve-out.

Used by: 22-96601 (Dammam Waterpark) - FP-01.

---

### Variant E - Standard, no regulations clause

Identical to Variant A but with the entire "Regulations / standards / landlord guidelines" paragraph omitted. Used on the RAK Sled proposal.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
>
> emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
>
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
>
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
>
> emittiv is not responsible for the coordination of the package with any acoustic or fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.

Differences from Variant A: omits the regulations/standards/landlord-guidelines paragraph.

Used by: 24-97106 (RAK Sled) - FP-01.

---

### Variant F - "Not an electrical specialist" rewording

Softens the electrical exclusion from a flat "does not provide" to a "not an electrical specialist - information provided to be verified by qualified specialists" framing. Used on a revised JOH proposal.

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed. emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
>
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
>
> emittiv is not an electrical specialist. Any information provided relating to electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc. shall be reviewed and verified by suitably qualified specialists.
>
> emittiv is not responsible for the coordination of the package with any acoustic or fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.
>
> Regulations / standards / landlord guidelines - It is the responsibility of the Client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Differences from Variant A: electrical clause is reworded (allows emittiv to provide electrical info, but disclaims specialist status and shifts verification to qualified specialists); "Client" capitalised in the regulations clause. Note: this co-exists with Variant A on the same project (24-96606 JOH FP-01 uses Variant A; FP-02 uses this wording).

Used by: 24-96606 (JOH) - FP-02.

---

**Recommended standard wording:**

Variant A is the clear canonical version - 61 of 70 documents, all the major packages, with the complete set of exclusions and the Part L/CIBSE carve-out. Use it verbatim, with blank lines between paragraphs:

> emittiv's role is defined by the services listed in the Services section of this document and all additional services agreed.
>
> emittiv's scope includes the design of the lighting, video, and sound [as indicated in the Packages section above] and the specification of the equipment that produces it in conjunction with the Client and other members of the Design Team.
>
> emittiv does not provide architectural services such as fully coordinated working details, details of mechanical fixings, builder's work etc.
>
> emittiv does not provide electrical engineering services i.e. cable sizing, circuitry drawings, co-ordination with existing power or ventilation layout, associated builder's work etc.
>
> emittiv is not responsible for the coordination of the package with any acoustic or fire rated treatments or the specification of any such treatments, the design and specification of which must be carried out by others.
>
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE. This includes any country, city, state, landlord, client, tenant regulations or guidelines.

Swap the scope line (second paragraph) to match the package on offer - e.g. "audio and acoustics" (Variant C) or "lighting, video, sound, and other elements" (Variant D) - keeping every other paragraph identical. Variant F's softer electrical wording is a deliberate per-project choice (allows emittiv to share electrical info under disclaimer) and should be selected consciously, not adopted as default.

---

## Contract Details & Site Attendance

This clause type combines two recurring components that appear together in most proposals: an **Anticipated Programme** (work-stage durations, commencing on receipt of a fully executed contract) and a **Site Attendance** statement (where work is undertaken, travel/visit charging, remote-meeting preference). The programme block is almost always project-specific (different stages and durations per job), so the durable, reusable content lives in the boilerplate sentences that wrap it and in the Site Attendance paragraph. Below, variants are grouped by the stable wording patterns rather than by the per-project stage tables.

---

### 1. Site Attendance only — Dubai office (standalone)

The core Site Attendance paragraph, used on its own without a programme table.

> We understand that all works are to be undertaken in conjunction with the Client's Dubai based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 22-97111 TPF Sound System FP-01, 22-97114 HoH FP-01, 24-97102 DMC 2A FP-01, 24-96606 JOH FP-01 comments response (count 4, this exact standalone form).

### 2. Site Attendance only — Abu Dhabi office (standalone)

Identical to variant 1 but the client office is Abu Dhabi.

> We understand that all works are to be undertaken in conjunction with the Client's Abu Dhabi based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 24-97113 Level 63 FP-01.

---

### 3. Programme boilerplate — standard caveats (no Site Attendance)

The standard programme wrapper sentences, used without an attached Site Attendance paragraph. Stage tables vary per project; the durable wording is the caveat set. Minor wording differences exist across instances ("Stage 1 - / Stage 1 — / Stage 1 |" separators; some omit "The durations shown are to be confirmed..." or "Requests to reduce...").

> We propose the following programme, commencing with receipt of a fully executed contract.
>
> [project-specific stage table]
>
> These estimations of work stage durations do not take into consideration Client review or Revision periods.
> The durations shown are to be confirmed with the Client during Preliminaries.
> Requests to reduce deliverable time frames may require additional staff and fees.

Used by: 22-97111 TPF Sound System FP-01 (+signed), 22-97114 HoH FP-01, 24-97102 DMC 2A FP-01, 24-97113 Level 63 FP-01, 25-97108 RAK Beach District FP-01 (programme-only forms) +several more partial-block variants.

### 4. Programme + Site Attendance — Dubai office (the dominant combined form)

The most common full clause: standard programme boilerplate plus the Dubai Site Attendance paragraph. The only meaningful differences between the many instances are (a) the project-specific stage/duration table, (b) "Design Phase" / "Anticipated Programme" headings present or absent, (c) separator style ("-", "—", "|", or "Stage | Milestone | Duration" tables), and (d) occasional "Post Contract Phase" stages appended. The boilerplate sentences and the Site Attendance paragraph are stable.

> [optional heading: Anticipated Programme / Design Phase]
>
> We propose the following programme, commencing with receipt of a fully executed contract.
>
> [project-specific stage table — e.g. Preliminaries, Concept Design, Schematic Design, Detailed Design, Tender Documentation]
>
> These estimations of work stage durations do not take into consideration Client review or Revision periods.
> The durations shown are to be confirmed with the Client during Preliminaries.
> Requests to reduce deliverable time frames may require additional staff and fees.
>
> Site Attendance
> We understand that all works are to be undertaken in conjunction with the Client's Dubai based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 22-96601 Dammam Waterpark, 22-97112 The View Exp Ctr, 22-97113 MAF FEC FP-01/FP-02, 22-97115 Kids Activity Park AUH FP-01/FP-02, 23-96602 Shoot the Chute, 23-97101 Marina Island, 23-97105 Tape Theatre FP-01, 23-97106 Fountain Control, 23-97107 DAFZA Refurb +37 more (Dubai-office combined form is the workspace default).

### 5. Programme + Site Attendance — Abu Dhabi office

Same combined structure as variant 4, with the Abu Dhabi office substitution. Often carries an added "We understand that there are time constraints on this project and will endeavour to progress the design as quickly as possible." line.

> We propose the following programme, commencing with receipt of a fully executed contract.
>
> [project-specific stage table]
>
> These estimations of work stage durations do not take into consideration Client review or Revision periods.
> The durations shown are to be confirmed with the Client during Preliminaries.
> Requests to reduce deliverable time frames may require additional staff and fees.
>
> Site Attendance
> We understand that all works are to be undertaken in conjunction with the Client's Abu Dhabi based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 24-97107 Reserve Cut, 25-97101 Shanghai Tang FP-01/FP-02, 25-97105 Shanghai Tang v2 FP-01 (25-97101-FP-01), 25-97105 Shanghai Tang v2 FP-01 (25-97105-FP-01).

### 6. Programme + Site Attendance — Ras Al Khaimah office

Same combined structure, RAK office. Includes the WAMI "Review and Report Phase / Post Contract Phase" forms and the Wynn AA/EL forms ("Ras al-Khaimah").

> We propose the following programme, commencing with receipt of a fully executed contract.
>
> [project-specific stage table]
>
> These estimations of work stage durations do not take into consideration Client review or Revision periods.
> The durations shown are to be confirmed with the Client during Preliminaries.
>
> Site Attendance
> We understand that all works are to be undertaken in conjunction with the Client's Ras Al Khaimah based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 25-97102 WAMI FP-01 (+FP-01 WDD notes, FP-02), 23-97102 Wynn AA-FP-01 / EL-FP-01, 24-97106 RAK Sled FP-01. (Spelling varies: "Ras Al Khaimah" vs "Ras al-Khaimah".)

### 7. Programme + Site Attendance — remote / overseas client office

Variant where the works are explicitly "undertaken remotely" and/or the client office is outside the UAE (Wielsbeke Belgium, Dammam KSA). Some add "ancillary costs" to the travel-charge sentence.

> [project-specific stage table + standard caveats]
>
> Site Attendance
> We understand that all works are to be undertaken [remotely] in conjunction with the Client's [Wielsbeke / Dammam] based office.
> This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation [and ancillary] costs.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 23-96601 Dammam Adv World (Wielsbeke), 23-96603 Khobar Grand Mosque (Dammam, remote), 23-96604 DockX (Wielsbeke, remote, "ancillary costs"), 23-96605 Jumpoline (Wielsbeke), 24-96603 Marasi Gate (Dammam, "visits will be charged"), 24-97102 DMC 2A FP-01.

### 8. Programme + Site Attendance — two/limited client visits included

Variant 4 modified so the proposal provides a fixed number of visits rather than "does not provide for any travel."

> [programme + caveats]
>
> Site Attendance: We understand that all works are to be undertaken in conjunction with the Client's Dubai based office. This proposal provides for two visits to the End Client offices. If required, additional visits will be charged at the prevailing daily rates, plus travel and accommodation costs. In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 22-96601 Dammam Waterpark FP-01.

### 9. Programme + Site Attendance — monthly visit included (JOH FP-02)

Same family, but one visit per month included with travel/accommodation invoiced separately. Also adds a pause/resume clause for duration-based stages.

> [programme + caveats + pause/resume note]
>
> Site Attendance: We understand that all works are to be undertaken in conjunction with the Client's Dubai based office. This proposal does not provide for any travel out of country. Up to one visit per month is included, with travel and accommodation costs to be invoiced separately. In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 24-96606 JOH FP-02.

### 10. Programme + extended on-site supervision (KSA residency)

Specialised Site Attendance for projects requiring staff based in-country for months, with flights/accommodation/meal allowances. Materially different from the standard paragraph.

> Site Attendance:
> All design phase work shall be undertaken by our Dubai office.
> The Client has requested site supervision around the clock during construction phases.
> We have allowed for 3 staff to be based in Saudi for 3 months. They would work in rotating 9hr shifts, 6 days per week.
> Programming supervision would include an additional 2 staff to be present in KSA for one month on a similar schedule.
> ...
> This proposal allows for flights, accommodation, meal and travel allowances for staff during the Post Contract Phase, for one continuous stay.
> Delays or breaks in site supervision may require adjustments to these fees.
> Any down time in country will be charged at our daily billing rate.
> In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Used by: 23-96607 HOH KSA FP-01.

### 11. Programme — durations defined in scope documents (no Preliminaries confirmation)

Aljada family replaces "to be confirmed with the Client during Preliminaries" with durations defined in the scope documents and a different change-control sentence. Otherwise the Dubai Site Attendance paragraph is standard.

> These estimations of work stage durations do not take into consideration Client review or Revision periods. The durations shown are as defined with the Client in the scope documents. Any changes to the durations shown may incur additional fees.

Used by: 25-97103 Aljada FG FP-01, 25-97103 Aljada Block F + G FP-02.

### 12. Programme with sound-system staff allocations (The View)

Programme augmented with per-stage staff-day allocations and an equipment-delay fee note, plus the standard Dubai Site Attendance paragraph. Two near-identical instances differ only in table separator style ("-" list vs "|" columns) and the "Staff allocations" vs "The proposal allows for the following staff allocations" lead-in.

> [Stage 1/2 review & calibrate programme]
> Note that the system tuning and calibration periods are expected to require 1 weeks' work...
> Some non-disruptive works may be carried out during operational hours...
> [per-stage Snr Engineer / Asst Engineer / Wireman day allocations]
> Delays caused by equipment issues will incur additional fees at the rates listed in the Additional Fees section of this document.
> Site Attendance: [standard Dubai paragraph]

Used by: 22-97112 The View Exp Ctr FP-01 (+signed).

---

**Recommended standard wording:**

Programme block (insert the project-specific stage/duration table where indicated):

> Anticipated Programme
>
> We propose the following programme, commencing with receipt of a fully executed contract.
>
> [Stage / Milestone / Duration table]
>
> These estimations of work stage durations do not take into consideration Client review or Revision periods. The durations shown are to be confirmed with the Client during Preliminaries. Requests to reduce deliverable time frames may require additional staff and fees.

Site Attendance block (substitute the client's city; the Dubai form is the default):

> Site Attendance
>
> We understand that all works are to be undertaken in conjunction with the Client's Dubai based office. This proposal does not provide for any travel out of country. If required, these will be charged at the prevailing daily rates, plus travel and accommodation costs. In the interest of efficiency and safety, emittiv prefers remote meetings wherever possible.

Notes for the editable fields: the client city (Dubai / Abu Dhabi / Ras Al Khaimah / Dammam / Wielsbeke) and the "remotely" qualifier are per-project; where a fixed number of visits is included, replace "does not provide for any travel out of country. If required, these will be charged..." with "provides for N visits... additional visits will be charged..." (variant 8) or the monthly-visit wording (variant 9). Use "as defined with the Client in the scope documents" (variant 11) instead of "to be confirmed with the Client during Preliminaries" when durations are contractually fixed. The KSA on-site residency paragraph (variant 10) is a distinct, project-specific block — do not fold it into the standard wording.

---

## Optional Services

This clause lists Services excluded from the Proposal that emittiv can provide on request. All variants share the same lead-in sentence ("Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:") and differ only in the bullet list. Bullet character (`•` vs `-`) is a PDF-extraction artifact and is treated as identical here.

### Variant A — Standard full list (canonical)

The dominant clause: 13 lighting-design exclusions, ending with the as-built line attributed to the Electrical Engineer or Contractor.

> Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:
> - VR / AR / XR Simulations
> - Designs for shows within performance spaces or systems
> - Content creation for video and sound systems
> - Sunlight design and studies, including the hire of testing facilities
> - Design, supervision, construction, testing and evaluation of mock-ups and/or models
> - Design development of custom designed luminaires
> - Formal review or evaluation of final building contract or tender documents
> - Review and checking of proposed substitute luminaires, as selected by others
> - Obtaining project related quotations from lighting manufacturers
> - Shop and field inspections
> - Whole life cost and running cost reports
> - Final lamp schedule for the Client's building maintenance staff
> - As built / installed drawings [to be provided by the Electrical Engineer or Contractor]

Used by (60 docs): 22-97114 HoH, 23-96601 Dammam Adv World, 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-97101 Marina Island, 23-97102 Wynn (AA + EL), 23-97107 DAFZA Refurb, 23-97108 Ciel Lobby (FP-01/02), 24-96603 Marasi Gate, 24-97102 DMC 2A, 24-97104 HoH Programming, 24-97106 RAK Sled +47 more.

Includes the `•`-bulleted spelling and the minor sub-variant "Sunlight design and studies **including**..." (dropped comma) — both used by 22-97112 The View Exp Ctr, 22-97111 TPF Sound System (3 docs).

### Variant B — As-built attributed to the Client

Identical to Variant A except the final line reads "[to be provided by **the Client**]" instead of "the Electrical Engineer or Contractor".

> - As built / installed drawings [to be provided by the Client]

Used by (4 docs): 24-96606 JOH (FP-01 + comments responses + FP-02).

### Variant C — KSA HOH (drops shop/field inspections)

Variant A minus the "Shop and field inspections" line; otherwise identical (12 items).

> Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:
> • VR / AR / XR Simulations
> • Designs for shows within performance spaces or systems
> • Content creation for video and sound systems
> • Sunlight design and studies, including the hire of testing facilities
> • Design, supervision, construction, testing and evaluation of mock-ups and/or models
> • Design development of custom designed luminaires
> • Formal review or evaluation of final building contract or tender documents
> • Review and checking of proposed substitute luminaires, as selected by others
> • Obtaining project related quotations from lighting manufacturers
> • Whole life cost and running cost reports
> • Final lamp schedule for the Client's building maintenance staff
> • As built / installed drawings [to be provided by the Electrical Engineer or Contractor]

Used by (1 doc): 23-96607 HOH KSA.

### Variant D — Dammam Waterpark (drops substitute-review + generic "manufacturers")

Variant A minus the "Review and checking of proposed substitute luminaires" line, and "Obtaining project related quotations from **manufacturers**" (drops "lighting").

> • Obtaining project related quotations from manufacturers

Used by (1 doc): 22-96601 Dammam Waterpark.

### Variant E — AV/equipment wording (non-lighting projects)

Replaces lighting-specific terms with generic equipment terms: "custom designed **equipment**", "substitute **devices**", quotations from "**manufacturers**". Also drops the sunlight-studies line and the lamp-schedule line (11 items). Used on AV / theatre projects.

> Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:
> - VR / AR / XR Simulations
> - Designs for shows within performance spaces or systems
> - Content creation for video and sound systems
> - Design, supervision, construction, testing and evaluation of mock-ups and/or models
> - Design development of custom designed equipment
> - Formal review or evaluation of final building contract or tender documents
> - Review and checking of proposed substitute devices, as selected by others
> - Obtaining project related quotations from manufacturers
> - Shop and field inspections
> - Whole life cost and running cost reports
> - As built / installed drawings [to be provided by the Electrical Engineer or Contractor]

Used by (2 docs): 23-97105 Tape Theatre.

### Variant F — Dubai Island Promenade FP-02 (mixed wording)

Variant A but with "substitute **devices**" (rather than luminaires), dropping the sunlight-studies line and the lamp-schedule line (11 items).

> • Review and checking of proposed substitute devices, as selected by others

Used by (1 doc): 23-97111 Dubai Island Promenade (FP-02). Note: FP-01 of the same project uses Variant A.

### Variant G — Fountain Control (heavily reduced)

A short 8-item subset for a controls-only scope: drops shows, mock-ups, substitute review, quotations, and shop/field inspections.

> Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:
> - VR / AR / XR Simulations
> - Content creation for video and sound systems
> - Sunlight design and studies, including the hire of testing facilities
> - Design development of custom designed luminaires
> - Formal review or evaluation of final building contract or tender documents
> - Whole life cost and running cost reports
> - Final lamp schedule for the Client's building maintenance staff
> - As built / installed drawings [to be provided by the Electrical Engineer or Contractor]

Used by (1 doc): 23-97106 Fountain Control.

---

**Recommended standard wording:**

Use **Variant A** verbatim — it is the established standard (60 of 75 docs) and the most complete list:

> Unless agreed in writing by emittiv, the following Services are excluded from this Proposal but can be provided upon request:
> - VR / AR / XR Simulations
> - Designs for shows within performance spaces or systems
> - Content creation for video and sound systems
> - Sunlight design and studies, including the hire of testing facilities
> - Design, supervision, construction, testing and evaluation of mock-ups and/or models
> - Design development of custom designed luminaires
> - Formal review or evaluation of final building contract or tender documents
> - Review and checking of proposed substitute luminaires, as selected by others
> - Obtaining project related quotations from lighting manufacturers
> - Shop and field inspections
> - Whole life cost and running cost reports
> - Final lamp schedule for the Client's building maintenance staff
> - As built / installed drawings [to be provided by the Electrical Engineer or Contractor]

For non-lighting (AV / theatre) scopes, substitute "luminaires" → "equipment/devices" and "lighting manufacturers" → "manufacturers" (Variant E pattern). Where the Client takes responsibility for as-built records, use the "[to be provided by the Client]" closing line (Variant B).

---

## Preliminaries

This section consolidates the "Preliminaries" clause family across the fee-proposal corpus. The dominant clause is a standard deliverables/required-information block that appears in two near-identical forms (bullet style only: `•` vs `-`), sometimes preceded by a "Stages 1-N" preamble and occasionally extended with project-specific information items. A separate "company introduction" clause was also filed under this type in a couple of documents.

---

### Variant A — Standard deliverables block (core clause)

The base clause. Two trivially different forms exist depending on bullet marker (`•` bullets, 16 docs; `-` dashes, 12 docs); content is otherwise word-identical. Representative text (`•` form):

> Typical deliverables include:
> • Assistance in defining project aspirations
> • Assistance in project planning and brief development as appropriate
> • Review of site and project information
>
> emittiv requires the following information in order to commence:
> • Any particular requirements or aspirations
> • Any rules and regulations applying to the site
> • Design criteria and standards
> • Operational and functional requirements
> • Program and Area allocations
> • Existing project drawings, sketches, models and reports
> • Any established package budgets

Used by: 23-96602 Shoot the Chute, 23-96603 Khobar Grand Mosque, 23-97101 Marina Island, 23-97102 Wynn, 23-97108 Ciel Lobby, 23-97111 Dubai Island Promenade, 24-97107 Reserve Cut, 24-97110 Masdar B-02, 25-97108 RAK Beach District, 25-97106 MAF MOE FEC v3, 22-97113 MAF FEC, 22-97115 Kids Activity Park AUH +16 more (24 docs total across both bullet styles).

---

### Variant B — Standard block with "Stages 1-N" preamble

Identical deliverables block, prefixed by the sequencing/approval preamble. The stage count varies by project (1-2, 1-3, 1-4, 1-5); otherwise stable. Representative text:

> This proposal includes the services described below and is divided into Stages 1-4.
> Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
> The deliverables outlined in this document are typical for a wide range of projects.
> Specific Deliverables will be agreed with the Client during Preliminaries.
>
> Typical deliverables include:
> • Assistance in defining project aspirations
> • Assistance in project planning and brief development as appropriate
> • Review of site and project information
>
> emittiv requires the following information in order to commence:
> • Any particular requirements or aspirations
> • Any rules and regulations applying to the site
> • Design criteria and standards
> • Operational and functional requirements
> • Program and Area allocations
> • Existing project drawings, sketches, models and reports
> • Any established package budgets

Differences across instances of this variant: stage count (1-2, 1-3, 1-4, 1-5); bullet style (`•` / `-`); a couple of documents add an inline heading ("Design Phase — Preliminaries", "Prelimlinaries — Typical deliverables include:" [sic]); one Wynn EL document inserts a bracketed Exhibit reference and adds "as required" to the Specific Deliverables line; the WAMI 25-97102-FP-02 instance moves the agreement line to "Specific Deliverables will be reviewed and agreed with the Client during each stage."

Used by: 24-96603 Marasi Gate, 25-97105 Shanghai Tang v2, 25-97101 Shanghai Tang, 24-96602, 23-97102 Wynn (EL), 23-97107 DAFZA Refurb, 23-97109 Ciel VIP, 24-97104 HoH Programming (preamble only), 24-97105 MAF MiCC Kids, 24-97106 RAK Sled, 24-97108 Tamani Hotel, 24-97111 Masdar OS-48 +2 more (~16 docs).

---

### Variant C — Reduced deliverables + BoQ information item (JOH family)

A trimmed variant: the deliverables list keeps only "Review of site and project information", drops "or aspirations" from the first required item, and adds "Any existing package BoQs and specification sheets" to the required-information list. Appears in both bare and Stages-1-3-preamble forms, both bullet styles. Representative text:

> Typical deliverables include:
> - Review of site and project information
>
> emittiv requires the following information in order to commence:
> - Any particular requirements
> - Any rules and regulations applying to the site
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Any established package budgets
> - Any existing package BoQs and specification sheets

Used by: 24-96606 JOH (FP-01, FP-01 comments, FP-01 comments response, FP-02 — all revisions of the one project).

---

### Variant D — Standard block with "Standards and Codes" addition

Standard `•` block with one extra required-information item: "Soft copies of specific Standards and Codes to be observed".

> emittiv requires the following information in order to commence:
> • Any particular requirements or aspirations
> • Any rules and regulations applying to the site
> • Design criteria and standards
> • Soft copies of specific Standards and Codes to be observed
> • Operational and functional requirements
> • Program and Area allocations
> • Existing project drawings, sketches, models and reports
> • Any established package budgets

Used by: 22-96601 Dammam Waterpark.

---

### Variant E — Standard block with "Hotel Operators' guidelines" addition

Standard `-` block with one extra required-information item: "Hotel Operators' guidelines and requirements". One instance is bare, the other carries the Stages 1-2 preamble.

> emittiv requires the following information in order to commence:
> - Any particular requirements or aspirations
> - Any rules and regulations applying to the site
> - Hotel Operators' guidelines and requirements
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Any established package budgets

Used by: 23-97109 Ciel VIP (FP-01, FP-02).

---

### Variant F — Extended block (collaboration platforms + design-team intro)

A fuller variant used on the WAMI project. "Review of site and project information" becomes "Initial review of site and project information", and two required-information items are added: "Access to online storage / collaboration platforms" and "Introduction to existing design team members" (one revision expands this to "...and an explanation of our role in the project, in order to facilitate efficient information gathering").

> Typical deliverables include:
> - Assistance in defining project aspirations
> - Assistance in project planning and brief development as appropriate
> - Initial review of site and project information
>
> emittiv requires the following information in order to commence:
> - Any particular requirements or aspirations
> - Any rules and regulations applying to the site
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Access to online storage / collaboration platforms
> - Any established package budgets
> - Introduction to existing design team members

Used by: 25-97102 WAMI (FP-01, FP-01 WDD notes, FP-02).

---

### Variant G — Renovation/value-engineering deliverables (Dammam Water Park)

A re-worded deliverables list for an existing-design / value-engineering context. The required-information block is the standard set; only the deliverables differ.

> Typical deliverables include:
> - Review of existing/approved Schematic Design information and plans for site wide systems
> - Review of existing/approved Concept Design reports for Show Systems
> - Review of existing/approved budgets and preliminary pricing documents to assist with value engineering and design adjustments
>
> emittiv requires the following information in order to commence:
> - Any particular requirements or aspirations
> - Any rules and regulations applying to the site
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Any established package budgets

Used by: 24-96601 Dammam Water Park.

---

### Variant H — "Site Inspection[s]" deliverable addition

Standard `•` block with "Site Inspection[s]" added as the first deliverable.

> Typical deliverables include:
> • Site Inspection[s]
> • Assistance in defining project aspirations
> • Assistance in project planning and brief development as appropriate
> • Review of site and project information
>
> emittiv requires the following information in order to commence:
> • Any particular requirements or aspirations
> • Any rules and regulations applying to the site
> • Design criteria and standards
> • Operational and functional requirements
> • Program and Area allocations
> • Existing project drawings, sketches, models and reports
> • Any established package budgets

Used by: 24-97106 RAK Sled.

---

### Variant I — Company introduction (filed under Preliminaries)

A distinct clause: the emittiv "who we are" introduction, not a deliverables block. Two near-identical forms (one with line breaks between the project-types list and closing paragraph, one fully run-together; the run-together form also drops the final "For further information... visit our website" sentence). This is a different clause type that happens to have been classified here.

> emittiv is a multidisciplinary sensory design consultancy working with lighting, video, sound, scent, and control systems for the built environment. We design experiences, not just systems - making sure all of our elements work together to deliver a unified and polished concept.
> We work closely with Architects, Interior Designers, Lead Design Consultants and Owners / Operators on projects worldwide.
> Our aim is to enhance people's everyday lives by adding layers that make our spaces more interesting and beautiful.
> Our creative and commercial experience enables us to develop intelligent and appropriate designs within the parameters of budgets and time frames.
> We are entirely independent of any manufacturers or distributors, and operate on a fee for service basis.
>
> Our combined experience covers all types and scale of projects including:
> Attractions, Bars, Brand Activations, Casual Dining Venues, Cinemas, Corporate Headquarters, Department Stores, Destination Locations, Family Entertainment Centres, Galleries, Gaming Venues, Hotels, Landmark Buildings, Landscape, Malls, Museums, Nightclubs, Offices, Parks, Performance Venues, Public Realm, Residential, Retail Flagships, Shopping Centres, Spas, Theatres, Theme Parks.
>
> We can be appointed in a number of ways and tailor our services to a specific project and Clients' needs.
> All project teams are overseen by a Director and/or Associate to ensure a high level of service at all times.
> For further information on our services and project experience please visit our website: www.emittiv.com

Used by: 23-96605 Jumpoline, 25-96601 (run-together form, no website line).

---

**Recommended standard wording:**

For the core Preliminaries clause, use the Stages-preamble form with `-` (dash) bullets and the canonical 3-item deliverables / 7-item required-information lists. This is the most complete and most-used structure; the dash bullet renders consistently across InDesign and plain text. Adjust the stage count per project; append project-specific required-information items (BoQs, Hotel Operator guidelines, Standards/Codes, collaboration-platform access) only when relevant.

> This proposal includes the services described below and is divided into Stages 1-N.
> Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
> The deliverables outlined in this document are typical for a wide range of projects.
> Specific Deliverables will be agreed with the Client during Preliminaries.
>
> Typical deliverables include:
> - Assistance in defining project aspirations
> - Assistance in project planning and brief development as appropriate
> - Review of site and project information
>
> emittiv requires the following information in order to commence:
> - Any particular requirements or aspirations
> - Any rules and regulations applying to the site
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Any established package budgets

The company-introduction clause (Variant I) should be moved out of "Preliminaries" into a dedicated "About emittiv / Introduction" clause type; it is not a Preliminaries deliverable.

---

## Deliverables

Consolidated catalog of the FP "Deliverables" clause type. Variants are grouped by the design/service stage they describe. Within each group, near-identical wordings are merged and their differences noted. Bullet markers vary across source documents (`-`, `•`); these are treated as trivial formatting differences unless otherwise noted.

---

### 1. Proposal structure / division into stages

A short framing clause stating the proposal is divided into N stages, that each stage progresses on Client approval of the previous, and that specific deliverables are agreed during Preliminaries. The only meaningful difference between variants is the stage count.

> This proposal includes the services described below and is divided into Stages 1-7.
> Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work.
> The deliverables outlined in this document are typical for a wide range of projects.
> Specific Deliverables will be agreed with the Client during Preliminaries.

Differences across instances: stage count (1-5, 1-6, 1-7); some are formatted as separate lines, others as a single paragraph (semantically identical).

Used by: 24-97106 RAK Sled (1-7), 24-97112 Military Museum (1-5), 22-97113 MAF FEC (1-6), 22-97113 MAF FEC FP-02 (1-5, with appended Preliminaries block)

One FP-02 variant (22-97113 MAF FEC) extends the framing clause with a Preliminaries deliverables list and an information-required-to-commence list:

> This proposal includes the services described below and is divided into Stages 1-5. Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work. The deliverables outlined in this document are typical for a wide range of projects. Specific Deliverables will be agreed with the Client during Preliminaries.
>
> Preliminaries — Typical deliverables include:
> - Assistance in defining project aspirations
> - Assistance in project planning and brief development as appropriate
> - Review of site and project information
>
> emittiv requires the following information in order to commence:
> - Any particular requirements or aspirations
> - Any rules and regulations applying to the site
> - Design criteria and standards
> - Operational and functional requirements
> - Program and Area allocations
> - Existing project drawings, sketches, models and reports
> - Any established package budgets

Used by: 22-97113 MAF FEC FP-02

---

### 2. Concept Design Report

The Concept Design stage clause. All four instances are the same clause; the only variation is the discipline named in the report title ("Specialist Lighting", "Sensory Systems", "Audio and Lighting", "Lighting and Audio"). The bullet list and the "up to one revision" line are identical across all.

> We will prepare a [Discipline] Concept Design Report for the Project which will provide guidelines for the Client and other Designers to ensure that the [Discipline] concepts are in line with the overall Project aspirations. This report shall be based on coordination with other Designers / Consultants as well as requirements of the Client.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings
> - Briefing and presentation meetings with the Client
> - Carrying out conceptual design studies
> - Undertake research to establish current / future trends and establish design baselines
> - Provide recommendations on package requirements [such as lux levels, uniformities, resolution, sound levels, and intelligibility as required]
> - High level Package Master Plans
> - Produce presentation materials to convey the overall design intention
>
> We have allowed for up to one revision of the Concept Design Report.

Differences across instances: discipline token only — "Specialist Lighting" (2 docs), "Sensory Systems", "Audio and Lighting", "Lighting and Audio". Bullet marker varies (`-` vs `•`).

Used by: 22-97115 Kids Activity Park AUH, 23-96605 Jumpoline, 23-97109 Ciel VIP, 23-97111 Dubai Island Promenade, 25-97101 Shanghai Tang

---

### 3. Schematic Design Report (Schematic / Design Development phase)

The schematic-design clause. The opening sentence and the "up to one revision of the Schematic Design Report" closer are constant; the bullet list of preliminary deliverables varies by how much detail the project carried (number of "Prepare preliminary…" items).

Representative fuller form (most-common bullet set):

> In this design phase, the design is developed with a primary focus on achieving the ideas presented in the Concept Design phase.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings
> - Carry out further development of package design
> - Prepare preliminary 3D package calculations [as required]
> - Prepare preliminary package layouts
> - Prepare preliminary package detail sketches
> - Prepare preliminary package equipment schedule
> - Prepare preliminary package control intent
> - Prepare preliminary logical control channel schedule
> - Prepare preliminary package scene information
> - Prepare preliminary package power requirements
> - Assist others in checking that the package schemes are within the approved budgets
> - Identify integration points with other systems as required
>
> We have allowed for up to one revision of the Schematic Design Report.

Differences across instances (all otherwise the same clause):
- **Trimmed bullet set** (22-97115 Kids Activity Park AUH): drops detail sketches, scene info, and power; calc note reads "[as required Lighting Only]".
- **No scene-information bullet** (23-97111 Dubai Island Promenade).
- **Extra acoustic bullet** (23-97109 Ciel VIP): adds "Identify and recommend appropriate acoustic treatments as required".
- **LOD-200 / detailed bullets** (23-96603 Khobar Grand Mosque): adds "Carry out preliminary development of package specific 3D layouts based on linked files to LOD 200 standards".
- Bullet marker varies (`-` vs `•`); one instance (23-96605 Jumpoline) omits the blank line between the opener and "Typical deliverables include:".

Used by: 22-97115 Kids Activity Park AUH, 23-96603 Khobar Grand Mosque, 23-96605 Jumpoline, 23-97109 Ciel VIP, 23-97111 Dubai Island Promenade, 25-97101 Shanghai Tang

---

### 4. Detailed Design / Design Development (final package layouts → "Basis of Design")

The final-design clause taking the package to tender-ready / "Basis of Design" level. Two opening sentences appear, both leading into the same "Prepare final…" bullet family.

Representative form:

> In this design phase we further develop and identify the specifics of the systems to the level that they can be issued for tender and finalised by the awarded contractor for construction from a "Basis of Design" perspective.
>
> Typical deliverables include:
> - Attend design workshops / Design Team meetings / mock-ups
> - Prepare final package layouts
> - Assist others with the coordination of the package design
> - Prepare final package equipment schedule
> - Prepare final package details where applicable for incorporation into Architect's details
> - Prepare final 3D package calculations
> - Prepare final package power requirements
> - Specify integration points with other systems as required

Differences across instances:
- **Opening sentence variant**: "...integration with other design disciplines." (25-97105 Shanghai Tang v2, 25-97101 Shanghai Tang) instead of the "Basis of Design" wording.
- **LOD-300 / extra bullets** (23-96603 Khobar Grand Mosque): adds "Carry out further development of package specific 3D layouts based on linked files to LOD 300 standards" and "Prepare final package details … for incorporation into Architect's details".
- **"Prepare final package layouts - by Client"** (23-97111 Dubai Island Promenade): layouts noted as Client responsibility.
- **DD release note** (25-97105 Shanghai Tang v2): appends "Emittiv shall provide 50% and 100% document releases of the DD package to facilitate coordination between the Design Team Members."

Used by: 23-96603 Khobar Grand Mosque, 23-96605 Jumpoline, 23-97111 Dubai Island Promenade, 25-97105 Shanghai Tang v2, 25-97101 Shanghai Tang

---

### 5. Tender / Construction Documentation (issue for tender)

The tender-documentation clause. Body is constant across instances; the only variation is whether the opening sentence carries a perspective tag ("Open Tender" / plain).

> This stage will prepare the design documents so they can be issued for tender and finalised by the awarded Contractor for construction.
>
> Typical deliverables include:
> - Following approval provide documents for final tender and assistance in co-ordinating system design with architectural, electrical, structural and mechanical design.
> - Final system calculations and compliance with relevant Building Regulations.
> - Final specification and schedule of devices, accessories and associated equipment.
> - The above information will allow detailed costing by others – the Project's Cost Estimator / Quantity Surveyor / Tendering Contractor / other.
> - Final specification for the operational requirements of the control system and control schedule.

Differences across instances: 23-96603 Khobar Grand Mosque adds "...from a 'Open Tender' perspective." to the opening sentence. Hyphen vs en-dash before "the Project's Cost Estimator" varies. Bullet marker varies (`•`).

Used by: 23-96603 Khobar Grand Mosque, 25-97105 Shanghai Tang v2, 25-97101 Shanghai Tang

---

### 6. Submittal / substitution review (supervision)

> Typical deliverables include:
> • Review and checking of proposed substitute devices, as selected by others, for compliance with specification.
> • Review and analyse submittals based on technical compliance, company experience, project understanding, and other factors as required.
> • Provide recommendations to the Client based on previous experiences and local knowledge.

Used by: 24-97101 HoH Supervision

---

### 7. Procurement support (supervision)

> Typical deliverables include:
> • Attend regular meetings with Contractors and Client to ensure ongoing progress and development of the system.
> • Provide support to Contractor to develop installation programmes and timelines.
> • Provide support to Contractor and Client for procurement pathways.
> • Provide support to Contractor and Client to minimise lead times.
>
> This proposal allows for 9 days [envisaged to occur over 6 weeks] of work in this stage. This time will be split between Technical, Design, and Project Management staff as required.

Used by: 24-97101 HoH Supervision

---

### 8. Shop drawing / production information review (supervision)

> Typical deliverables include:
> • Attend workshop / meetings with Contractor to clarify questions from the system engineering team prior to commencement.
> • Review Contractors production information.
> • Ongoing resolution of design queries.
> • Provide recommendations to Client in order to resolve coordination issues as required.
>
> For clarity, Shop Drawing reviews are dependent on submittals by the Contractor. An agreement shall be reached with the Client on what shall constitute "Substantial Completion" and allow invoicing of the final payment for this stage. Any outstanding drawings shall be reviewed as they are submitted.

Used by: 24-97101 HoH Supervision

A shorter, related installation-review clause (no production-info / payment language) appears once:

> Typical deliverables include:
> - Review Contractors production information
> - Resolution of site queries
> - Final review of package installation.
> - The Package Contractor shall supply personnel to perform this work in compliance with the local legislation and union agreements.

Used by: 23-97111 Dubai Island Promenade

---

### 9. Site inspection / installation supervision

> Typical deliverables include:
> • Attend meetings with the Installation and Design Teams as considered necessary by the Design Team (the number of site meetings will be quantified).
> • Attend site inspections as agreed to review the progress and workmanship of the Contractor on site.
> • Provide feedback to the Client following site inspections.
> • Resolution of site queries.
> • Final review of package installation.
>
> Installation Supervision is estimated at 2 days per month over a 6 month installation period. This time can be broken up into 4 x 1/2 day visits or sessions as required, and spread over multiple months. Additional time has been allocated for weekly meetings, updates and reviews, and other general activities as required. Any unused time shall roll over into following months. Delays in construction timelines may require additional visits.

Used by: 24-97101 HoH Supervision

---

### 10. Commissioning / testing witness (supervision)

> Typical deliverables include:
> • Review and provide feedback on commissioning procedures developed by the Contractor.
> • Confirm addressing and grouping structures are appropriate for intended use.
> • Confirm control system is operational and ready for programming.
> • Witness functionality and grouping tests to confirm the system is ready for programming.
> • Coordinate addressing of fixtures with Programmers to streamline the works.
> • Direct aiming and focussing of adjustable devices.
>   - The Package Contractor shall supply personnel to perform this work in compliance with the local legislation and any union agreements.
>
> This proposal allows for 15 days on site for this stage. Typical staffing would include the Designer and 1 Assistant. Given the number of spaces in this project, multiple rooms should be ready for inspection on each visit.

Used by: 24-97101 HoH Supervision

A short focusing/aiming + programming-oversight variant (different wording, "Important Notes" block) appears once:

> Typical deliverables include:
> - Supervision of focusing / aiming of adjustable device.
> - Oversee the programming of installed control equipment where applicable.
>
> Important Notes:
> - This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> - The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the contractor and manufacturer.
> - Any content required for the system operation shall be installed / loaded by the contractor prior to programming activities commencing.
> - emittiv will oversee the scene setting of all package control systems with an engineer from the manufacturer of the systems [provided by the Contractor].

Used by: 23-97111 Dubai Island Promenade

---

### 11. Programming — design / pre-patching (off-site preparation)

> Typical deliverables include:
> • In conjunction with the Control System Contractor, develop and review lighting control documentation including Fixture IDs, Patch Sheets, and control connection diagrams.
> • In conjunction with the Client and Creative Team, develop a programming schedule defining the requirements and desired outcomes for each space.
> • Pre-patching of control software.
> • Establish preliminary presents in the control system to be updated on site.
> • Program preliminary and place-holder scenes and sequences [where possible] to be updated on site.

Used by: 24-97104 HoH Programming

---

### 12. Programming — on-site scene setting / validation

Two related but distinct on-site programming clauses appear.

Validation / walk-through form:

> Typical deliverables include:
> • Flash out and validate the installed lighting system to ensure all connected devices are responding as expected. Any issues or errors are to be addressed by the Lighting Contractors.
> • Confirm all preliminary programming scenes are operating as intended.
> • Update existing and new scenes to suit site conditions.
> • Implement and validate interactivity / triggers with site wide control system.
> • Initial walk through and comments with Client Team.
> • Update scenes as required.
> • Program and confirm any schedules as required.
> • Final walk through and sign off with Client Team.

Used by: 24-97104 HoH Programming

Scene-development form (with "Important Notes" and day allocation):

> Typical deliverables include:
> • Develop initial scene information.
> • Supervise the programming of installed control equipment.
> • Establish connectivity with site wide control devices.
>
> Important Notes:
> • This will be carried out following the manufacturer's commissioning of the system and after the Contractor's confirmation of all circuits / cables being correctly installed, labelled and operational.
> • The systems can only be focused / aimed and programmed when all the devices are installed i.e. fully operational to all areas as stated within the scope, devices and accessories as specified and control systems, fully commissioned by the Contractor and/or Manufacturer.
> • Other systems that impact or rely on the lighting scenes [screens, projection, cameras, signage etc.] must be in their operational states during the lighting programming.
> • Any content required for the system operation shall be installed / loaded by the Contractor prior to programming activities commencing.
>
> This proposal allows for 15 days of programming time in this stage. Typical staffing would include the Designer and 1 Assistant. Some works may be undertaken off site, prior to commencement, in order to facilitate a streamlined workflow.

Used by: 24-97101 HoH Supervision

---

### 13. Defects / snagging (and training where included)

Short defects-review form:

> Typical deliverables include:
> - Assistance with producing the package defects report to responsible areas.
> - Assistance in checking completion / making good of any defects noted in our system report and any other relevant reports.

Used by: 23-97111 Dubai Island Promenade

Extended defects + training form:

> Typical deliverables include:
> • Assistance with producing the package defects report for relevant areas.
> • Assistance in checking completion / making good of any defects noted in our system report and any other relevant reports.
> • Assistance in defining and developing training session[s] for operations staff to explain the functionality of the Lighting Control Systems.
> • Conduct some "Hands On" training with selected operations staff to familiarise them with required actions.
>
> This proposal allows for 5 days of work in this stage. This is intended to be completed by the Designer and will be broken down to allow for session planning and delivery for the Lighting Control System.
>
> The Contractor shall provide training for the operation and maintenance of the installed fittings and devices. These sessions shall be attended by emittiv to confirm sufficient information is provided to the operations staff.

Used by: 24-97101 HoH Supervision

---

### 14. Project-specific rate / deployment schedule (not a reusable clause)

This entry is a bespoke commercial deployment table (weekly role rates, phase fees, on/off-site configurations) rather than a deliverables clause. Retained for traceability; do not treat as a template clause.

> Review and Report Phase - Stage 1 and Stage 2 weekly deployment allocation by role (Partner AED 49,400/wk … Administrative Support AED 10,800/wk).
> Total Fees: AED 1,069,440 (Stage 1: AED 483,360 | Stage 2: AED 586,080)
> On-going Support and Validation Phase - Stage 3 weekly configurations: 3a Light Off-Site AED 10,680/wk … 3f Heavy On-Site AED 185,760/wk
> Typical expected deployment configurations during construction supervision stages. Indicative estimates only.

Used by: 25-97102 WAMI

---

**Recommended standard wording:**

The four core design-stage clauses are stable enough to canonicalise, with the discipline name as the only token to swap per project.

- **Proposal structure:** "This proposal includes the services described below and is divided into Stages 1-N. Each Stage will progress in sequence, after receiving approval confirmation from the Client for the previous Stage's work. The deliverables outlined in this document are typical for a wide range of projects. Specific Deliverables will be agreed with the Client during Preliminaries." (N = actual stage count.)

- **Concept Design Report** — use the Section 2 representative verbatim, substituting the discipline (e.g. "Lighting", "Audio and Lighting", "Sensory Systems") in both the title and the "concepts are in line with" sentence. Keep the full 7-bullet list and the "up to one revision of the Concept Design Report" closer.

- **Schematic Design Report** — use the Section 3 representative (the full 12-bullet set). Drop the scene-information / detail-sketch bullets only when the package genuinely excludes them; add the acoustic-treatment bullet only for audio/acoustic scopes; use the LOD-200 layout bullet when 3D modelling to a stated LOD is in scope. Always close with "We have allowed for up to one revision of the Schematic Design Report."

- **Detailed Design / Basis of Design** — use the Section 4 representative. Prefer the "...from a 'Basis of Design' perspective." opener for design-only engagements; use the "...integration with other design disciplines." opener where DD-package coordination releases (e.g. 50%/100%) are committed, in which case append that release note. Add LOD-300 layout bullets when 3D-to-LOD is in scope.

- **Tender Documentation** — use the Section 5 representative; add the perspective tag ("Open Tender") to the opening sentence only when the procurement route warrants it. Standardise on the en-dash form "by others – the Project's Cost Estimator…".

The supervision, programming, commissioning, defects and training clauses (Sections 6-13) are stage- and project-specific (day allocations, staffing, site conditions). No single canonical version is recommended; reuse the closest representative above and re-quantify days/staffing per engagement.

---

## Other

Clauses that don't fit the structured FP fields - company boilerplate, next-steps/LPO closings, document register blocks, confidentiality notices, insurance statements, qualifications, rate cards, team bios, and project-specific one-offs. Grouped by theme below.

---

### 1. Company introduction ("About emittiv")

The standard "who we are" boilerplate. Several near-identical variants differing only in: presence/absence of the project-type list, the website sign-off line, "Destination Locations"/"Attractions"/"Residential" in the type list, and paragraph spacing.

**1a. Full version (with type list + website line)** - the most common form (13 uses).

> emittiv is a multidisciplinary sensory design consultancy working with lighting, video, sound, scent, and control systems for the built environment. We design experiences, not just systems - making sure all of our elements work together to deliver a unified and polished concept.
> We work closely with Architects, Interior Designers, Lead Design Consultants and Owners / Operators on projects worldwide.
> Our aim is to enhance people's everyday lives by adding layers that make our spaces more interesting and beautiful.
> Our creative and commercial experience enables us to develop intelligent and appropriate designs within the parameters of budgets and time frames.
> We are entirely independent of any manufacturers or distributors, and operate on a fee for service basis.
>
> Our combined experience covers all types and scale of projects including:
> Attractions, Bars, Brand Activations, Casual Dining Venues, Cinemas, Corporate Headquarters, Department Stores, Destination Locations, Family Entertainment Centres, Galleries, Gaming Venues, Hotels, Landmark Buildings, Landscape, Malls, Museums, Nightclubs, Offices, Parks, Performance Venues, Public Realm, Residential, Retail Flagships, Shopping Centres, Spas, Theatres, Theme Parks.
>
> We can be appointed in a number of ways and tailor our services to a specific project and Clients' needs.
> All project teams are overseen by a Director and/or Associate to ensure a high level of service at all times.
> For further information on our services and project experience please visit our website: www.emittiv.com

Used by: 22-97115 (FP-02), 23-97105, 23-97107, 24-96602, 24-97104, 24-97107, 24-97108, 24-97111, 25-97102 (FP-02), 25-97105, 25-97106, 23-96606, 23-96604 +1 more

**1b. Variants (trivial differences):**
- Type list inline (one paragraph instead of wrapped), no website line - Used by: 23-97108, 24-97112, 24-97105 (3 uses).
- No type list and no website line (shortest form) - Used by: 25-97109, 25-97102 (FP-01 WDD notes).
- Blank line between every paragraph + bullet-dash type list, no website line, type list omits Attractions/Destination Locations/Residential - Used by: 22-97111.

---

### 2. Next steps / LPO closing

By far the most frequent clause type. The canonical "please issue an LPO" closing. All variants share the same three sentences plus a "Sincerely" sign-off; they differ only in (a) the document reference string, (b) hyphen vs bullet (•) separators in the signature, (c) "emittiv" vs "emittiv llc-fz", (d) "Founder and Lighting Director" vs "Lighting Director" vs "Founder and Audio Director" (Andrew Hawkes on audio-led proposals), and (e) line-wrapped vs single-paragraph body.

> If you would like to proceed with this project, please issue an LPO mentioning the document reference {DOC-REF} r{NN}.
> If you have any questions, or would like to discuss this further, please do not hesitate to contact us.
> We look forward to working with you on this project.
>
> Sincerely,
> Martin Robert - Founder and Lighting Director - emittiv llc-fz

Used by: 25-97103, 24-96606, 25-97102, 22-96601, 22-97114, 22-97115, 23-96601, 23-96602, 23-96603, 23-96604, 23-96605, 23-96607, 23-97101, 23-97102, 23-97106, 23-97107, 23-97108, 23-97109, 23-97111, 24-96601 +30 more

**Signature variants observed:**
- Hyphen separators + "emittiv llc-fz" (most common).
- Bullet (•) separators + "emittiv llc-fz".
- "emittiv" with no "llc-fz" suffix (older 2022-2023 proposals: 22-96601, 22-97113, 23-96601-style).
- "Founder and Audio Director" with Andrew Hawkes (23-97105 Tape Theatre - audio-led).
- "Lighting Director" without "Founder and" (26-96801 Rozana Muscat).

**Conditional / extended closings (same clause + extra sentence):**
- Acoustic-consultancy trigger: "If the LPO is issued for a value of AED {amount} it will be acknowledged that the Acoustic Consultancy has been ordered by the Client." - Used by: 24-97107 (87,500), 25-97101 (82,500 / 77,500), 24-97113 (87,500), 25-97105/97101 (82,500).
- Contract-clause acknowledgement: "We confirm that we have received the Contract General Clause [Appendix C] document... We are still reviewing this..." prepended to the closing - Used by: 24-97106.
- Insurance/T&C acceptance ("Additional Information" + "Next Steps") - Used by: 23-97102 (EL-FP).

---

### 3. Document register - Tracking & Distribution

The revision-control block listing date/release/author and the distribution list. Appears in several combinations: Tracking alone, Tracking + Distribution, or Tracking + Distribution + Confidential, with either table-style or "| "-delimited rows.

> Tracking
>
> Date: {YYMMDD} | Release: {NN} | Author: {INITIALS} | Reference: {reason}
>
> Distribution
>
> Date: {YYMMDD} | Release: {NN} | Distribution: {names}
>
> This document is maintained as part of a document register. Please confirm with the author that you are using the current version.

Used by: 22-97115, 23-97105, 24-97107, 24-97111, 24-97112, 23-97111, 24-96606, 25-97102, 25-97106, 24-97105 +N more

**Variants:**
- Tracking + Distribution + Confidential combined into one block - Used by: 23-96604, 23-96605, 24-97111, 25-97102, 23-97111.
- Column-header table form ("Date | Release | Author | Reference" with rows beneath) - Used by: 24-96606, 25-97106, 23-96606.
- Multi-release history (two+ rows) - Used by: 22-97115, 23-97111, 25-97102 (FP-02), 25-97106 (FP-02).

---

### 4. Confidentiality notice

Standalone confidentiality statement (also appears merged into the register block above).

> Confidential
> This document contains sensitive commercial information.
> It must not be reproduced or distributed without the express written permission of emittiv.
> This document and its contents are only applicable to the named project, for the named Client.

Used by: 22-97115; also embedded in the combined register blocks of 23-96604, 23-96605, 24-97111, 25-97102, 23-97111.

---

### 5. Insurance statements

Multiple distinct insurance clauses. Two main standing forms (PI + Third-Party) plus project-specific variations.

**5a. Professional Indemnity + Third-Party Liability (the standard pair, WAMI form):**

> Professional Indemnity
> emittiv maintains Professional Indemnity insurance coverage of $10m USD for each and every claim.
> This policy is currently being renewed and will be issued to the Client prior to the commencement for works.
> To date, there have been no previous claims against emittiv for any reason.
>
> Third-Party Liability
> emittiv DOES NOT currently maintain Third-Party or Public Liability coverage, as this does not typically cover our business activities or works.
> Please advise if this is a firm requirement for the WAMI project, and we will investigate available options.

Used by: 25-97102 (FP-01 and FP-02; also appears split into two separate clauses in the WDD-notes variant).

**5b. Insurance - provider-change form (older):**

> We are currently changing our insurance provider do better suit the nature of our business activities.
> Coverage details shall be provided prior to commencement of any works, if awarded.
> Our Professional Indemnity insurance shall cover up to 10 million AED for each and every claim.
> Our Workman's Compensation insurance shall cover construction site supervision activities and travel to and from site.
> To date, there have been zero claims against Emittiv L.L.C-FZ on any past or present projects.

Used by: 23-97106.

**5c. No Public Liability disclaimer (embedded in observations clause):** "emittiv does not carry or maintain Public Liability / Third Party insurance as this falls under the scope of installing contractors." - Used by: 24-97106 (within its RFP-observations clause).

---

### 6. Qualifications & previous works

The "Emittiv is qualified to undertake the proposed works" section with Specialist Knowledge + reference projects (Expo 2020, SeaWorld, House of Hype, Rua al Medina). Three near-identical variants differing only in formatting (pipe-delimited vs newline role/duration headers) and whether a "Previous Works" sub-heading is present.

> Emittiv is qualified to undertake the proposed works based on several factors including:
>
> Specialist Knowledge and Experience
> Unlike most other lighting consultants, we have extensive experience in a wide range of lighting styles and projects. We regularly work across entertainment, theatrical, hospitality, landscape, public realm, and even car park lighting. Our diverse skill set is an excellent match to the required tasks that will come up in this project. Additionally, our founder - Martin Robert - is an authority on lighting control protocols and systems.
>
> Expo 2020 | Role: Site Wide Lighting Consultant | Duration: 2019-2022
> [...]
> SeaWorld | Role: Specialist Consultant to Contractor | Duration: 2020-2021
> [...]
> House of Hype | Role: Lighting Designer | Duration: 2022-2025
> [...]
> Rua al Medina - SuperBlock 5 | Role: Lighting Designer | Duration: 2022-2025
> [...]

Used by: 25-97102 (FP-01 WDD notes, FP-01, FP-02 - three formatting variants).

---

### 7. Team / staff bios

Specialist team profiles. Two forms: a long-form with full career paragraphs + key experience, and a condensed form with one-line summaries.

**7a. Full bios (Martin Robert, Sky Bembury, Ryan Marginson, Daryl Bowen, Goran Salkovic + Notes):**

> Martin Robert — emittiv founder, Lighting and Control Specialist
> Specialties: Lighting, Video, Control Systems
> [...full career paragraphs and Key Experience per person...]
> Notes: The proposed staff are indicative of the calibre of specialists emittiv typically provide to our projects...

Used by: 25-97102 (FP-01).

**7b. Condensed bios** (same five people, one-line summaries) - Used by: 25-97102 (FP-01 WDD notes).

---

### 8. Rate cards (staff time-charge rates)

Tabular role/rate schedules. Distinct per year/proposal; not a reusable boilerplate clause but recurring in structure.

- **2025 full rate card** (Partner → Administrative Support, hour/day/week/month, with time definitions and VAT note) - Used by: 25-97102 (FP-01).
- **2025 staff weekly rates + stage fee totals + Stage 3 deployment configs** - Used by: 25-97102 (FP-01).
- **2023 rate card** (Design Manager → Technical Supervisor, half-day/day/week/month) - Used by: 23-97102 Wynn.

---

### 9. Company license details

> EMITTIV L.L.C-FZ
> License Number: 2204336.01
> Company Type: Limited Liability Company
> Formation Number: 2204336
> Address: Meydan Grandstand, 6th floor, Meydan Road, Nad Al Sheba, Dubai, U.A.E.
> Activities: Specialized design activities (Code 7410.00); Other professional, scientific and technical activities n.e.c. (Code 7490.00); Other specialized wholesale (Code 4660.00)
> Issue Date: 25/10/2022 | Expiry Date: 24/10/2025
> Managers: Martin John Robert
> Issued under the Meydan - Free Zone regulations.

Used by: 25-97102 (FP-01 WDD notes, FP-02 - minor field-order/phrasing differences).

---

### 10. RFP acknowledgement / reference documents

The "we have received a Request for Proposal" acceptance statement, usually followed by a list of client-provided reference documents. Project-specific (client name + document list vary).

> emittiv ["Consultant", "We"] have received a Request for Proposal from {Client} ["Client"].
>
> emittiv confirms that we are prepared to accept the appointment based on the information included in this proposal.
> emittiv confirms that we are suitably qualified and have the capacity to deliver this project to professional standards.
>
> Reference documents provided by the Client, which form the basis of this proposal include:
> • {document list}

Used by: 23-96607 (Hyperspace), 23-97107 (P&T Group), 24-97107 (Conrad Hotels).

---

### 11. Project description / packages (one-offs)

Project-specific scope/packages descriptions - not reusable boilerplate.

- Reserve Cut project description + Packages (Lighting/Audio + optional Acoustics) - Used by: 24-97107.

---

### 12. Conflict / dispute / quality declarations (Fountain Control set)

Short standalone declarations, all from 23-97106:

> We confirm that we are not currently aware of any existing or potential future conflicts of interest with regards to our execution of this project.

> We confirm that we are not currently aware of any existing or potential future disputes with other projects or Clients that would impact our execution of this project.

> Given the highly specific and bespoke nature of these works, standard Quality Management systems do not lend themselves to this type of work.
> Our internal procedures require all documents to be review by at least 2 specialists, however, and we regularly consult with manufacturers and other industry professionals to ensure our product is accurate and current.

Used by: 23-97106 (all three).

---

### 13. RFP observations / caveats (one-off)

Project-specific budget/programme challenge plus standing liability/regulations caveats.

> The RFP documents outline some items which we believe should be addressed at the outset.
> [...budget and programme observations...]
> Please note that as a design studio, emittiv does not carry or maintain Public Liability / Third Party insurance as this falls under the scope of installing contractors.
> Regulations / standards / landlord guidelines - It is the responsibility of the client to inform emittiv at time of appointment of any specific regulatory or energy efficiency standards / regulations / guidelines that need to be adhered to other than Part L or CIBSE...

Used by: 24-97106 (RAK Sled).

---

**Recommended standard wording:**

The two clauses worth standardising as canonical templates:

**Company introduction (clause 1a):** Use the full version with the project-type list and website line, hyphen-separated where punctuation is needed:

> emittiv is a multidisciplinary sensory design consultancy working with lighting, video, sound, scent, and control systems for the built environment. We design experiences, not just systems - making sure all of our elements work together to deliver a unified and polished concept.
> We work closely with Architects, Interior Designers, Lead Design Consultants and Owners / Operators on projects worldwide.
> Our aim is to enhance people's everyday lives by adding layers that make our spaces more interesting and beautiful.
> Our creative and commercial experience enables us to develop intelligent and appropriate designs within the parameters of budgets and time frames.
> We are entirely independent of any manufacturers or distributors, and operate on a fee for service basis.
>
> Our combined experience covers all types and scale of projects including:
> Attractions, Bars, Brand Activations, Casual Dining Venues, Cinemas, Corporate Headquarters, Department Stores, Destination Locations, Family Entertainment Centres, Galleries, Gaming Venues, Hotels, Landmark Buildings, Landscape, Malls, Museums, Nightclubs, Offices, Parks, Performance Venues, Public Realm, Residential, Retail Flagships, Shopping Centres, Spas, Theatres, Theme Parks.
>
> We can be appointed in a number of ways and tailor our services to a specific project and Clients' needs.
> All project teams are overseen by a Director and/or Associate to ensure a high level of service at all times.
> For further information on our services and project experience please visit our website: www.emittiv.com

**Next steps / LPO closing (clause 2):** Standardise on hyphen separators and the full "emittiv llc-fz" suffix, with the document reference and revision as the only variables:

> If you would like to proceed with this project, please issue an LPO mentioning the document reference {DOC-REF} r{NN}.
> If you have any questions, or would like to discuss this further, please do not hesitate to contact us.
> We look forward to working with you on this project.
>
> Sincerely,
> Martin Robert - Founder and Lighting Director - emittiv llc-fz

(Swap the signatory line to "Andrew Hawkes - Founder and Audio Director - emittiv llc-fz" on audio-led proposals; append the acoustic-consultancy trigger sentence when an optional acoustic price is offered.)

For the remaining themes (register block, confidentiality, insurance pair, qualifications, team bios) the WAMI 25-97102 FP-02 forms are the most recent and complete and should be treated as the reference wording. Rate cards, license details, and project-specific descriptions are inherently per-proposal and not canonicalised.

---

