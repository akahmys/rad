#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::same_length_and_capacity)]

wit_bindgen::generate!({
    path: "../../wit/web-access.wit",
    world: "web-access-extension",
});

use crate::exports::radcomp::web_access::web_access::Guest;
use crate::radcomp::web_access::host_rpc;
use crate::radcomp::web_access::types::RasRpcCommand;

struct MyWebAccess;

impl MyWebAccess {
    fn clean_html(html: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        let mut current_tag = String::new();
        let mut skip_content = false;
        let mut skip_tag_depth = 0;
        let mut skip_tags = std::collections::HashSet::new();
        skip_tags.insert("script".to_string());
        skip_tags.insert("style".to_string());
        skip_tags.insert("head".to_string());
        skip_tags.insert("nav".to_string());
        skip_tags.insert("footer".to_string());

        let chars: Vec<char> = html.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == '<' {
                in_tag = true;
                current_tag.clear();
            } else if c == '>' {
                in_tag = false;
                if !result.is_empty() && !result.ends_with(' ') {
                    result.push(' ');
                }
                let tag_lower = current_tag.to_lowercase();
                if tag_lower.starts_with('/') {
                    let closed_tag = tag_lower.trim_start_matches('/');
                    if skip_tags.contains(closed_tag) {
                        if skip_tag_depth > 0 {
                            skip_tag_depth -= 1;
                        }
                        if skip_tag_depth == 0 {
                            skip_content = false;
                        }
                    }
                } else {
                    let opened_tag = tag_lower.split_whitespace().next().unwrap_or("");
                    if skip_tags.contains(opened_tag) {
                        skip_content = true;
                        skip_tag_depth += 1;
                    }
                }
            } else if in_tag {
                current_tag.push(c);
            } else if !skip_content {
                result.push(c);
            }
            i += 1;
        }

        // Clean up whitespace
        let mut cleaned = String::new();
        let mut last_was_space = false;
        for c in result.chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    cleaned.push(' ');
                    last_was_space = true;
                }
            } else {
                cleaned.push(c);
                last_was_space = false;
            }
        }
        cleaned.trim().to_string()
    }
}

impl Guest for MyWebAccess {
    fn search(query: String) -> Result<String, String> {
        let tavily_key = std::env::var("TAVILY_API_KEY").unwrap_or_default();
        if !tavily_key.is_empty() {
            let req_body = serde_json::json!({
                "api_key": tavily_key,
                "query": query,
                "max_results": 5
            });
            let req_str = req_body.to_string();
            let _ = std::fs::write("search_req.json", req_str);

            let cmd = "curl -sS -X POST -H 'Content-Type: application/json' -d @search_req.json https://api.tavily.com/search";
            let res = host_rpc::call(&RasRpcCommand::Command(cmd.to_string()))?;
            let _ = std::fs::remove_file("search_req.json");

            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&res) {
                if let Some(results) = json_val.get("results").and_then(|r| r.as_array()) {
                    let mut formatted = Vec::new();
                    for (idx, item) in results.iter().enumerate() {
                        let title = item
                            .get("title")
                            .and_then(|t| t.as_str())
                            .unwrap_or("No Title");
                        let url = item.get("url").and_then(|u| u.as_str()).unwrap_or("No URL");
                        let content = item.get("content").and_then(|c| c.as_str()).unwrap_or("");
                        formatted.push(format!("{}. {title} ({url})\n   {content}", idx + 1));
                    }
                    return Ok(formatted.join("\n\n"));
                }
            }
        }

        // Fallback: DuckDuckGo Instant Answer API
        let encoded_query: String = query
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u32)
                }
            })
            .collect();

        let cmd =
            format!("curl -sS -L 'https://api.duckduckgo.com/?q={encoded_query}&format=json'");
        if let Ok(res) = host_rpc::call(&RasRpcCommand::Command(cmd)) {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&res) {
                let mut formatted = Vec::new();
                if let Some(abs_text) = json_val.get("AbstractText").and_then(|a| a.as_str()) {
                    if !abs_text.is_empty() {
                        formatted.push(format!("Instant Answer:\n{abs_text}"));
                    }
                }
                if let Some(related) = json_val.get("RelatedTopics").and_then(|r| r.as_array()) {
                    let mut idx = 1;
                    for item in related {
                        if let Some(text) = item.get("Text").and_then(|t| t.as_str()) {
                            let url = item.get("FirstURL").and_then(|u| u.as_str()).unwrap_or("");
                            formatted.push(format!("{idx}. {text} ({url})"));
                            idx += 1;
                            if idx > 5 {
                                break;
                            }
                        }
                    }
                }
                if !formatted.is_empty() {
                    return Ok(formatted.join("\n\n"));
                }
            }
        }

        Ok(format!("No search results found for query: {query}"))
    }

    fn fetch(url: String) -> Result<String, String> {
        let cmd = format!("curl -sSL -A 'Mozilla/5.0 (Windows NT 10.0; Win64; x64)' '{url}'");
        let html = host_rpc::call(&RasRpcCommand::Command(cmd))?;
        let cleaned = Self::clean_html(&html);
        if cleaned.is_empty() {
            Err("Empty content fetched or failed to scrape page".to_string())
        } else {
            Ok(cleaned)
        }
    }
}

export!(MyWebAccess);

#[cfg(test)]
mod tests;
