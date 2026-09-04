use serde::Serialize;

use crate::protocol::RecipeDiagnostic;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInstance {
    pub id: String,
    pub component_id: String,
    pub locator: serde_json::Value,
    pub parsed: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentFinding {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentScanResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<ComponentInstance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<ComponentFinding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInvokeResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<ComponentInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<ComponentFinding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<crate::protocol::FilePreview>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<Vec<String>>,
}

pub fn instance_id(
    component_id: &str,
    parsed: &serde_json::Value,
    locator: &serde_json::Value,
) -> String {
    if let Some(id) = parsed.get("id").and_then(|v| v.as_str()) {
        return format!("{component_id}/{id}");
    }
    if let Some(path) = context_path(locator) {
        if let Some(stem) = crate::sandbox::path_template::stem_from_path(path) {
            return format!("{component_id}/{stem}");
        }
    }
    format!("{component_id}/unknown")
}

pub fn context_path(locator: &serde_json::Value) -> Option<&str> {
    locator
        .get("source")
        .and_then(|s| s.get("path"))
        .and_then(|v| v.as_str())
        .or_else(|| locator.get("path").and_then(|v| v.as_str()))
        .or_else(|| locator.as_str())
}

pub fn normalize_parse_context(locator: serde_json::Value) -> serde_json::Value {
    let locator = match locator {
        serde_json::Value::String(path) => serde_json::json!({ "path": path }),
        other => other,
    };
    if locator.get("source").is_some() {
        return locator;
    }
    if let Some(path) = locator.get("path").and_then(|v| v.as_str()) {
        let mut obj = locator
            .as_object()
            .cloned()
            .unwrap_or_default();
        obj.insert(
            "source".to_string(),
            serde_json::json!({ "path": path }),
        );
        return serde_json::Value::Object(obj);
    }
    locator
}

pub fn expand_parse_results(parsed: serde_json::Value) -> Result<Vec<serde_json::Value>, String> {
    if parsed.is_null() {
        return Ok(Vec::new());
    }
    if let Some(obj) = parsed.as_object() {
        if obj.get("skip").and_then(|v| v.as_bool()) == Some(true) {
            return Ok(Vec::new());
        }
        if let Some(instances) = obj.get("instances").and_then(|v| v.as_array()) {
            return Ok(instances.clone());
        }
    }
    Ok(vec![parsed])
}

pub fn soft_validate_parsed(
    instance_id: &str,
    locator: &serde_json::Value,
    parsed: &serde_json::Value,
) -> Vec<ComponentFinding> {
    let mut findings = Vec::new();
    if parsed.get("id").and_then(|v| v.as_str()).is_none() {
        findings.push(ComponentFinding {
            severity: "warning".to_string(),
            code: "parse.missing_id".to_string(),
            message: "parse result is missing id".to_string(),
            instance_id: Some(instance_id.to_string()),
            locator: Some(locator.clone()),
        });
    }
    findings
}

pub fn normalize_findings(
    raw: serde_json::Value,
    default_instance_id: &str,
    default_locator: &serde_json::Value,
) -> Vec<ComponentFinding> {
    let items = match raw {
        serde_json::Value::Array(arr) => arr,
        serde_json::Value::Null => return Vec::new(),
        serde_json::Value::Object(map) if map.is_empty() => return Vec::new(),
        other => vec![other],
    };
    items
        .into_iter()
        .filter_map(|item| {
            let obj = item.as_object()?;
            Some(ComponentFinding {
                severity: obj
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("error")
                    .to_string(),
                code: obj
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("finding")
                    .to_string(),
                message: obj
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("component finding")
                    .to_string(),
                instance_id: obj
                    .get("instanceId")
                    .or_else(|| obj.get("instance_id"))
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
                    .or_else(|| Some(default_instance_id.to_string())),
                locator: obj
                    .get("locator")
                    .cloned()
                    .or_else(|| Some(default_locator.clone())),
            })
        })
        .collect()
}
