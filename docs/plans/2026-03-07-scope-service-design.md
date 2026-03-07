# Scope/Deliverables Service Design

**Date:** 2026-03-07
**Status:** Approved
**Author:** Martin + Claude

## Overview

Standalone microservice (`e-fees-scope`) for managing a clause library, assembling numbered scope documents from building blocks, and using LLM to contextualize scope text. Includes a corpus/knowledge base built from 66+ historical fee proposal PDFs for RAG-augmented generation.

## Architecture

```
┌─────────────┐     ┌──────────────────┐     ┌────────────┐
│  e-fees UI  │────▶│  e-fees-scope    │────▶│  SurrealDB │
│  (Tauri)    │     │  :3201 (axum)    │     │  10.0.23.11│
└─────────────┘     │                  │     └────────────┘
                    │  ┌────────────┐  │
┌─────────────┐     │  │ LLM Client │──┼────▶┌────────────┐
│  e-fees-api │────▶│  └────────────┘  │     │   Ollama   │
│  :3200      │     │                  │     │ 10.0.21.20 │
└─────────────┘     └──────────────────┘     │  :11434    │
                                             └────────────┘
```

- **Container:** Standalone Docker on AI server (br0 network, port 3201)
- **Language:** Rust (axum 0.8), same toolchain as e-fees-api
- **Database:** SurrealDB 10.0.23.11:8000 (ns: emittiv, db: projects) — same instance, new tables
- **LLM:** Ollama at 10.0.21.20:11434, Qwen3.5 (must set `"think": false`)

## Data Model

### `clause` — Library of Reusable Scope Building Blocks

```surql
DEFINE TABLE clause SCHEMAFULL;

DEFINE FIELD category      ON clause TYPE string;         -- "Concept Design", "Detailed Design", etc.
DEFINE FIELD subcategory   ON clause TYPE option<string>;  -- finer grouping within category
DEFINE FIELD title         ON clause TYPE string;         -- short label
DEFINE FIELD body          ON clause TYPE string;         -- template text, supports {{placeholders}}
DEFINE FIELD conditions    ON clause TYPE option<object>;  -- when to auto-include
  -- conditions.control_types: ["DALI", "DMX", "0-10V"]
  -- conditions.fixture_types: ["custom", "standard", "decorative"]
  -- conditions.regions: ["UAE", "KSA", "International"]
  -- conditions.project_types: ["hospitality", "residential", "commercial"]
DEFINE FIELD sort_order    ON clause TYPE int;            -- ordering within category
DEFINE FIELD tags          ON clause TYPE option<array<string>>;
DEFINE FIELD is_default    ON clause TYPE bool DEFAULT true;  -- included unless excluded
DEFINE FIELD status        ON clause TYPE string DEFAULT "active";  -- active | archived
DEFINE FIELD version       ON clause TYPE int DEFAULT 1;
DEFINE FIELD created_at    ON clause TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at    ON clause TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_clause_category ON clause FIELDS category;
DEFINE INDEX idx_clause_status   ON clause FIELDS status;
```

### `scope_assembly` — Generated Scope for a Specific Fee

```surql
DEFINE TABLE scope_assembly SCHEMAFULL;

DEFINE FIELD fee_id         ON scope_assembly TYPE record<fee>;
DEFINE FIELD clauses        ON scope_assembly TYPE array;
  -- Each element: { clause_id: record<clause>, override_body: option<string>, included: bool }
DEFINE FIELD generated_text ON scope_assembly TYPE string;       -- final assembled output
DEFINE FIELD numbering      ON scope_assembly TYPE option<object>; -- numbering scheme used
DEFINE FIELD llm_model      ON scope_assembly TYPE option<string>;
DEFINE FIELD llm_polished   ON scope_assembly TYPE bool DEFAULT false;
DEFINE FIELD created_at     ON scope_assembly TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at     ON scope_assembly TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_scope_fee ON scope_assembly FIELDS fee_id UNIQUE;
```

### `proposal_corpus` — Historical Proposal Knowledge Base

```surql
DEFINE TABLE proposal_corpus SCHEMAFULL;

DEFINE FIELD filename       ON proposal_corpus TYPE string;
DEFINE FIELD project_number ON proposal_corpus TYPE option<string>;  -- extracted YY-CCCNN
DEFINE FIELD project_name   ON proposal_corpus TYPE option<string>;
DEFINE FIELD extracted_text ON proposal_corpus TYPE string;          -- full text from PDF
DEFINE FIELD sections       ON proposal_corpus TYPE option<array>;   -- parsed scope sections
  -- Each: { heading: string, body: string, page: int }
DEFINE FIELD metadata       ON proposal_corpus TYPE option<object>;
  -- metadata.date, metadata.client, metadata.region, metadata.project_type
DEFINE FIELD embedding      ON proposal_corpus TYPE option<array<float>>;  -- for vector search
DEFINE FIELD created_at     ON proposal_corpus TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_corpus_project ON proposal_corpus FIELDS project_number;
```

**Vector search** on `embedding` field enables RAG: when generating scope, retrieve similar past proposals for context.

## API Endpoints

