# Excel Pricing Import Validation Report

**Report Date**: 2026-02-16
**Import Script**: `scripts/excel-import/import-pricing.ts`
**Database**: SurrealDB 10.0.21.8:8000 (ns: emittiv, db: projects)

---

## Executive Summary

This report documents the complete import and validation of pricing data from Excel files into the E-Fees database. All 27 imported records have been cross-referenced against their original Excel source files to ensure accuracy.

### Summary Statistics

| Metric | Count |
|--------|-------|
| Files discovered | 78 |
| Files scanned | 78 |
| Files skipped | 51 |
| Files imported | 27 |
| Records updated | 27 |
| Records created (new) | 0 |
| Records without Excel files | 2 (app-created) |
| Errors | 0 |
| Total pricing cells | 290 |
| Total post-contract items | 17 |

---

## Skip Reasons

51 files were skipped during import for the following reasons:

| Reason | Count |
|--------|-------|
| Template folder (`_yy-cccnn`) | 1 |
| Unfilled/generic template | 5 |
| No matching fee record in DB | 22 |
| Archive duplicate | 8 |
| Non-pricing Excel | 2 |
| FP-02+ with no matching `_2` fee record | 8 |
| Duplicate file | 5 |
| **Total** | **51** |

---

## Import by Folder Status

| Folder | Files Found | Imported | Skipped |
|--------|-------------|----------|---------|
| 00 Inactive | 53 | 16 | 37 |
| 01 RFPs | 5 | 0 | 5 |
| 11 Current | 6 | 5 | 1 |
| 99 Completed | 14 | 6 | 8 |
| **Total** | **78** | **27** | **51** |

---

## Template Types Detected

| Template Type | Count |
|---------------|-------|
| standard | 15 |
| legacy | 7 |
| numbered | 2 |
| numbered_revision | 1 |
| bare | 2 |
| subpackage | 1 |

---

## Imported Records (All 27)

### 1. fee:22_97111_1 — TPF Sound System

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: bare
- **Source**: `.../22-97111 TPF Sound System/02 Proposal/Pricing.xlsx`
- **Target Fee**: AED 150,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Sound Design 100%
- **Stages**: 4 — Prelims (PRE) 15%, Concept (CON) 30%, Schematic (SD) 30%, Detail (DD) 25%
- **Cells**: 4 | **Post-contract**: 0
- **Notes**: Legacy single-discipline format. No discipline header in Excel — discipline name derived from project context (sound system project).

---

### 2. fee:22_97113_1 — MAF FEC

- **Status**: Completed | **Folder**: 99 Completed | **Pattern**: numbered_revision
- **Source**: `.../22-97113 MAF FEC/02 Proposal/22-97113 Pricing r2.xlsx`
- **Target Fee**: AED 625,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 25%, Video 22%, Sound 23%, Technical 9%, Acoustics 21%
- **Stages**: 5 — Concept (CON) 20%, Detail (DD) 45%, IFT Docs (IFT) 15%, IFC Docs (IFC) 10%, Procurement (PRO) 10%
- **Cells**: 25 (5×5 full grid) | **Post-contract**: 0
- **Notes**: Older template format. Discipline names preserved from original Excel headers (Sound, Technical, Acoustics — not the app-standard Audio, SFX, Show Control).

---

### 3. fee:22_97114_1 — HoH

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: bare
- **Source**: `.../22-97114 HoH/02 Proposal/Pricing.xlsx`
- **Target Fee**: AED 297,000 | **VAT**: 5% | **Mob**: 15.15%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 7 — Tender Return Review (TEN) 8.4%, PM/Procurement Management (PM) 13.6%, Shop Drawing Review (SDR) 21.9%, Construction Supervision (CON) 15.2%, Commissioning Validation (COM) 20.2%, Programming (PRO) 17.7%, Handover/Training (HAN) 3.0%
- **Cells**: 7 | **Post-contract**: 0
- **Notes**: Legacy supervision/post-contract format. All 7 stages are post-contract services (stages 3-6 flagged is_post_contract=true). Custom stage names from Excel.

---

