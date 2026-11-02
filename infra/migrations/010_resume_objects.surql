USE NS mensetsu DB production;

DEFINE TABLE OVERWRITE resume_upload SCHEMAFULL;
DEFINE FIELD OVERWRITE user_id ON TABLE resume_upload TYPE record<user>;
DEFINE FIELD OVERWRITE workspace_id ON TABLE resume_upload TYPE option<record<workspace>>;
DEFINE FIELD OVERWRITE object_key ON TABLE resume_upload TYPE string;
DEFINE FIELD OVERWRITE staging_key ON TABLE resume_upload TYPE option<string>;
DEFINE FIELD OVERWRITE filename ON TABLE resume_upload TYPE string;
DEFINE FIELD OVERWRITE size_bytes ON TABLE resume_upload TYPE int ASSERT $value > 0 AND $value <= 5242880;
DEFINE FIELD OVERWRITE content_type ON TABLE resume_upload TYPE string ASSERT $value = 'application/pdf';
DEFINE FIELD OVERWRITE status ON TABLE resume_upload TYPE string ASSERT $value IN ['pending', 'completed', 'attached'];
DEFINE FIELD OVERWRITE created_at ON TABLE resume_upload TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE resume_upload TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_resume_upload_user ON TABLE resume_upload COLUMNS user_id;
DEFINE INDEX IF NOT EXISTS idx_resume_upload_key ON TABLE resume_upload COLUMNS object_key UNIQUE;

DEFINE FIELD OVERWRITE source_filename ON TABLE resume TYPE option<string>;
DEFINE FIELD OVERWRITE source_upload_id ON TABLE resume TYPE option<record<resume_upload>>;
DEFINE FIELD OVERWRITE source_object_key ON TABLE resume TYPE option<string>;
