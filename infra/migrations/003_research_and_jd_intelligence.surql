USE NS mensetsu DB production;

DEFINE FIELD OVERWRITE candidate_name ON TABLE resume TYPE option<string>;
DEFINE FIELD OVERWRITE experience_highlights ON TABLE resume TYPE option<array<string>>;
DEFINE FIELD OVERWRITE extracted_skills ON TABLE resume TYPE option<array<object>>;

DEFINE FIELD OVERWRITE title ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE role_summary ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE canonical_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE jd_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE source_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE department ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE employment_type ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE seniority ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE locations ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE remote_policy ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE salary ON TABLE job_description TYPE option<object>;
DEFINE FIELD OVERWRITE responsibilities ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE required_qualifications ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE preferred_qualifications ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE technical_skills ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE soft_skills ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE experience_requirements ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE education_requirements ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE benefits ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD OVERWRITE application_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE posted_at ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD OVERWRITE valid_through ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD OVERWRITE retrieved_at ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD OVERWRITE extraction_method ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE extraction_confidence ON TABLE job_description TYPE option<number>;
DEFINE FIELD OVERWRITE source_evidence ON TABLE job_description TYPE option<array<object>>;
DEFINE FIELD OVERWRITE raw_content_reference ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE schema_version ON TABLE job_description TYPE option<string>;
DEFINE FIELD OVERWRITE updated_at ON TABLE job_description TYPE datetime DEFAULT time::now();

DEFINE FIELD OVERWRITE content_hash ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE retrieval_method ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE raw_snapshot_reference ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE publisher ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE published_date ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE access_status ON TABLE source TYPE option<string>;
DEFINE FIELD OVERWRITE freshness_metadata ON TABLE source TYPE option<object>;

DEFINE TABLE IF NOT EXISTS source_chunk SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE source_chunk TYPE record<workspace>;
DEFINE FIELD OVERWRITE source_id ON TABLE source_chunk TYPE record<source>;
DEFINE FIELD OVERWRITE chunk_index ON TABLE source_chunk TYPE int;
DEFINE FIELD OVERWRITE content ON TABLE source_chunk TYPE string;
DEFINE FIELD OVERWRITE start_char ON TABLE source_chunk TYPE option<int>;
DEFINE FIELD OVERWRITE end_char ON TABLE source_chunk TYPE option<int>;
DEFINE FIELD OVERWRITE created_at ON TABLE source_chunk TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_chunk_source ON TABLE source_chunk COLUMNS source_id;
DEFINE INDEX IF NOT EXISTS idx_chunk_workspace ON TABLE source_chunk COLUMNS workspace_id;

DEFINE FIELD OVERWRITE evidence_snippet ON TABLE claim TYPE option<string>;
DEFINE FIELD OVERWRITE source_url ON TABLE claim TYPE option<string>;
DEFINE FIELD OVERWRITE classification ON TABLE claim TYPE option<string>;
DEFINE FIELD OVERWRITE contradiction_note ON TABLE claim TYPE option<string>;
DEFINE FIELD OVERWRITE conflicting_claim_id ON TABLE claim TYPE option<record<claim>>;

REMOVE INDEX IF EXISTS idx_skill_slug ON TABLE skill;