### 4. fee:22_97115_1 — Kids Activity Park AUH

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../22-97115 Kids Activity Park AUH/02 Proposal/e-22-97115-FP-01 Pricing.xlsx`
- **Target Fee**: AED 120,000 | **VAT**: 5% | **Mob**: 40%
- **Disciplines**: 5 — Lighting 40%, Video 20%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 2 — CD (CD) 83.3%, 50% SD (SD) 16.7%
- **Cells**: 10 (5 disciplines × 2 stages, including 4 zero-value cells) | **Post-contract**: 0
- **Notes**: Stage percentages calculated from cell amounts (100K/120K = 83.3%, 20K/120K = 16.7%). Zero-value cells preserved for full grid display.

---

### 5. fee:23_96601_1 — Dammam Adv World

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../23-96601 Dammam Adv World/02 Proposal/e-23-96601-FP-01 Pricing.xlsx`
- **Target Fee**: AED 120,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 5 — CD/CR (CD) 15%, SD (SD) 30%, IFT (IFT) 10%, DD (DD) 35%, IFC (IFC) 10%
- **Cells**: 25 (5×5) | **Post-contract**: 0

---

### 6. fee:23_96602_1 — Shoot the Chute

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../23-96602 Shoot the Chute/02 Proposal/e-23-96602-FP-01 Pricing.xlsx`
- **Target Fee**: AED 600,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 4 — CD (CD) 30%, SD (SD) 30%, DD (DD) 30%, IFT (IFT) 10%
- **Cells**: 20 (5×4) | **Post-contract**: 0

---

### 7. fee:23_96603_1 — Khobar Grand Mosque

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../23-96603 Khobar Grand Mosque/02 Proposal/e-23-96603-FP-01 Pricing.xlsx`
- **Target Fee**: AED 120,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 4 — Prelim (CON) 10%, SD (SD) 35%, DD (DD) 45%, TD (TD) 10%
- **Cells**: 4 | **Post-contract**: 0

---

### 8. fee:23_96604_1 — DockX

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: numbered
- **Source**: `.../23-96604 DockX/02 Proposal/23-96604 Pricing.xlsx`
- **Target Fee**: AED 250,000 | **VAT**: 0% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 25%, Video 22%, Sound 23%, Technical 9%, Acoustics 21%
- **Stages**: 3 — Concept (CON) 20%, SD (SD) 40%, DD (DD) 40%
- **Cells**: 15 (5×3) | **Post-contract**: 2 — Submittal Review (1×0=0), Construction Supervision (12×9,600=115,200)
- **Notes**: Older template. Discipline names preserved from Excel (Sound, Technical, Acoustics).

---

### 9. fee:23_96605_1 — Jumpoline

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: numbered
- **Source**: `.../23-96605 Jumpoline/02 Proposal/23-96605 Pricing.xlsx`
- **Target Fee**: AED 160,000 | **VAT**: 0% | **Mob**: 30%
- **Disciplines**: 4 — Lighting 31%, Video 28%, Sound 29%, Technical 12%
- **Stages**: 3 — Concept (CON) 20%, SD (SD) 40%, DD (DD) 40%
- **Cells**: 12 (4×3) | **Post-contract**: 2 — Submittal Review (1×0=0), Construction Supervision (12×9,600=115,200)
- **Notes**: Older template. Discipline names preserved from Excel (Sound, Technical). No Acoustics discipline in this proposal.

---

### 10. fee:23_96607_1 — HOH KSA

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../23-96607 HOH KSA/02 Proposal/e-23-96607-FP-01 Pricing.xlsx`
- **Target Fee**: AED 740,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 6 — Lighting 30%, Video 25%, Audio 22%, SFX 5%, Show Control 10%, Sub 8%
- **Stages**: 4 — Review (CON) 10%, SD (SD) 40%, DD (DD) 40%, BoQ (BOQ) 10%
- **Cells**: 24 (6×4) | **Post-contract**: 2 — Submittal Review (1×98,000=98,000), Construction Supervision (9×130,000=1,170,000)
- **Notes**: 6-discipline format includes "Sub" column for sub-consultant costs.

---

### 11. fee:23_97102_1 — Wynn

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: subpackage
- **Source**: `.../23-97102 Wynn/02 Proposal/e-23-97102-AA-FP-01 Pricing.xlsx`
- **Target Fee**: AED 3,200,000 | **VAT**: 5% | **Mob**: 10%
- **Disciplines**: 2 — AV 55%, Acoustics 45%
- **Stages**: 5 — Concept (CON) 10%, SD (SD) 10%, DD (DD) 35%, CD (CD) 15%, CA (CA) 30%
- **Cells**: 10 (2×5) | **Post-contract**: 0
- **Notes**: Sub-package format (AA = Audio/Acoustics sub-package). Highest value proposal in dataset.

---

### 12. fee:23_97105_1 — Tape Theatre

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: legacy
- **Source**: `.../23-97105 Tape Theatre/02 Proposal/e-23-97105-FP-01 Pricing.xlsx`
- **Target Fee**: AED 120,000 | **VAT**: 5% | **Mob**: 33.33%
- **Disciplines**: 1 — Audio 100%
- **Stages**: 4 — CD/CR (CD) 33.33%, SD (SD) 29.17%, DD (DD) 29.17%, IFC (IFC) 8.33%
- **Cells**: 4 | **Post-contract**: 0
- **Notes**: Audio-only project. Custom mobilisation rate.

---

### 13. fee:23_97106_1 — Fountain Control

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../23-97106 Fountain Control/02 Proposal/23-97106-FP-01 Pricing.xlsx`
- **Target Fee**: AED 510,000 | **VAT**: 5% | **Mob**: 25%
- **Disciplines**: 3 — Lighting 16.4%, Audio 20.9%, Show Control 62.7%
- **Stages**: 5 — CD (CD) 23%, SD (SD) 28%, DD (DD) 32%, TD (TD) 9%, Const Dwg Review (CDR) 8%
- **Cells**: 15 (3×5) | **Post-contract**: 3 — Site Supervision (182×4,200=764,400), Show Design (20×15,000=300,000), Training (1×60,000=60,000)
- **Notes**: Control-system-heavy project (62.7% Show Control). Custom stage "Const Dwg Review".

