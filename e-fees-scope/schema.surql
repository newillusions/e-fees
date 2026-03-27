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
DEFINE FIELD conditions  ON clause TYPE option<object> FLEXIBLE;
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
DEFINE FIELD clauses          ON scope_assembly TYPE array<object> FLEXIBLE;
DEFINE FIELD clauses.*        ON scope_assembly TYPE object FLEXIBLE;
DEFINE FIELD clauses.*.clauses   ON scope_assembly TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD clauses.*.clauses.* ON scope_assembly TYPE object FLEXIBLE;
DEFINE FIELD generated_text ON scope_assembly TYPE string;
DEFINE FIELD numbering      ON scope_assembly TYPE option<object> FLEXIBLE;
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

-- ============================================================
-- Stage configuration (canonical stages with aliases)
-- ============================================================
DEFINE TABLE stage_config SCHEMAFULL;

DEFINE FIELD canonical_name ON stage_config TYPE string;
DEFINE FIELD default_label  ON stage_config TYPE string;
DEFINE FIELD aliases        ON stage_config TYPE option<array<string>>;
DEFINE FIELD sort_order     ON stage_config TYPE int;
DEFINE FIELD intro_text     ON stage_config TYPE option<string>;
DEFINE FIELD status         ON stage_config TYPE string DEFAULT "active";

DEFINE INDEX idx_stage_canonical ON stage_config FIELDS canonical_name UNIQUE;

-- ============================================================
-- Deliverable library (atomic deliverable items)
-- ============================================================
DEFINE TABLE deliverable SCHEMAFULL;

DEFINE FIELD title            ON deliverable TYPE string;
DEFINE FIELD short_name       ON deliverable TYPE string;
DEFINE FIELD body             ON deliverable TYPE string;
DEFINE FIELD stage            ON deliverable TYPE string;
DEFINE FIELD layer            ON deliverable TYPE string
    ASSERT $value IN ['generic', 'discipline', 'conditional'];
DEFINE FIELD discipline       ON deliverable TYPE option<string>;
DEFINE FIELD condition        ON deliverable TYPE option<object> FLEXIBLE;
DEFINE FIELD replaces         ON deliverable TYPE option<record<deliverable>>;
DEFINE FIELD sort_order       ON deliverable TYPE int;
DEFINE FIELD source_proposals ON deliverable TYPE option<array<string>>;
DEFINE FIELD usage_history    ON deliverable TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD usage_history.*  ON deliverable TYPE object FLEXIBLE;
DEFINE FIELD tags             ON deliverable TYPE option<array<string>>;
DEFINE FIELD status           ON deliverable TYPE string DEFAULT "active";
DEFINE FIELD version          ON deliverable TYPE int DEFAULT 1;
DEFINE FIELD created_at       ON deliverable TYPE datetime DEFAULT time::now();
DEFINE FIELD updated_at       ON deliverable TYPE datetime DEFAULT time::now();

DEFINE INDEX idx_deliverable_stage  ON deliverable FIELDS stage;
DEFINE INDEX idx_deliverable_layer  ON deliverable FIELDS layer;
DEFINE INDEX idx_deliverable_status ON deliverable FIELDS status;

-- ============================================================
-- Extend scope_assembly for deliverable tracking
-- ============================================================
DEFINE FIELD deliverables_used   ON scope_assembly TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD deliverables_used.* ON scope_assembly TYPE object FLEXIBLE;  -- FLEXIBLE required for nested fields
DEFINE FIELD stage_labels        ON scope_assembly TYPE option<object> FLEXIBLE;
DEFINE FIELD manual_items        ON scope_assembly TYPE option<array<object>> FLEXIBLE;
DEFINE FIELD manual_items.*      ON scope_assembly TYPE object FLEXIBLE;  -- FLEXIBLE required for nested fields

-- ============================================================
-- Extend stage_config for dictionary/autocomplete
-- ============================================================
DEFINE FIELD code             ON stage_config TYPE option<string>;
DEFINE FIELD usage_count      ON stage_config TYPE int DEFAULT 0;
DEFINE FIELD is_post_contract ON stage_config TYPE bool DEFAULT false;

-- ============================================================
-- Scope revision history (separate table to avoid record bloat)
-- ============================================================
DEFINE TABLE scope_revision SCHEMAFULL;

DEFINE FIELD fee_id         ON scope_revision TYPE record<fee>;
DEFINE FIELD revision       ON scope_revision TYPE int;
DEFINE FIELD clauses        ON scope_revision TYPE string;  -- JSON string snapshot (audit, not queried by field)
DEFINE FIELD generated_text ON scope_revision TYPE string;
DEFINE FIELD stages_at_time ON scope_revision TYPE option<array<string>>;
DEFINE FIELD saved_at       ON scope_revision TYPE datetime DEFAULT time::now();
DEFINE FIELD trigger        ON scope_revision TYPE string;

DEFINE INDEX idx_scope_rev_fee ON scope_revision FIELDS fee_id, revision;

-- ============================================================
-- Extend scope_assembly for revision tracking and stage snapshot
-- ============================================================
DEFINE FIELD current_revision ON scope_assembly TYPE int DEFAULT 0;
DEFINE FIELD stages_snapshot  ON scope_assembly TYPE option<array<string>>;
