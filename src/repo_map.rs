use std::fs;
use std::path::Path;
use tree_sitter::{Node, Parser};

pub fn extract_repo_map(workspace_path: &Path) -> Result<String, String> {
    let mut parser = Parser::new();
    parser
        .set_language(tree_sitter_rust::language())
        .map_err(|e| format!("Failed to set language: {e}"))?;

    let mut result = Vec::with_capacity(32);
    visit_dirs(workspace_path, workspace_path, &mut parser, &mut result)?;

    Ok(result.join("\n\n"))
}

fn visit_dirs(
    root: &Path,
    dir: &Path,
    parser: &mut Parser,
    result: &mut Vec<String>,
) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)
        .map_err(|e| format!("Failed to read dir: {e}"))?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    // 順序を安定させるためにソート
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // 除外フォルダ
        if name.starts_with('.') || name == "target" || name == "node_modules" {
            continue;
        }

        if path.is_dir() {
            visit_dirs(root, &path, parser, result)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let rel_path = path.strip_prefix(root).unwrap_or(&path);
            let file_summary = process_rust_file(&path, rel_path, parser)?;
            if !file_summary.is_empty() {
                result.push(file_summary);
            }
        }
    }

    Ok(())
}

fn process_rust_file(path: &Path, rel_path: &Path, parser: &mut Parser) -> Result<String, String> {
    let code = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file {}: {}", path.display(), e))?;

    let tree = parser
        .parse(&code, None)
        .ok_or_else(|| format!("Failed to parse code in {}", path.display()))?;

    let mut signatures = Vec::new();
    let root = tree.root_node();
    let mut cursor = root.walk();
    for child in root.children(&mut cursor) {
        if let Some(sig) = get_signature(&child, &code) {
            signatures.push(format!("  {}", sig));
        }
    }

    if signatures.is_empty() {
        Ok(String::new())
    } else {
        Ok(format!(
            "File: {}\n{}",
            rel_path.display(),
            signatures.join("\n")
        ))
    }
}

fn get_signature(node: &Node, code: &str) -> Option<String> {
    let kind = node.kind();
    if kind != "struct_item"
        && kind != "impl_item"
        && kind != "function_item"
        && kind != "trait_item"
    {
        return None;
    }

    let start = node.start_byte();
    let end = node.end_byte();
    let text = code.get(start..end)?;

    if let Some(pos) = text.find('{') {
        let sig = text[..pos].trim();
        let cleaned = sig.lines().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
        Some(cleaned)
    } else {
        let sig = text.lines().next().unwrap_or(text).trim();
        Some(sig.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_extraction() -> Result<(), Box<dyn std::error::Error>> {
        let mut parser = Parser::new();
        parser.set_language(tree_sitter_rust::language())?;

        let code = r#"
            pub struct MyStruct {
                pub x: i32,
            }

            impl MyStruct {
                pub fn new() -> Self {
                    Self { x: 0 }
                }
            }

            fn my_private_function(a: String) -> Option<usize> {
                None
            }

            pub trait Run {
                fn run(&self);
            }
        "#;

        let tree = parser.parse(code, None).ok_or("Failed to parse code")?;
        let root = tree.root_node();
        let mut cursor = root.walk();

        let mut signatures = Vec::new();
        for child in root.children(&mut cursor) {
            if let Some(sig) = get_signature(&child, code) {
                signatures.push(sig);
            }
        }

        assert_eq!(signatures.len(), 4);
        assert_eq!(signatures[0], "pub struct MyStruct");
        assert_eq!(signatures[1], "impl MyStruct");
        assert_eq!(
            signatures[2],
            "fn my_private_function(a: String) -> Option<usize>"
        );
        assert_eq!(signatures[3], "pub trait Run");

        Ok(())
    }
}
