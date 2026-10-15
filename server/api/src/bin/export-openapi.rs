use utoipa::OpenApi;
use std::fs;

fn sort_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries: Vec<(&String, &serde_json::Value)> = map.iter().collect();
            entries.sort_by_key(|(k, _)| *k);
            let sorted_map: serde_json::Map<String, serde_json::Value> = entries
                .into_iter()
                .map(|(k, v)| (k.clone(), sort_json(v)))
                .collect();
            serde_json::Value::Object(sorted_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sort_json).collect())
        }
        _ => value.clone(),
    }
}

fn main() {
    let openapi = mensetsu_api::interfaces::http::ApiDoc::openapi();
    let val = serde_json::to_value(&openapi).expect("Failed to convert OpenAPI spec to JSON Value");
    let sorted_val = sort_json(&val);
    let mut json = serde_json::to_string_pretty(&sorted_val).expect("Failed to serialize OpenAPI spec");
    json.push('\n');

    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let packages_contracts_dir = repo_root.join("packages").join("contracts");
    fs::create_dir_all(&packages_contracts_dir).expect("Failed to create packages/contracts dir");
    let packages_out_path = packages_contracts_dir.join("openapi.json");
    fs::write(&packages_out_path, &json).expect("Failed to write packages/contracts/openapi.json");
    println!("Exported deterministic OpenAPI spec to {}", packages_out_path.display());

    let api_contracts_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts");
    fs::create_dir_all(&api_contracts_dir).expect("Failed to create server/api/contracts dir");
    let api_out_path = api_contracts_dir.join("openapi.json");
    fs::write(&api_out_path, &json).expect("Failed to write server/api/contracts/openapi.json");
    println!("Exported deterministic OpenAPI spec to {}", api_out_path.display());
}