### Clause CRUD

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/clauses` | List clauses (filter: `?category=`, `?status=`, `?tag=`) |
| `GET` | `/clauses/{id}` | Get single clause |
| `POST` | `/clauses` | Create clause |
| `PUT` | `/clauses/{id}` | Update clause (increments version) |
| `DELETE` | `/clauses/{id}` | Archive clause (soft delete) |
| `GET` | `/clauses/categories` | List distinct categories with counts |

### Scope Generation

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/scope/generate` | Assemble scope for a fee ID |
| `GET` | `/scope/{fee_id}` | Get assembled scope |
| `PUT` | `/scope/{fee_id}` | Edit scope (manual clause overrides) |
| `POST` | `/scope/{fee_id}/regenerate` | Re-polish with LLM |
| `GET` | `/scope/{fee_id}/export` | Export as structured JSON (for InDesign) |

### Corpus / Knowledge Base

| Method | Path | Purpose |
|--------|------|---------|
| `POST` | `/corpus/ingest` | Ingest a PDF (send to Docling, extract, store) |
| `POST` | `/corpus/ingest-batch` | Batch ingest from a directory path |
| `GET` | `/corpus` | List ingested documents |
| `GET` | `/corpus/{id}` | Get document with extracted sections |
| `GET` | `/corpus/search` | Vector similarity search across corpus |

### Health

| Method | Path | Purpose |
|--------|------|---------|
| `GET` | `/health` | Service health + DB + Ollama status |

## Scope Generation Flow

```
1. Receive fee_id
2. Fetch fee + project from SurrealDB (name, city, country, control type, etc.)
3. Select clauses:
   a. All where is_default = true AND status = "active"
   b. Plus condition-matched (control_types ∩ project context, etc.)
   c. Minus any explicitly excluded
4. Order: category sort_order → subcategory → clause sort_order
5. Auto-number hierarchically:
   1.0 Concept Design
     1.1 Lighting concept development...
     1.2 Initial fixture selection...
   2.0 Detailed Design
     2.1 Lighting layout drawings...
6. Substitute placeholders: {{project_name}}, {{city}}, {{client_name}}, etc.
7. RAG retrieval: find 2-3 similar past proposals from corpus (vector search)
8. LLM polish (Ollama/Qwen3.5):
   - System: "You are a lighting design consultant writing scope of services."
   - Context: project details + similar past scope sections
   - Instruction: "Refine these scope clauses for professional tone and
     project specificity. Maintain structure and numbering. Do not add
     or remove deliverables."
   - Must set: { "think": false } in Ollama request
9. Store assembly + generated text in scope_assembly
10. Return assembled scope
```

## PDF Ingestion Pipeline

**Source:** `/mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/99 All RFPs/` (66 PDFs)
**Secondary:** `98 All RFPs/` (older 2020-era proposals)

### Pipeline Steps

```
1. Read PDF file path
2. Send to Docling-Serve (10.0.21.42:5001) for text extraction
3. Parse extracted text:
   a. Identify scope sections (regex: numbered headings, "Scope of Services", etc.)
   b. Extract metadata (project name, client, date — from header/footer)
   c. Split into sections with headings
4. Generate embedding via Ollama (for vector search)
5. Store in proposal_corpus table
6. Optionally: auto-extract clauses from sections → seed clause table (with review flag)
```

### Corpus Uses

- **RAG retrieval** during scope generation — find similar past proposals
- **Clause seeding** — extract common patterns, deduplicate, seed library
- **Style reference** — LLM uses corpus examples to match Emittiv's writing voice
- **Analytics** — track which clause patterns appear most, how scope varies by region/type

## Authentication

Same API key pattern as e-fees-api: `X-API-Key` header, validated against env var.

## Environment Variables

```env
SURREAL_URL=ws://10.0.23.11:8000
SURREAL_USER=root
SURREAL_PASS=...
SURREAL_NS=emittiv
SURREAL_DB=projects

OLLAMA_URL=http://10.0.21.20:11434
OLLAMA_MODEL=qwen3:4b

DOCLING_URL=http://10.0.21.42:5001

API_KEY=efees-scope-2026-...

# Optional: corpus source path (for batch ingest)
CORPUS_PATH=/data/rfps
```

## Deployment

```dockerfile
FROM rust:1.89-slim AS builder
# ... (same pattern as e-fees-api Dockerfile)

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/e-fees-scope /usr/local/bin/
EXPOSE 3201
CMD ["e-fees-scope"]
```

Docker run on AI server (br0 network):
```bash
docker run -d \
  --name e-fees-scope \
  --network br0 --ip 10.0.21.81 \
  -p 3201:3201 \
  --env-file /mnt/user/appdata/e-fees-scope/.env \
  -v /mnt/user/emittiv/nc/__groupfolders/1/01 Projects/01 RFPs/99 All RFPs:/data/rfps:ro \
  --restart unless-stopped \
  e-fees-scope:latest
```

## Shared Crate

Uses `e-fees-core` from the workspace for shared types (Project, Fee, etc.). The scope service's Cargo.toml references it as a path dependency during build, same pattern as e-fees-api.

## Future Extensions

- **InDesign integration:** scope export endpoint returns structured JSON matching InDesign template fields
- **Clause versioning:** full history of clause edits (version field + audit trail)
- **Fine-tuning:** use corpus as training data if/when local fine-tuning becomes viable
- **Approval workflow:** scope review/approval states before export
- **Multi-language:** clause variants per language (English/Arabic)
