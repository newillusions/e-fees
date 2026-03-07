-- E-Fees Scope Service Schema
-- Run against SurrealDB: ns=emittiv, db=projects

-- ============================================================
-- Clause library
-- ============================================================
DEFINE TABLE clause SCHEMAFULL;

DEFINE FIELD category    ON clause TYPE string;
DEFINE FIELD subcategory ON clause TYPE option<string>;
DEFINE FIELD title       ON clause TYPE string;
DEFINE FIELD body        ON clause TYPE string;
DEFINE FIELD conditions  ON clause TYPE option<object>;
DEFINE FIELD sort_order  ON clause TYPE int;
DEFINE FIELD tags        ON clause TYPE option<array<string>>;
DEFINE FIELD is_default  ON clause TYPE bool DEFAULT true;
DEFINE FIELD status      ON clause TYPE string DEFAULT "active";
DEFINE FIELD version     ON clause TYPE int DEFAULT 1;
DEFINE FIELD created_at  ON clause TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at  ON clause TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_clause_category ON clause FIELDS category;
DEFINE INDEX idx_clause_status   ON clause FIELDS status;

-- ============================================================
-- Scope assembly (generated scope per fee)
-- ============================================================
DEFINE TABLE scope_assembly SCHEMAFULL;

DEFINE FIELD fee_id         ON scope_assembly TYPE record<fee>;
DEFINE FIELD clauses        ON scope_assembly TYPE array;
DEFINE FIELD generated_text ON scope_assembly TYPE string;
DEFINE FIELD numbering      ON scope_assembly TYPE option<object>;
DEFINE FIELD llm_model      ON scope_assembly TYPE option<string>;
DEFINE FIELD llm_polished   ON scope_assembly TYPE bool DEFAULT false;
DEFINE FIELD created_at     ON scope_assembly TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at     ON scope_assembly TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_scope_fee ON scope_assembly FIELDS fee_id UNIQUE;

-- ============================================================
-- Proposal corpus (historical PDFs for RAG)
-- ============================================================
DEFINE TABLE proposal_corpus SCHEMAFULL;

DEFINE FIELD filename       ON proposal_corpus TYPE string;
DEFINE FIELD project_number ON proposal_corpus TYPE option<string>;
DEFINE FIELD project_name   ON proposal_corpus TYPE option<string>;
DEFINE FIELD extracted_text ON proposal_corpus TYPE string;
DEFINE FIELD sections       ON proposal_corpus TYPE option<array>;
DEFINE FIELD metadata                    ON proposal_corpus TYPE option<object>;
DEFINE FIELD metadata.extraction_method  ON proposal_corpus TYPE option<string>;
DEFINE FIELD embedding      ON proposal_corpus TYPE option<array<float>>;
DEFINE FIELD created_at     ON proposal_corpus TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_corpus_project ON proposal_corpus FIELDS project_number;
