use crate::contexts::workspace::infrastructure::db::Db;
use crate::shared::error::{AppError, Result};

pub const SCHEMA: &str = include_str!("../../../schema.surql");

pub fn split_statements(sql: &str) -> Vec<&str> {
    let bytes = sql.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    let mut quote: Option<u8> = None;
    let mut has_code = false;

    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            if b == b'\\' {
                i += 2;
                continue;
            }
            if b == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match b {
            b'\'' | b'"' | b'`' => {
                quote = Some(b);
                has_code = true;
            }
            b'-' if bytes.get(i + 1) == Some(&b'-') => {
                i = skip_line(bytes, i);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'/') => {
                i = skip_line(bytes, i);
                continue;
            }
            b'#' => {
                i = skip_line(bytes, i);
                continue;
            }
            b'/' if bytes.get(i + 1) == Some(&b'*') => {
                i = sql[i + 2..].find("*/").map_or(bytes.len(), |p| i + 2 + p + 2);
                continue;
            }
            b';' => {
                if has_code {
                    out.push(strip_comments(&sql[start..i]));
                }
                start = i + 1;
                has_code = false;
            }
            b if !b.is_ascii_whitespace() => has_code = true,
            _ => {}
        }
        i += 1;
    }
    if has_code {
        out.push(strip_comments(&sql[start..]));
    }
    out
}

fn skip_line(bytes: &[u8], i: usize) -> usize {
    bytes[i..].iter().position(|&c| c == b'\n').map_or(bytes.len(), |p| i + p + 1)
}

fn strip_comments(stmt: &str) -> &str {
    let mut s = stmt.trim_start();
    loop {
        if s.starts_with("--") || s.starts_with("//") || s.starts_with('#') {
            s = s.find('\n').map_or("", |p| &s[p + 1..]).trim_start();
        } else if let Some(rest) = s.strip_prefix("/*") {
            s = rest.find("*/").map_or("", |p| &rest[p + 2..]).trim_start();
        } else {
            return s.trim_end();
        }
    }
}

pub fn check_target(statements: &[&str], ns: &str, db: &str) -> Result<()> {
    let uses: Vec<Vec<&str>> =
        statements.iter().filter(|s| s.starts_with("USE ")).map(|s| s.split_whitespace().collect()).collect();
    let targets: Vec<(&str, &str)> = uses
        .iter()
        .filter_map(|w| match w.as_slice() {
            ["USE", "NS", n, "DB", d] => Some((*n, *d)),
            _ => None,
        })
        .collect();
    let [(n, d)] = targets.as_slice() else {
        return Err(AppError::Database(format!(
            "schema.surql must contain exactly one `USE NS .. DB ..`, found {}",
            targets.len()
        )));
    };
    if let Some(bad) = uses.iter().find(|w| !matches!(w.as_slice(), ["USE", "NS", x] | ["USE", "NS", x, "DB", _] if *x == *n)) {
        return Err(AppError::Database(format!("schema.surql switches to another target: {}", bad.join(" "))));
    }
    if *n == ns && *d == db {
        Ok(())
    } else {
        Err(AppError::Database(format!(
            "schema.surql targets {n}/{d} but SURREAL_NS/SURREAL_DB is {ns}/{db}; refusing to apply the schema to a database the app does not use"
        )))
    }
}

const MAX_ATTEMPTS: u32 = 20;

pub async fn apply(client: &Db, sql: &str, ns: &str, db: &str) -> Result<()> {
    let statements = split_statements(sql);
    check_target(&statements, ns, db)?;
    let wrapped = format!("BEGIN TRANSACTION;\n{sql}\n;COMMIT TRANSACTION;");
    let expected = statements.len() + 2;

    for attempt in 1..=MAX_ATTEMPTS {
        let mut resp = client
            .query(&wrapped)
            .await
            .map_err(|e| AppError::Database(format!("schema.surql could not be submitted: {e}")))?;

        if resp.num_statements() != expected {
            return Err(AppError::Database(format!(
                "schema.surql: server parsed {} statements, expected {expected}",
                resp.num_statements()
            )));
        }

        let errors = resp.take_errors();
        if errors.is_empty() {
            return Ok(());
        }
        if only_conflicts(&errors) && attempt < MAX_ATTEMPTS {
            tracing::warn!(attempt, "schema.surql transaction lost a commit race with a concurrent applier; retrying");
            tokio::time::sleep(std::time::Duration::from_millis(20 * u64::from(attempt))).await;
            continue;
        }
        return Err(describe(&statements, errors, attempt));
    }
    unreachable!("loop returns on success or on the final attempt")
}

fn is_cascade(e: &surrealdb::Error) -> bool {
    use surrealdb::types::QueryError;
    matches!(e.query_details(), Some(QueryError::NotExecuted | QueryError::Cancelled))
}

fn only_conflicts(errors: &std::collections::HashMap<usize, surrealdb::Error>) -> bool {
    let mut roots = errors.values().filter(|e| !is_cascade(e)).peekable();
    roots.peek().is_some()
        && roots.all(|e| matches!(e.query_details(), Some(surrealdb::types::QueryError::TransactionConflict)))
}

