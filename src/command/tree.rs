use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

struct TreeRenderer<'a> {
    dag: &'a rad_models::Dag,
    child_map: HashMap<&'a str, Vec<&'a str>>,
    current_id: Option<&'a str>,
    visited: HashSet<String>,
    output: String,
}

impl<'a> TreeRenderer<'a> {
    fn new(dag: &'a rad_models::Dag) -> Self {
        let mut child_map: HashMap<&str, Vec<&str>> = HashMap::new();
        for (id, node) in &dag.nodes {
            for parent in &node.parent_ids {
                child_map.entry(parent.as_str()).or_default().push(id.as_str());
            }
        }
        for children in child_map.values_mut() {
            children.sort_unstable();
        }

        Self {
            dag,
            child_map,
            current_id: dag.current_node_id.as_deref(),
            visited: HashSet::new(),
            output: String::new(),
        }
    }

    fn render(&mut self) -> String {
        let mut roots = Vec::new();
        for (id, node) in &self.dag.nodes {
            if node.parent_ids.is_empty() {
                roots.push(id.as_str());
            }
        }
        roots.sort_unstable();

        for (i, root) in roots.iter().enumerate() {
            let is_last = i == roots.len() - 1;
            self.render_node(root, "", is_last);
        }

        std::mem::take(&mut self.output)
    }

    fn render_node(&mut self, node_id: &str, prefix: &str, is_last: bool) {
        if !self.visited.insert(node_id.to_string()) {
            return;
        }

        let Some(node) = self.dag.nodes.get(node_id) else {
            return;
        };

        let (marker, connector) = get_node_symbols(node_id, self.current_id, prefix, is_last);
        let short_text = format_node_text(node);

        let _ = writeln!(
            self.output,
            "{}{}{} {marker} [{}] - {}",
            prefix, connector, node.id, node.node_type, short_text
        );

        if let Some(children) = self.child_map.get(node_id).cloned() {
            let new_prefix = if prefix.is_empty() {
                String::new()
            } else if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };
            for (i, child) in children.iter().enumerate() {
                let child_is_last = i == children.len() - 1;
                self.render_node(child, &new_prefix, child_is_last);
            }
        }
    }
}

/// Renders the DAG history as an ASCII tree.
#[must_use]
pub fn render_dag_tree(dag: &rad_models::Dag) -> String {
    let mut renderer = TreeRenderer::new(dag);
    renderer.render()
}

fn get_node_symbols(
    node_id: &str,
    current_id: Option<&str>,
    prefix: &str,
    is_last: bool,
) -> (&'static str, &'static str) {
    let marker = if Some(node_id) == current_id {
        "*"
    } else {
        " "
    };

    let connector = if prefix.is_empty() {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    (marker, connector)
}

fn format_node_text(node: &rad_models::DagNode) -> String {
    let text_cleaned = node.text.replace('\n', " ");
    if text_cleaned.len() > 40 {
        format!("{}...", &text_cleaned[..40])
    } else {
        text_cleaned
    }
}
