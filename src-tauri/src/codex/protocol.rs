use serde_json::Value;

pub fn response_id(message: &Value) -> Option<u64> {
    message.get("id")?.as_u64()
}

pub fn notification_method(message: &Value) -> Option<&str> {
    if message.get("id").is_none() {
        message.get("method")?.as_str()
    } else {
        None
    }
}

pub fn sanitized_error(message: &Value) -> String {
    message
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("Codex app-server returned an error")
        .chars()
        .take(180)
        .collect()
}

pub fn is_auth_error(message: &Value) -> bool {
    let error = message.get("error").unwrap_or(message);
    let text = error.to_string().to_ascii_lowercase();
    text.contains("token_invalidated")
        || text.contains("authentication token has been invalidated")
        || text.contains("http 401")
        || text.contains("\"status\":401")
        || text.contains("unauthorized")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_nested_auth_failures() {
        assert!(is_auth_error(&json!({
            "error": {
                "message": "response: HTTP 401",
                "data": {"code": "token_invalidated", "status": 401}
            }
        })));
        assert!(!is_auth_error(&json!({
            "error": {"message": "app-server unavailable"}
        })));
    }
}
