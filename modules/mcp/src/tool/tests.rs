use super::render_result;

#[test]
fn text_content_blocks_are_joined() {
    let res = serde_json::json!({
        "result": { "content": [
            { "type": "text", "text": "first" },
            { "type": "text", "text": "second" }
        ]}
    });
    assert_eq!(render_result(&res), "first\nsecond");
}

/// Tool-level failure arrives as `isError`, not as a JSON-RPC error. The
/// orchestrator's circuit breaker counts consecutive failures by looking for a
/// leading `Error:`, so the prefix is added rather than left to the server's
/// own wording.
#[test]
fn an_is_error_result_is_prefixed() {
    let res = serde_json::json!({
        "result": { "content": [{ "type": "text", "text": "no such file" }], "isError": true }
    });
    assert_eq!(render_result(&res), "Error: no such file");
}

#[test]
fn an_is_error_result_that_already_says_error_is_not_prefixed_twice() {
    let res = serde_json::json!({
        "result": { "content": [{ "type": "text", "text": "Error: already said" }], "isError": true }
    });
    assert_eq!(render_result(&res), "Error: already said");
}

#[test]
fn a_jsonrpc_error_is_reported_as_one() {
    let res = serde_json::json!({ "error": { "code": -32601, "message": "method not found" } });
    assert_eq!(
        render_result(&res),
        "Error from MCP server: method not found"
    );
}

#[test]
fn an_empty_result_says_so_rather_than_returning_nothing() {
    let res = serde_json::json!({ "result": { "content": [] } });
    assert_eq!(render_result(&res), "No content returned from MCP server.");
}

/// Non-text blocks (images, resources) are dropped rather than rendered. The
/// extension did the same; recording it so a future change is a decision.
#[test]
fn non_text_blocks_are_skipped() {
    let res = serde_json::json!({
        "result": { "content": [
            { "type": "image", "data": "...", "mimeType": "image/png" },
            { "type": "text", "text": "caption" }
        ]}
    });
    assert_eq!(render_result(&res), "caption");
}