fn describe(statements: &[&str], errors: std::collections::HashMap<usize, surrealdb::Error>, attempts: u32) -> AppError {
    let mut roots: Vec<(usize, String)> =
        errors.iter().filter(|(_, e)| !is_cascade(e)).map(|(i, e)| (*i, e.to_string())).collect();
    roots.sort_by_key(|(i, _)| *i);
    let detail = roots
        .iter()
        .map(|(i, msg)| match i.checked_sub(1).and_then(|f| statements.get(f).map(|s| (f, s))) {
            Some((f, stmt)) => format!("  statement #{f}: {msg}\n    {stmt}"),
            None => format!("  transaction control: {msg}"),
        })
        .collect::<Vec<_>>()
        .join("\n");
    AppError::Database(format!(
        "schema.surql failed and was rolled back ({} errored statement(s), attempt {attempts}); no schema changes were applied\n{detail}",
        roots.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_top_level_semicolons_only() {
        let sql = "-- header; not a statement\nDEFINE TABLE a;\n/* block; comment */\nDEFINE FIELD x ON a TYPE string ASSERT $value != ';';\n# hash; comment\nRETURN \"a;b\"; RETURN `we;ird`\n";
        assert_eq!(
            split_statements(sql),
            vec![
                "DEFINE TABLE a",
                "DEFINE FIELD x ON a TYPE string ASSERT $value != ';'",
                "RETURN \"a;b\"",
                "RETURN `we;ird`",
            ]
        );
    }

    #[test]
    fn handles_escaped_quotes_and_empty_statements() {
        assert_eq!(
            split_statements(r"RETURN 'it\'s; fine';;  ; RETURN 2;"),
            vec![r"RETURN 'it\'s; fine'", "RETURN 2"]
        );
    }

    #[test]
    fn real_schema_is_entirely_idempotent() {
        let statements = split_statements(SCHEMA);
        check_target(&statements, "mensetsu", "production").unwrap();
        assert!(statements.len() > 300, "suspiciously few statements: {}", statements.len());
        let offenders: Vec<&&str> = statements
            .iter()
            .filter(|s| s.starts_with("DEFINE "))
            .filter(|s| !s.contains(" OVERWRITE ") && !s.contains(" IF NOT EXISTS "))
            .collect();
        assert!(offenders.is_empty(), "non-idempotent DEFINEs make re-apply fail: {offenders:#?}");
        let others: Vec<&&str> =
            statements.iter().filter(|s| !s.starts_with("DEFINE ") && !s.starts_with("USE ")).collect();
        assert!(others.is_empty(), "schema.surql must contain only DEFINE/USE statements: {others:#?}");
    }

    #[test]
    fn target_mismatch_fails_closed() {
        let stmts = split_statements(SCHEMA);
        let err = check_target(&stmts, "staging", "production").unwrap_err().to_string();
        assert!(err.contains("mensetsu/production") && err.contains("staging/production"), "{err}");
        assert!(check_target(&["DEFINE TABLE a"], "mensetsu", "production").is_err());
        assert!(check_target(&["USE NS a DB b", "USE NS a DB b"], "a", "b").is_err());
        assert!(check_target(&["USE NS a", "USE NS a DB b"], "a", "b").is_ok());
        assert!(check_target(&["USE NS other", "USE NS a DB b"], "a", "b").is_err(), "must not switch ns mid-file");
    }

    use surrealdb::types::QueryError;
    use std::collections::HashMap;

    fn q(kind: QueryError) -> surrealdb::Error {
        surrealdb::Error::query("x".into(), Some(kind))
    }

    #[test]
    fn retries_a_pure_commit_race_only() {
        let race: HashMap<usize, surrealdb::Error> =
            [(1, q(QueryError::NotExecuted)), (2, q(QueryError::NotExecuted)), (3, q(QueryError::TransactionConflict))].into();
        assert!(only_conflicts(&race));

        let real = [(1, q(QueryError::NotExecuted)), (2, surrealdb::Error::thrown("boom".into())), (3, q(QueryError::Cancelled))].into();
        assert!(!only_conflicts(&real), "a real error must fail closed");

        let mixed = [(2, surrealdb::Error::thrown("boom".into())), (3, q(QueryError::TransactionConflict))].into();
        assert!(!only_conflicts(&mixed), "a conflict must not mask a real error");

        let cascade_only = [(1, q(QueryError::NotExecuted)), (2, q(QueryError::Cancelled))].into();
        assert!(!only_conflicts(&cascade_only), "no root cause is not a retryable race");
    }

    #[test]
    fn error_names_only_root_causes_by_file_index() {
        let stmts = ["USE NS a DB b", "DEFINE TABLE OVERWRITE t SCHEMAFULL", "DEFINE INDEX IF NOT EXISTS i ON TABLE t COLUMNS x UNIQUE"];
        let errors = [
            (1, q(QueryError::NotExecuted)),
            (2, q(QueryError::NotExecuted)),
            (3, surrealdb::Error::thrown("index already contains 'dup'".into())),
            (4, q(QueryError::NotExecuted)),
        ]
        .into();
        let msg = describe(&stmts, errors, 1).to_string();
        assert!(msg.contains("statement #2: index already contains 'dup'"), "{msg}");
        assert!(msg.contains("DEFINE INDEX IF NOT EXISTS i ON TABLE t"), "{msg}");
        assert!(msg.contains("rolled back") && msg.contains("1 errored"), "{msg}");
        assert!(!msg.contains("not executed"), "cascade noise must be filtered: {msg}");
    }
}