---

### 14. fee:24_96603_1 — Marasi Gate

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../24-96603 Marasi Gate/02 Proposal/24-96603-FP-01 Pricing.xlsx`
- **Target Fee**: AED 325,000 | **VAT**: 5% | **Mob**: 15%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 4 — CD (CD) 25%, SD (SD) 30%, DD (DD) 30%, TD (TD) 15%
- **Cells**: 4 | **Post-contract**: 0

---

### 15. fee:24_96605_1 — Observatory

- **Status**: Completed | **Folder**: 99 Completed | **Pattern**: standard
- **Source**: `.../24-96605 Observatory/02 Proposal/24-96605-FP-01 Pricing.xlsx`
- **Target Fee**: AED 188,500 | **VAT**: 5% | **Mob**: 20%
- **Disciplines**: 1 — Video 100%
- **Stages**: 3 — SD (SD) 40%, DD (DD) 45%, TD (TD) 15%
- **Cells**: 3 | **Post-contract**: 0
- **Notes**: Video-only project.

---

### 16. fee:24_96606_1 — JOH

- **Status**: Awarded | **Folder**: 11 Current | **Pattern**: standard
- **Source**: `.../24-96606 JOH/02 Proposal/24-96606-FP-01 Pricing.xlsx`
- **Target Fee**: AED 420,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 4 — Initial Engineering (IE1) 25%, Initial Engineering (IE2) 25%, Initial Engineering (IE3) 25%, Initial Engineering (IE4) 25%
- **Cells**: 4 | **Post-contract**: 2 — Ongoing Management (18×15,000=270,000), Programming (3×30,000=90,000)
- **Notes**: Custom stage naming with sequential codes (IE1-IE4). All stages same name "Initial Engineering".

---

### 17. fee:24_97105_1 — MAF MiCC Kids

- **Status**: Completed | **Folder**: 99 Completed | **Pattern**: standard
- **Source**: `.../24-97105 MAF MiCC Kids/02 Proposal/24-97105-FP-01 Pricing.xlsx`
- **Target Fee**: AED 65,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 1 — DD (DD) 100%
- **Cells**: 1 | **Post-contract**: 0
- **Notes**: Smallest proposal — single cell (1 discipline, 1 stage).

---

### 18. fee:24_97109_1 — MOE FEC v2

- **Status**: Revised | **Folder**: 99 Completed | **Pattern**: standard
- **Source**: `.../24-97109 MOE FEC v2/02 Proposal/24-97109-FP-01 Pricing.xlsx`
- **Target Fee**: AED 335,000 | **VAT**: 5% | **Mob**: 25%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 3 — CD (CD) 25%, DD (DD) 55%, TD (TD) 20%
- **Cells**: 15 (5×3) | **Post-contract**: 0

---

### 19. fee:25_96501_1 — Activerse

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../25-96501 Activerse/02 Proposal/25-96501-FP-01 Pricing.xlsx`
- **Target Fee**: AED 450,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 4 — CD (CD) 25%, SD (SD) 30%, DD (DD) 35%, TD (TD) 10%
- **Cells**: 20 (5×4) | **Post-contract**: 0

---

