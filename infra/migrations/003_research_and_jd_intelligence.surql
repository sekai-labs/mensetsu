USE NS mensetsu DB production;

DEFINE FIELD candidate_name ON TABLE resume TYPE option<string>;
DEFINE FIELD experience_highlights ON TABLE resume TYPE option<array<string>>;
DEFINE FIELD extracted_skills ON TABLE resume TYPE option<array<object>>;

DEFINE FIELD title ON TABLE job_description TYPE option<string>;
DEFINE FIELD role_summary ON TABLE job_description TYPE option<string>;
DEFINE FIELD canonical_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD jd_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD source_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD department ON TABLE job_description TYPE option<string>;
DEFINE FIELD employment_type ON TABLE job_description TYPE option<string>;
DEFINE FIELD seniority ON TABLE job_description TYPE option<string>;
DEFINE FIELD locations ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD remote_policy ON TABLE job_description TYPE option<string>;
DEFINE FIELD salary ON TABLE job_description TYPE option<object>;
DEFINE FIELD responsibilities ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD required_qualifications ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD preferred_qualifications ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD technical_skills ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD soft_skills ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD experience_requirements ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD education_requirements ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD benefits ON TABLE job_description TYPE option<array<string>>;
DEFINE FIELD application_url ON TABLE job_description TYPE option<string>;
DEFINE FIELD posted_at ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD valid_through ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD retrieved_at ON TABLE job_description TYPE option<datetime>;
DEFINE FIELD extraction_method ON TABLE job_description TYPE option<string>;
DEFINE FIELD extraction_confidence ON TABLE job_description TYPE option<number>;
DEFINE FIELD source_evidence ON TABLE job_description TYPE option<array<object>>;
DEFINE FIELD raw_content_reference ON TABLE job_description TYPE option<string>;
DEFINE FIELD schema_version ON TABLE job_description TYPE option<string>;
DEFINE FIELD updated_at ON TABLE job_description TYPE datetime DEFAULT time::now();

DEFINE FIELD content_hash ON TABLE source TYPE option<string>;
DEFINE FIELD retrieval_method ON TABLE source TYPE option<string>;
DEFINE FIELD raw_snapshot_reference ON TABLE source TYPE option<string>;
DEFINE FIELD publisher ON TABLE source TYPE option<string>;
DEFINE FIELD details.* ON TABLE pipeline_log TYPE any;
DEFINE FIELD details.* ON TABLE pipeline_execution TYPE any;
DEFINE FIELD published_date ON TABLE source TYPE option<string>;
DEFINE FIELD access_status ON TABLE source TYPE option<string>;
DEFINE FIELD freshness_metadata ON TABLE source TYPE option<object>;

DEFINE TABLE source_chunk SCHEMAFULL;
DEFINE FIELD workspace_id ON TABLE source_chunk TYPE record<workspace>;
DEFINE FIELD source_id ON TABLE source_chunk TYPE record<source>;
DEFINE FIELD chunk_index ON TABLE source_chunk TYPE int;
DEFINE FIELD content ON TABLE source_chunk TYPE string;
DEFINE FIELD start_char ON TABLE source_chunk TYPE option<int>;
DEFINE FIELD end_char ON TABLE source_chunk TYPE option<int>;
DEFINE FIELD created_at ON TABLE source_chunk TYPE datetime DEFAULT time::now();
DEFINE INDEX idx_chunk_source ON TABLE source_chunk COLUMNS source_id;
DEFINE INDEX idx_chunk_workspace ON TABLE source_chunk COLUMNS workspace_id;

DEFINE FIELD evidence_snippet ON TABLE claim TYPE option<string>;
DEFINE FIELD source_url ON TABLE claim TYPE option<string>;
DEFINE FIELD classification ON TABLE claim TYPE option<string>;
DEFINE FIELD contradiction_note ON TABLE claim TYPE option<string>;
DEFINE FIELD conflicting_claim_id ON TABLE claim TYPE option<record<claim>>;

REMOVE INDEX idx_skill_slug ON TABLE skill;
DEFINE INDEX idx_skill_ws_slug ON TABLE skill COLUMNS workspace_id, slug;
