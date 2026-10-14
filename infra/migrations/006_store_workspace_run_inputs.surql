USE NS mensetsu DB production;

DEFINE FIELD OVERWRITE company_urls ON TABLE workspace TYPE option<array<string>>;
DEFINE FIELD OVERWRITE source_pdf_base64 ON TABLE resume TYPE option<string>;
DEFINE FIELD OVERWRITE source_filename ON TABLE resume TYPE option<string>;
