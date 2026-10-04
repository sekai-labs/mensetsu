use utoipa::OpenApi;
use std::fs;

fn main() {
    let openapi = mensetsu_api::interfaces::http::ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI spec");
    fs::create_dir_all("contracts").expect("Failed to create contracts dir");
    fs::write("contracts/openapi.json", json).expect("Failed to write openapi.json");
    println!("Exported OpenAPI spec to contracts/openapi.json");
}
