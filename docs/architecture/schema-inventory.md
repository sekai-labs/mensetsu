# Schema inventory before cleanup

**Snapshot:** 2026-11-12, local development database `mensetsu/production`, SurrealDB v3.2.4.
**Method:** aggregate `count()` queries only; no field values were read. Record counts and populated-field counts are point-in-time evidence, not a production census. Runtime evidence comes from a repository scan of `server/api/src`, `server/intelligence/src`, `app/web`, `app/extension`, and generated contracts. External consumers outside this repository were not available for review.

This inventory is read-only. It does not remove or rename any table, field, relation, or index, and it does not propose a destructive cleanup in this release. The `application_dispatch` outbox table and the saved-job keyset index are introduced by migration `016` and therefore have no rows in this pre-migration snapshot.

## Table counts

| Table | Rows | Runtime evidence | Disposition |
| --- | ---: | --- | --- |
| `source_chunk` | 0 | No table read/write in Rust API or Python sources. | Preserve; candidate for a later reviewed cleanup. |
| `concept` | 0 | No table read/write; `concept` appears only as prose in Python schema descriptions. | Preserve; candidate for a later reviewed cleanup. |
| `company_entity` | 0 | No table read/write. | Preserve; candidate for a later reviewed cleanup. |
| `rel_requires` | 0 | No runtime writer. The relation targets `role_entity`, which has no table definition. | Preserve; reconcile with product needs before any change. |
| `rel_has_skill` | 0 | No table read/write. | Preserve; candidate for a later reviewed cleanup. |
| `rel_depends_on` | 0 | No table read/write. | Preserve; candidate for a later reviewed cleanup. |
| `rel_supports` | 0 | No table read/write. | Preserve; candidate for a later reviewed cleanup. |
| `rel_describes` | 0 | No table read/write. | Preserve; candidate for a later reviewed cleanup. |
| `source` | 0 | Python reader/orchestration paths produce source metadata; the table is empty. | Preserve; metadata fields remain part of provenance design. |
| `claim` | 0 | Both intelligence pipelines produce evidence snippets; the API research handlers read claim data. | Preserve; active schema despite current row count. |
| `job_description` | 12 | Active: workspace role views and Python extraction outputs read/write it. | Preserve. |
| `job_application` | 19 | Active: direct and saved-job application preparation read/write it. | Preserve. |
| `saved_job` | 1 | Active: extension saved-job list, prepare, approve, and dismiss handlers. | Preserve; pagination index added by `016`. |
| `workspace` | 13 | Active workspace lifecycle and run dispatch. | Preserve. |
| `user_llm_config` | 2 | Active: settings and workspace LLM resolution read/write it. | Preserve. |
| `system_llm_provider` | 1 | Active provider routing. | Preserve. |
| `user_llm_provider` | 1 | Active provider override. | Preserve. |
| `learning_unit` | 0 | Learning path schema; no populated rows in this snapshot. | Preserve. |
| `skill` | 0 | Workspace skill projection; no populated rows in this snapshot. | Preserve. |
| `skill_relation` | 0 | Workspace skill relationships; no populated rows in this snapshot. | Preserve. |
| `interview_question` | 0 | Practice schema; no populated rows in this snapshot. | Preserve. |
| `match_analysis` | 0 | Match-analysis schema; no populated rows in this snapshot. | Preserve. |

## Populated optional fields

### `job_description` (12 rows)

| Field | Populated | Runtime evidence |
| --- | ---: | --- |
| `title` | 6 | Active role display. |
| `canonical_url` | 6 | Active job identity. |
| `jd_url` | 6 | Active job identity. |
| `source_url` | 6 | Active provenance. |
| `retrieved_at` | 6 | Active provenance. |
| `extraction_method` | 6 | Active provenance. |
| `updated_at` | 12 | Active timestamp. |
| `role_summary` | 0 | Produced by Python static/hybrid outputs and read by API role handlers. |
| `department` | 0 | No runtime read/write found. |
| `employment_type` | 0 | No runtime read/write found. |
| `seniority` | 0 | No runtime read/write found. |
| `locations` | 0 | No runtime read/write found. |
| `remote_policy` | 0 | No runtime read/write found. |
| `salary` | 0 | No runtime read/write found. |
| `responsibilities` | 0 | Read by API role handlers. |
| `required_qualifications` | 0 | No runtime read/write found. |
| `preferred_qualifications` | 0 | No runtime read/write found. |
| `technical_skills` | 0 | Produced by both pipeline outputs. |
| `soft_skills` | 0 | No runtime read/write found. |
| `experience_requirements` | 0 | No runtime read/write found. |
| `education_requirements` | 0 | No runtime read/write found. |
| `benefits` | 0 | No runtime read/write found. |
| `application_url` | 0 | No runtime read/write found. |
| `posted_at` | 0 | No runtime read/write found. |
| `valid_through` | 0 | No runtime read/write found. |
| `extraction_confidence` | 0 | No runtime read/write found. |
| `source_evidence` | 0 | No runtime read/write found. |
| `raw_content_reference` | 0 | No runtime read/write found. |
| `schema_version` | 0 | No runtime read/write found for this field. |

### `source` optional metadata (0 rows)

`content_hash`, `retrieval_method`, `raw_snapshot_reference`, `publisher`, `published_date`, `access_status`, and `freshness_metadata` are all populated in 0 rows. Python fetch/read paths construct `content_hash` and `retrieval_method`; the remaining fields have no runtime read/write found in the scanned sources.

### `claim` (0 rows)

`evidence_snippet`, `source_url`, `classification`, `contradiction_note`, and `conflicting_claim_id` are populated in 0 rows. `evidence_snippet` is produced by both intelligence pipelines and read by API research handlers; `conflicting_claim_id` and `classification` have no populated data and no runtime writer found in the scanned sources.

### `job_application` (19 rows)

`source_title` is populated in 19 rows and is written by direct and saved-job preparation, but no read hit was found in the scanned sources, making it write-only. `retrieved_at` is populated in 19 rows and is written as provenance during capture and import. `application_method` is populated in 10 rows, `contact_email` in 1 row, `workspace_id` in 6 rows, and `confirmation_url` in 6 rows.

### `workspace` (13 rows)

`company_urls` is populated in 9 rows and is active in workspace creation and research input. `user_prompt` and `current_objective` are populated in 0 rows; the dispatcher intentionally does not read mutable workspace objective.

### `saved_job` (1 row)

`job_application_id` is populated in 0 rows, so the only saved job is currently unlinked. Optional job facts (`application_url`, `job_description`, `role_title`, `company_name`, `location`) are populated in 0 rows; `contact_email` is populated in 1 row.

## External-consumer review

The OpenAPI and generated clients expose application response DTOs, not raw SurrealDB tables. `job_application.source_title` is not part of `JobApplicationResponseDto`. The dormant graph tables, `source_chunk`, `concept`, `company_entity`, and most optional `job_description` fields are not referenced by the Rust API, Python intelligence sources, web UI, or extension sources in this repository. That is repository evidence only; no external database or analytics consumer was available to confirm whether these fields are read outside the codebase.

## Deferred proposals

`source_chunk`, `concept`, `company_entity`, the unused graph relations, the zero-populated optional research fields, and write-only `job_application.source_title` remain in place. Dropping them, adding speculative vector indexes, or adding a new graph writer is deferred until the candidate schema is reconciled with actual product needs and an external-consumer review is completed.
