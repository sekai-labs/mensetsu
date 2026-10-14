pub fn clean_record_id(raw: &str) -> String {
    let s = raw.trim().trim_matches('`');
    let s = s.split_once(':').map_or(s, |(_, id)| id);
    s.trim_matches(|c| c == '`' || c == '⟨' || c == '⟩').to_string()
}

pub fn normalize_record_fields(value: &mut serde_json::Value, fields: &[&str]) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                normalize_record_fields(item, fields);
            }
        }
        serde_json::Value::Object(map) => {
            for field in fields {
                if let Some(serde_json::Value::String(s)) = map.get_mut(*field) {
                    *s = clean_record_id(s);
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_table_prefix_and_backticks() {
        assert_eq!(clean_record_id("workspace:`c893b383-dd67`"), "c893b383-dd67");
        assert_eq!(clean_record_id("`c893b383-dd67`"), "c893b383-dd67");
        assert_eq!(clean_record_id("learning_unit:⟨lu_1⟩"), "lu_1");
        assert_eq!(clean_record_id("run_abc"), "run_abc");
    }

    #[test]
    fn normalizes_named_fields_in_arrays_and_objects() {
        let mut v = serde_json::json!([
            {"id": "source:`s1`", "workspace_id": "workspace:`w1`", "url": "https://x.test/a:b"},
            {"id": "s2"}
        ]);
        normalize_record_fields(&mut v, &["id", "workspace_id"]);
        assert_eq!(v[0]["id"], "s1");
        assert_eq!(v[0]["workspace_id"], "w1");
        assert_eq!(v[0]["url"], "https://x.test/a:b");
        assert_eq!(v[1]["id"], "s2");
    }
}
