//! Local OpenAPI contract checks; references never perform I/O.
use super::{
    input::{Language, SourceDocument},
    rules::{finding, Rule, RuleResult, RuleSet, Selector},
    ReviewStatus,
};
use anyhow::{ensure, Result};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

type Stop<'a> = &'a dyn Fn() -> Option<&'static str>;
impl RuleSet {
    pub fn review_boundaries(
        &self,
        documents: &[SourceDocument],
        disabled: &[String],
        stop: Stop<'_>,
    ) -> Result<Vec<RuleResult>> {
        let mut results = Vec::new();
        let mut operations = 0;
        for document in documents {
            let mut findings = 0;
            if let Some(reason) = stop() {
                anyhow::bail!(reason);
            }
            if !matches!(document.language, Language::Json | Language::Yaml)
                || !document.text.contains("openapi")
            {
                continue;
            }
            ensure!(
                document.text.len() <= 1_048_576,
                "size_limit: OpenAPI document"
            );
            // Reject YAML anchors/aliases before deserialization so repeated
            // aliases cannot expand a bounded source into unbounded memory.
            static ALIAS: OnceLock<regex::Regex> = OnceLock::new();
            if document.language == Language::Yaml
                && ALIAS
                    .get_or_init(|| {
                        regex::Regex::new(r"(?m)(?:^|[\s\[\]{},:])[&*][^\s\[\]{},]+")
                            .expect("YAML alias guard")
                    })
                    .is_match(&document.text)
            {
                results.extend(self.boundary_unverified(
                    &document.path,
                    "unsupported_yaml_feature",
                    disabled,
                ));
                continue;
            }
            let parsed: std::result::Result<Value, _> = serde_yaml::from_str(&document.text);
            let root = match parsed {
                Ok(root) => root,
                Err(_) => {
                    results.extend(self.boundary_unverified(
                        &document.path,
                        "parse_error",
                        disabled,
                    ));
                    continue;
                }
            };
            let version = root["openapi"].as_str().unwrap_or("");
            if !version.starts_with("3.0.") && !version.starts_with("3.1.") {
                results.extend(self.boundary_unverified(
                    &document.path,
                    "unsupported_openapi_version",
                    disabled,
                ));
                continue;
            }
            let Some(paths) = root["paths"].as_object() else {
                results.extend(self.boundary_unverified(
                    &document.path,
                    "missing_operations",
                    disabled,
                ));
                continue;
            };
            let mut document_operations = 0;
            for (path, raw_item) in paths {
                let item = match resolve(&root, raw_item, &mut BTreeSet::new(), 0) {
                    Ok(item) => item,
                    Err(reason) => {
                        results.extend(self.boundary_unverified(&document.path, reason, disabled));
                        continue;
                    }
                };
                for method in [
                    "get", "put", "post", "delete", "options", "head", "patch", "trace",
                ] {
                    let Some(operation) = item.get(method) else {
                        continue;
                    };
                    if let Some(reason) = stop() {
                        anyhow::bail!(reason);
                    }
                    operations += 1;
                    document_operations += 1;
                    ensure!(
                        operations <= 1000,
                        "budget_exceeded: OpenAPI operation limit"
                    );
                    let line = document
                        .text
                        .lines()
                        .position(|line| line.contains(path))
                        .unwrap_or(0)
                        + 1;
                    let inputs = operation_inputs(&root, item, operation, stop);
                    for rule in self.rules.iter().filter(|r| r.rule_type == "api-boundary") {
                        if let Some(reason) = stop() {
                            anyhow::bail!(reason);
                        }
                        let (status, reason) = if disabled.contains(&rule.id) {
                            (ReviewStatus::Disabled, None)
                        } else if let Err(reason) = &inputs {
                            (ReviewStatus::NotVerified, Some(*reason))
                        } else {
                            check(
                                rule,
                                &root,
                                operation,
                                inputs.as_ref().unwrap(),
                                version,
                                stop,
                            )?
                        };
                        let mut result = RuleResult {
                            rule_id: rule.id.clone(),
                            path: document.path.clone(),
                            status,
                            reason: reason.map(str::to_owned),
                            findings: Vec::new(),
                        };
                        if status == ReviewStatus::Finding {
                            if findings >= 100 {
                                result.status = ReviewStatus::NotVerified;
                                result.reason = Some("finding_limit".into());
                            } else {
                                findings += 1;
                                let mut gap=finding(rule,document,line,"[REDACTED: inspect the OpenAPI operation or parameter at this location]".into());
                                gap.kind = "missing_requirement".into();
                                result.findings.push(gap);
                            }
                        }
                        results.push(result);
                    }
                }
            }
            if document_operations == 0 {
                results.extend(self.boundary_unverified(
                    &document.path,
                    "missing_operations",
                    disabled,
                ));
            }
        }
        if results.is_empty() {
            results = self.boundary_unverified("", "target_information_insufficient", disabled);
        }
        Ok(results)
    }
    fn boundary_unverified(
        &self,
        path: &str,
        reason: &str,
        disabled: &[String],
    ) -> Vec<RuleResult> {
        self.rules
            .iter()
            .filter(|r| r.rule_type == "api-boundary")
            .map(|r| RuleResult {
                rule_id: r.id.clone(),
                path: path.into(),
                status: if disabled.contains(&r.id) {
                    ReviewStatus::Disabled
                } else {
                    ReviewStatus::NotVerified
                },
                reason: Some(reason.into()),
                findings: Vec::new(),
            })
            .collect()
    }
}
struct Inputs<'a> {
    schemas: Vec<&'a Value>,
    null_handling: Vec<bool>,
}
fn resolve<'a>(
    root: &'a Value,
    node: &'a Value,
    seen: &mut BTreeSet<String>,
    depth: usize,
) -> std::result::Result<&'a Value, &'static str> {
    if depth > 32 {
        return Err("unresolved_ref");
    }
    let Some(reference) = node.get("$ref") else {
        return Ok(node);
    };
    let reference = reference.as_str().ok_or("unresolved_ref")?;
    if !reference.starts_with("#/") {
        return Err("external_ref_not_read");
    }
    if !seen.insert(reference.into()) {
        return Err("unresolved_ref");
    }
    let target = root.pointer(&reference[1..]).ok_or("unresolved_ref")?;
    resolve(root, target, seen, depth + 1)
}
fn collect_schema<'a>(
    root: &'a Value,
    schema: &'a Value,
    inputs: &mut Inputs<'a>,
    seen: &mut BTreeSet<String>,
    depth: usize,
    stop: Stop<'_>,
) -> std::result::Result<(), &'static str> {
    if let Some(reason) = stop() {
        return Err(reason);
    }
    if depth > 32 || inputs.schemas.len() >= 10_000 {
        return Err("unsupported_schema");
    }
    let schema = resolve(root, schema, seen, depth)?;
    if !schema.is_object()
        || [
            "allOf",
            "anyOf",
            "oneOf",
            "not",
            "if",
            "then",
            "else",
            "$id",
            "$schema",
            "$dynamicRef",
            "$dynamicAnchor",
            "patternProperties",
            "prefixItems",
            "unevaluatedProperties",
            "dependentSchemas",
            "contains",
        ]
        .iter()
        .any(|key| schema.get(key).is_some())
        || schema
            .get("additionalProperties")
            .is_some_and(Value::is_object)
    {
        return Err("unsupported_schema");
    }
    inputs.schemas.push(schema);
    if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
        for property in properties.values() {
            collect_schema(root, property, inputs, &mut seen.clone(), depth + 1, stop)?;
        }
    }
    if let Some(items) = schema.get("items") {
        collect_schema(root, items, inputs, &mut seen.clone(), depth + 1, stop)?;
    }
    Ok(())
}
fn operation_inputs<'a>(
    root: &'a Value,
    item: &'a Value,
    operation: &'a Value,
    stop: Stop<'_>,
) -> std::result::Result<Inputs<'a>, &'static str> {
    let mut inputs = Inputs {
        schemas: Vec::new(),
        null_handling: Vec::new(),
    };
    let mut effective = BTreeMap::new();
    for node in [item.get("parameters"), operation.get("parameters")]
        .into_iter()
        .flatten()
    {
        let parameters = node.as_array().ok_or("invalid_parameters")?;
        let mut names = BTreeSet::new();
        for parameter in parameters {
            let parameter = resolve(root, parameter, &mut BTreeSet::new(), 0)?;
            let key = (
                parameter["in"].as_str().ok_or("invalid_parameters")?,
                parameter["name"].as_str().ok_or("invalid_parameters")?,
            );
            if !names.insert(key) {
                return Err("invalid_parameters");
            }
            effective.insert(key, parameter);
        }
    }
    for parameter in effective.values() {
        let schema = parameter.get("schema").ok_or("missing_schema")?;
        let resolved = resolve(root, schema, &mut BTreeSet::new(), 0)?;
        inputs.null_handling.push(
            parameter.get("required").is_some_and(Value::is_boolean)
                || resolved.get("nullable").is_some_and(Value::is_boolean)
                || resolved["type"].as_array().is_some(),
        );
        collect_schema(root, schema, &mut inputs, &mut BTreeSet::new(), 0, stop)?;
    }
    if let Some(body) = operation.get("requestBody") {
        let body = resolve(root, body, &mut BTreeSet::new(), 0)?;
        let content = body["content"].as_object().ok_or("missing_schema")?;
        for media in content.values() {
            inputs
                .null_handling
                .push(body.get("required").is_some_and(Value::is_boolean));
            collect_schema(
                root,
                media.get("schema").ok_or("missing_schema")?,
                &mut inputs,
                &mut BTreeSet::new(),
                0,
                stop,
            )?;
        }
    }
    Ok(inputs)
}
fn has_type(schema: &Value, expected: &str) -> bool {
    schema["type"].as_str() == Some(expected)
        || schema["type"]
            .as_array()
            .is_some_and(|types| types.iter().any(|t| t.as_str() == Some(expected)))
}
fn check(
    rule: &Rule,
    root: &Value,
    operation: &Value,
    inputs: &Inputs<'_>,
    version: &str,
    stop: Stop<'_>,
) -> Result<(ReviewStatus, Option<&'static str>)> {
    use ReviewStatus::{Finding, NotApplicable, NotVerified, Pass};
    let ok = (Pass, Some("contract_only_runtime_not_verified"));
    let gap = (Finding, Some("missing_requirement"));
    let not_applicable = (NotApplicable, Some("no_applicable_input_type_in_operation"));
    Ok(match rule.check.selector {
        Selector::BoundaryNull => {
            if inputs.null_handling.is_empty() {
                not_applicable
            } else if inputs.null_handling.iter().all(|v| *v) {
                ok
            } else {
                gap
            }
        }
        Selector::BoundaryType => {
            if inputs.schemas.is_empty() {
                not_applicable
            } else if inputs.schemas.iter().all(|s| {
                s["type"].as_str().is_some_and(|t| {
                    matches!(
                        t,
                        "string" | "number" | "integer" | "object" | "array" | "boolean" | "null"
                    )
                }) || s["type"].as_array().is_some_and(|types| {
                    !types.is_empty()
                        && types.iter().all(|t| {
                            t.as_str().is_some_and(|t| {
                                matches!(
                                    t,
                                    "string"
                                        | "number"
                                        | "integer"
                                        | "object"
                                        | "array"
                                        | "boolean"
                                        | "null"
                                )
                            })
                        })
                })
            }) {
                ok
            } else {
                gap
            }
        }
        Selector::BoundaryLength => {
            let strings = inputs
                .schemas
                .iter()
                .filter(|s| has_type(s, "string"))
                .collect::<Vec<_>>();
            if strings.is_empty() {
                not_applicable
            } else if strings.iter().all(|s| {
                s["minLength"]
                    .as_u64()
                    .zip(s["maxLength"].as_u64())
                    .is_some_and(|(min, max)| min <= max)
            }) {
                ok
            } else {
                gap
            }
        }
        Selector::BoundaryNumber => {
            let numbers = inputs
                .schemas
                .iter()
                .filter(|s| has_type(s, "number") || has_type(s, "integer"))
                .collect::<Vec<_>>();
            if numbers.is_empty() {
                not_applicable
            } else if numbers.iter().all(|s| {
                let lower = s["minimum"].as_f64().or_else(|| {
                    version
                        .starts_with("3.1.")
                        .then(|| s["exclusiveMinimum"].as_f64())
                        .flatten()
                });
                let upper = s["maximum"].as_f64().or_else(|| {
                    version
                        .starts_with("3.1.")
                        .then(|| s["exclusiveMaximum"].as_f64())
                        .flatten()
                });
                lower.zip(upper).is_some_and(|(min, max)| min <= max)
            }) {
                ok
            } else {
                gap
            }
        }
        Selector::BoundaryInvalidInput => {
            if ["400", "422", "4XX"]
                .iter()
                .any(|key| operation["responses"].get(key).is_some())
            {
                ok
            } else {
                gap
            }
        }
        Selector::BoundaryAuthorization => {
            let security = operation
                .get("security")
                .or_else(|| root.get("security"))
                .and_then(Value::as_array);
            if let Some(security) = security.filter(|s| !s.is_empty()) {
                let mut valid = true;
                for requirement in security {
                    let Some(requirement) = requirement.as_object().filter(|r| !r.is_empty())
                    else {
                        valid = false;
                        break;
                    };
                    for name in requirement.keys() {
                        let scheme = &root["components"]["securitySchemes"][name];
                        match resolve(root, scheme, &mut BTreeSet::new(), 0) {
                            Ok(scheme) => {
                                valid &= matches!(
                                    scheme["type"].as_str(),
                                    Some(
                                        "http"
                                            | "apiKey"
                                            | "oauth2"
                                            | "openIdConnect"
                                            | "mutualTLS"
                                    )
                                )
                            }
                            Err(reason) => return Ok((NotVerified, Some(reason))),
                        }
                    }
                }
                if valid {
                    ok
                } else {
                    gap
                }
            } else {
                gap
            }
        }
        Selector::BoundaryErrors => {
            let Some(responses) = operation["responses"].as_object() else {
                return Ok((NotVerified, Some("error_schema_missing")));
            };
            let errors = responses
                .iter()
                .filter(|(status, _)| status.starts_with('4') || status.starts_with('5'))
                .map(|(_, value)| value)
                .collect::<Vec<_>>();
            if errors.is_empty() {
                return Ok((NotVerified, Some("error_schema_missing")));
            }
            let mut dangerous = false;
            for response in errors {
                let response = match resolve(root, response, &mut BTreeSet::new(), 0) {
                    Ok(r) => r,
                    Err(reason) => return Ok((NotVerified, Some(reason))),
                };
                let Some(content) = response["content"].as_object() else {
                    return Ok((NotVerified, Some("error_schema_missing")));
                };
                for media in content.values() {
                    let Some(schema) = media.get("schema") else {
                        return Ok((NotVerified, Some("error_schema_missing")));
                    };
                    let mut output = Inputs {
                        schemas: Vec::new(),
                        null_handling: Vec::new(),
                    };
                    if let Err(reason) =
                        collect_schema(root, schema, &mut output, &mut BTreeSet::new(), 0, stop)
                    {
                        return Ok((NotVerified, Some(reason)));
                    }
                    for schema in output.schemas {
                        if has_type(schema, "object")
                            && schema.get("additionalProperties") != Some(&Value::Bool(false))
                        {
                            return Ok((NotVerified, Some("unbounded_error_schema")));
                        }
                        dangerous |= schema["properties"].as_object().is_some_and(|p| {
                            p.keys().any(|key| {
                                matches!(
                                    key.to_ascii_lowercase().as_str(),
                                    "stack"
                                        | "stacktrace"
                                        | "trace"
                                        | "exception"
                                        | "debug"
                                        | "errordetails"
                                )
                            })
                        });
                    }
                }
            }
            if dangerous {
                gap
            } else {
                ok
            }
        }
        _ => anyhow::bail!("invalid_rules: selector is not an API boundary check"),
    })
}
