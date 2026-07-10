# Clause Selection Stage 3 - Corpus Suggestion: Design Proposal

**Status:** proposal (build gated - see decision below)
**Author:** e-fees instance (dispatched), 2026-07-10
**Context:** Phase 3 clause-selection Stages 1 and 2 are merged (PR #9, PR #16). The Stage 1
completion note (obs:zs8wqzhh1dal5fbjkyic) named "Stage 3 (corpus suggestion)" as the
explicit follow-on, without specifying a mechanism.

## Why this is a design doc, not a PR

The dispatch mission for this stage carried a build-vs-design gate: build directly only if a
conservative, deterministic mechanism falls out of *existing* data + patterns, the way Stage 2
did (Stage 2 reused Stage 1's `is_default` + `conditions` fields and the existing
`condition_matches` subset-match helper - zero new schema, zero new infra). That path does not
exist for Stage 3. Three facts from the current codebase pin it to the gate:

1. **No corpus-to-clause link exists.** `clause` (e-fees-scope/src/models.rs:9-23) has no
   usage-frequency or corpus-provenance field. `proposal_corpus` (models.rs:60-70) has no
   reference back to which library clauses it "contains." The only usage-frequency data that
   exists at all is hand-authored prose in `docs/clause-corpus/CLAUSE-CATALOG.md` ("Used by:
   23-97106 Fountain Control, 24-96603 Marasi Gate, ...") - narrative markdown from a one-time
   mining pass (2026-06-14), not a queryable table.
2. **Embeddings are reserved but unwired.** `ProposalCorpus.embedding: Option<Vec<f64>>` exists
   in the struct and is checked with `.is_some()` to set a `has_embedding` display flag
   (corpus.rs:48-49, 70-71) - but nothing in the ingestion pipeline (`do_ingest`,
   `extract_via_vision`, `extract_via_docling`) ever generates or stores a value into it. There
   is no vector index in `e-fees-scope/schema.surql`, and no call to an embedding model
   (Ollama's `nomic-embed-text`, used elsewhere in the workspace) anywhere in e-fees-scope.
3. **Naive text matching is unsound, by the project's own audit.** `docs/clause-corpus/
   LIBRARY-AUDIT.md` compared the 21 curated library clauses against the mined corpus and found
   several **"OUTDATED / DIVERGENT"** (e.g. "Defined Role": the curated body uses a
   `[Company Name]` placeholder and omits a regulations paragraph present in 61/69 historical
   proposals) and **"THIN"** verdicts. Curated clause bodies do not match historical proposal
   wording verbatim, so `string::contains`-style matching between `clause.body` and
   `proposal_corpus.extracted_text` (the pattern already used in `search_corpus`, corpus.rs:904-926)
   would have unreliable recall - it would miss real matches whenever the curated wording has
   drifted from the source text, which the audit shows is common.

Because neither "reuse an existing field" nor "reuse an existing matching pattern" is sound
here, a real Stage 3 needs a genuine infrastructure decision. That decision should be Martin's
to make explicitly (cost/complexity trade-off), not implicit in a PR diff.

## Options

### Option A - Embedding-based semantic similarity
Generate embeddings for `proposal_corpus.extracted_text` at ingest time (Ollama
`nomic-embed-text`, 10.0.21.20:11434 - already used elsewhere in the workspace) and for
`clause.body` at clause-create/update time. Add a SurrealDB vector index (MTREE/HNSW) on both.
At selection time, embed the current fee's known context (discipline, project description) and
run a similarity search to surface the nearest clauses.

- **Pros:** Most semantically flexible; matches the schema's apparent original intent (the
  `embedding` field was pre-provisioned for this). Naturally extends `search_corpus`
  (currently CONTAINS-only) toward real semantic search, a plausible future need regardless of
  Stage 3.
- **Cons:** New dependency wiring (embedding calls plus retry/error handling in the ingest and
  clause-write paths), new vector index plus SurrealDB v3 index-syntax research, ongoing
  embedding cost per corpus doc and per clause write, and, critically, text-to-text similarity
  between a fee's description and a PDF doesn't by itself say *which library clause* to
  suggest; it still needs Option B's clause-to-corpus link to turn "similar past proposal" into
  "suggest this clause."

### Option B - Structured usage-frequency table (LLM-classified, one-time mining job)
New table `clause_corpus_stat` (`clause_id`, `category`, `usage_count`, `sample_project_numbers[]`,
`classified_at`). Populate it with a one-time (then periodic, on corpus/library growth) mining
pass: for each active library clause, ask the LLM to classify each corpus document as
"contains an equivalent clause / does not" (structured yes/no plus confidence, not verbatim
matching, directly addressing the divergent-wording problem the audit found). At
selection time, `GET /scope/{fee_id}/clause-suggestions` returns currently-unselected clauses
ranked by `usage_count`, filtered to the fee's discipline(s) via the existing
`fee.disciplines[].discipline_id` field, with a "used in N of 69 historical proposals" badge -
auditable, unlike an opaque similarity score.

- **Pros:** Deterministic once mined (no live embedding calls at request time); produces a
  number Martin can sanity-check against the corpus directly; doesn't require standing up
  vector search infra; reuses the discipline field that already exists on `fee`.
- **Cons:** Still a schema addition (one new table) and a new LLM-assisted mining job - bounded,
  but real infrastructure, not "add a field." Mining cost is roughly 21 curated clauses times 69
  corpus docs, about 1,449 classification calls (batchable, one-time).

### Option C - Ship nothing new; treat Stage 2's `is_default` set as "the corpus answer"
Reject. This is already delivered by Stage 2 and doesn't use the corpus at Stage 3's stated
scope ("corpus suggestion" implies going beyond the static is_default flag).

## Recommendation

**Option B**, scoped as its own follow-on dispatch:
1. Schema: add `clause_corpus_stat` table (`e-fees-scope/schema.surql`).
2. Mining: new route or one-off script that LLM-classifies each (clause x corpus-doc) pair and
   upserts `clause_corpus_stat` - reuse the existing Ollama wiring pattern from
   `extract_clauses` (corpus.rs:641-831) but with structured classification output instead of
   free-form extraction, and instruct against verbatim-matching bias since the audit shows
   curated wording diverges from source text.
3. Suggestion endpoint: `GET /scope/{fee_id}/clause-suggestions`, filtered by
   `fee.disciplines[].discipline_id`, ranked by `usage_count`, excluding clauses already
   `included` in the fee's current selection.
4. Frontend: a "Suggested" section/badge in `ClausePicker.svelte`, additive to the existing
   Stage 1/2 picker, never auto-included, always an explicit user action to add.

Option A is worth revisiting later, but only if/when the app needs general semantic search over
the corpus for a reason broader than clause suggestion (e.g. free-text "find a proposal like
this one"), at which point Stage 3 could ride on that infrastructure instead of justifying it
alone.

## Scope fence (unchanged from this dispatch)

Clause **text/wording** stays owner-gated (Martin review pending per the existing
`clause-library-business-review` mission-record item) regardless of which option is chosen;
neither option modifies clause content, both only add ranking/suggestion metadata around the
existing library.