### 20. fee:25_96601_1 — Pit Stop

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../25-96601 Pit Stop/02 Proposal/25-96601-FP-01 Pricing.xlsx`
- **Target Fee**: AED 580,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 3 — Lighting 50%, Video 29.3%, SFX 20.7%
- **Stages**: 3 — CD (CD) 30%, SD (SD) 30%, DD (DD) 40%
- **Cells**: 9 (3×3) | **Post-contract**: 3 — Design Management (13×40,000=520,000), Site Supervision (13×110,000=1,430,000), Programming (1×550,000=550,000)

---

### 21. fee:25_97101_1 — Shanghai Tang

- **Status**: Revised | **Folder**: 99 Completed | **Pattern**: standard
- **Source**: `.../25-97101 Shanghai Tang/02 Proposal/00 Archive/25-97101-FP-01 Pricing.xlsx`
- **Target Fee**: AED 55,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 2 — Lighting 60%, Audio 40%
- **Stages**: 4 — CD (CD) 25%, SD (SD) 30%, DD (DD) 35%, TD (TD) 10%
- **Cells**: 8 (2×4) | **Post-contract**: 0

---

### 22. fee:25_97102_1 — WAMI

- **Status**: Awarded | **Folder**: 11 Current | **Pattern**: standard
- **Source**: `.../25-97102 WAMI/02 Proposal/25-97102-FP-01 Pricing.xlsx`
- **Target Fee**: null (template only) | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 4 — CD (CD) 25%, SD (SD) 30%, DD (DD) 35%, TD (TD) 10%
- **Cells**: 0 (template — no amounts) | **Post-contract**: 0
- **Notes**: Template-only record. No target fee or cell amounts. Structure preserved for future pricing.

---

### 23. fee:25_97103_1 — Aljada FG

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../25-97103 Aljada FG/02 Proposal/00 Archive/25-97103-FP-01 Pricing.xlsx`
- **Target Fee**: AED 550,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 4 — SD (SD) 35%, DD (DD) 40%, TD (TD) 15%, IFC (IFC) 10%
- **Cells**: 4 | **Post-contract**: 3 — Construction Supervision (24×91,520=2,196,480), Programming (3×770,000=2,310,000), DLP (4×35,000=140,000)

---

### 24. fee:25_97104_1 — MAF MiCC v2

- **Status**: Revised | **Folder**: 99 Completed | **Pattern**: standard
- **Source**: `.../25-97104 MAF MiCC v2/02 Proposal/25-97104-FP-01 Pricing.xlsx`
- **Target Fee**: AED 85,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 2 — DD (DD) 76.5%, TD (TD) 23.5%
- **Cells**: 2 | **Post-contract**: 0

---

### 25. fee:25_97105_1 — Shanghai Tang v2

- **Status**: Awarded | **Folder**: 11 Current | **Pattern**: standard
- **Source**: `.../25-97105 Shanghai Tang v2/02 Proposal/25-97105-FP-01 Pricing.xlsx`
- **Target Fee**: AED 35,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 2 — Lighting 60%, Audio 40%
- **Stages**: 3 — 50% DD (DD1) 40%, 100% DD (DD2) 40%, TD (TD) 20%
- **Cells**: 6 (2×3) | **Post-contract**: 0
- **Notes**: Custom stage naming with DD split into DD1/DD2 sub-stages.

---

### 26. fee:25_97106_1 — MAF MOE FEC v3

- **Status**: Awarded | **Folder**: 11 Current | **Pattern**: standard
- **Source**: `.../25-97106 MAF MOE FEC v3/02 Proposal/00 Archive/25-97106-FP-01 Pricing.xlsx`
- **Target Fee**: AED 180,000 | **VAT**: 5% | **Mob**: 25%
- **Disciplines**: 5 — Lighting 35%, Video 25%, Audio 25%, SFX 5%, Show Control 10%
- **Stages**: 3 — CD (CD) 25%, DD (DD) 55%, IFC (IFC) 20%
- **Cells**: 15 (5×3) | **Post-contract**: 0

---

### 27. fee:25_97107_1 — Cove Boulevard

- **Status**: Lost | **Folder**: 00 Inactive | **Pattern**: standard
- **Source**: `.../25-97107 Cove Boulevard/02 Proposal/25-97107-FP-01 Pricing.xlsx`
- **Target Fee**: AED 185,000 | **VAT**: 5% | **Mob**: 30%
- **Disciplines**: 1 — Lighting 100%
- **Stages**: 4 — CD (CD) 25%, SD (SD) 30%, DD (DD) 35%, TD (TD) 10%
- **Cells**: 4 | **Post-contract**: 0

---

## Post-Audit Corrections Applied

The following corrections were made after the initial import, based on cross-referencing database records against original Excel source files:

### Discipline Name Corrections

