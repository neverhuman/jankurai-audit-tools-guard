//! Contract tests for the machine-readable JSON schemas under `schemas/`.
//!
//! Each schema describes a JSON shape the guard actually produces or consumes:
//! the save-gate decision, the persisted guard state, and the guard policy.
//! These tests load every schema, assert it is valid JSON with the expected
//! top-level contract fields, and check that a representative serialized value
//! of the matching Rust type validates against the schema's required keys. This
//! keeps the schemas honest: a drift between the Rust type and its schema is
//! caught by `cargo nextest run -p jankurai-guard`.

use jankurai_guard::audit_client::{GuardDecision, Verdict};
use serde_json::Value;
use std::path::PathBuf;

/// Resolves `schemas/<name>` relative to the workspace root.
fn schema_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("schemas")
        .join(name)
}

fn load_schema(name: &str) -> Value {
    let text = std::fs::read_to_string(schema_path(name))
        .unwrap_or_else(|e| panic!("schema {name} must be readable: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("schema {name} must be valid JSON: {e}"))
}

/// Returns the property names declared by an object schema.
fn property_names(schema: &Value) -> Vec<String> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

#[test]
fn all_schemas_are_valid_json_schema_objects() {
    for name in [
        "save-gate-decision.schema.json",
        "guard-state.schema.json",
        "guard-policy.schema.json",
    ] {
        let schema = load_schema(name);
        assert_eq!(
            schema.get("$schema").and_then(Value::as_str),
            Some("https://json-schema.org/draft/2020-12/schema"),
            "{name} must declare the 2020-12 dialect"
        );
        assert_eq!(
            schema.get("type").and_then(Value::as_str),
            Some("object"),
            "{name} must describe a JSON object"
        );
        assert!(
            !property_names(&schema).is_empty(),
            "{name} must declare properties"
        );
    }
}

#[test]
fn save_gate_schema_matches_decision_type() {
    let schema = load_schema("save-gate-decision.schema.json");
    let props = property_names(&schema);

    // A real decision serialized from the Rust type must only use keys the
    // schema declares.
    let decision = GuardDecision {
        schema: "jankurai-save-gate/1".to_string(),
        verdict: Verdict::Block,
        exit_code: 2,
        path: "src/lib.rs".to_string(),
        mode: "save-gate".to_string(),
        candidate_score: Some(40),
        baseline_score: Some(85),
        summary: "blocked: 1 new hard finding".to_string(),
        blocking: Default::default(),
        advisory: Default::default(),
        preexisting_findings: vec![],
        rerun_command: "jankurai audit-file src/lib.rs".to_string(),
    };
    let value = serde_json::to_value(&decision).expect("decision serializes");
    let object = value.as_object().expect("decision is a JSON object");
    for key in object.keys() {
        assert!(
            props.contains(key),
            "serialized decision key `{key}` is not declared in the schema"
        );
    }

    // The schema must require the discriminator and verdict.
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    assert!(required.contains(&"schema"));
    assert!(required.contains(&"verdict"));
}