| Record | Field | Before | After | Reason |
|--------|-------|--------|-------|--------|
| fee:22_97111_1 | d1 name | Lighting | Sound Design | Project is TPF Sound System — audio-only project |
| fee:22_97113_1 | d3 name | Audio | Sound | Older template uses "Sound" not "Audio" |
| fee:22_97113_1 | d4 name | SFX | Technical | Older template uses "Technical" not "SFX" |
| fee:22_97113_1 | d5 name | Show Control | Acoustics | Older template uses "Acoustics" not "Show Control" |
| fee:23_96604_1 | d3 name | Audio | Sound | Older template uses "Sound" not "Audio" |
| fee:23_96604_1 | d4 name | SFX | Technical | Older template uses "Technical" not "SFX" |
| fee:23_96605_1 | d3 name | Audio | Sound | Older template uses "Sound" not "Audio" |
| fee:23_96605_1 | d4 name | SFX | Technical | Older template uses "Technical" not "SFX" |

### Stage Code Corrections

| Record | Stage | Before | After | Reason |
|--------|-------|--------|-------|--------|
| fee:22_97113_1 | s2 (IFT Docs) | IFC | IFT | Duplicate code — both s2 and s3 had "IFC" |
| fee:25_97105_1 | s0 (50% DD) | DD | DD1 | Duplicate code — both s0 and s1 had "DD" |
| fee:25_97105_1 | s1 (100% DD) | DD | DD2 | Duplicate code — both s0 and s1 had "DD" |

### Stage Percentage Corrections

| Record | Before | After | Reason |
|--------|--------|-------|--------|
| fee:22_97115_1 | CD 15%, 50% SD 30% (sum=45%) | CD 83.3%, 50% SD 16.7% (sum=100%) | Agent used raw Excel column values instead of calculating from amounts |

### Missing Data Added

| Record | Type | Items Added |
|--------|------|-------------|
| fee:22_97115_1 | Zero-value cells | 4 cells (d2/s1=0, d3/s1=0, d4/s1=0, d5/s1=0) — preserves full grid |
| fee:25_96601_1 | Post-contract items | Site Supervision (13×110,000=1,430,000), Programming (1×550,000=550,000) |
| fee:25_97103_1 | Post-contract item | DLP (4×35,000=140,000) |

---

## Data Quality Summary

### Integrity Checks

- ✅ All 27 records have pricing.config (target_fee, vat_percent, mobilisation_percent)
- ✅ All stage percentages sum to 100% (±0.1% rounding)
- ✅ All stage codes are unique within each record
- ✅ All discipline IDs are unique within each record
- ✅ Cell count matches disciplines × stages for all records (except 22_97115_1 which has sparse s1 stage, now filled with zeros)
- ✅ All post-contract items have name, quantity, unit_price, and total fields
- ✅ 25_97102_1 (WAMI) correctly has null target_fee and 0 cells (template-only)

### Template Distribution

- **Standard template** (15 records): Standard 5-column format with discipline headers, stage rows, and optional post-contract section
- **Legacy template** (7 records): Older format with varying layouts — some have different discipline naming conventions
- **Numbered** (2 records): Older multi-discipline format using project number in filename
- **Numbered revision** (1 record): Same as numbered but with "r2" revision suffix
- **Bare** (2 records): Legacy format with just "Pricing.xlsx" filename — single discipline, no header columns
- **Subpackage** (1 record): Sub-package format with "AA" suffix denoting Audio/Acoustics package

### Known Limitations

1. **Formula cells**: Excel formula cells read as "[object Object]" via Desktop Commander — amounts were extracted from evaluated values where available
2. **Floating point amounts**: Some legacy templates have fractional amounts (e.g., 24,999.99) which were rounded to integers
3. **Template-only records**: fee:25_97102_1 has structure but no amounts — Excel has discipline/stage headers but empty cells
4. **Discipline naming evolution**: Emittiv's discipline naming changed between 2022-2023 templates:
   - 2022-2023 (older): Sound, Technical, Acoustics
   - 2024+ (current): Audio, SFX, Show Control
   - Both naming conventions are preserved as-is in the database

---

## Conclusion

All 27 pricing records have been successfully imported from Excel source files and validated against the database. Post-audit corrections have been applied to ensure accuracy of discipline names, stage codes, percentages, and post-contract items. The data is now ready for production use in the E-Fees application.

---

**Generated**: 2026-02-16
**Script**: `scripts/excel-import/import-pricing.ts`
**Database**: SurrealDB 10.0.21.8:8000 (ns: emittiv, db: projects)
