//! The conversation graph, as a kernel module (ARCHITECTURE-NEXT.md §9.3).
//!
//! Stage 9's decision (A): the module owns the graph and persists it itself,
//! rather than being a window onto storage the host still owns. The graph moves
//! from `models/src/dag.rs` unchanged; what is new is that saving happens here,
//! on every mutation, instead of once per completed task in `src/main.rs`.
//!
//! The host reads and writes it through these methods during the migration
//! (AWU 986) and stops holding a copy at all when that lands.
#![deny(clippy::pedantic)]

mod graph;
mod store;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct OpenReq {
    pub workspace: String,
    pub session_id: String,
}

#[derive(serde::Deserialize)]
pub struct CreateNodeReq {
    /// Empty means a root node, exactly as `Dag::create_node` reads it.
    #[serde(default)]
    pub parent_id: String,
    pub node_type: String,
}

#[derive(serde::Deserialize)]
pub struct SetNodeTextReq {
    pub node_id: String,
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct MergeNodesReq {
    pub node_ids: Vec<String>,
    pub summary_text: String,
}

#[derive(serde::Deserialize)]
pub struct NodeReq {
    pub node_id: String,
}

#[derive(serde::Deserialize)]
pub struct GetReq {}

#[derive(serde::Serialize)]
pub struct IdRes {
    pub id: String,
}

#[derive(serde::Serialize)]
pub struct OkRes {
    pub ok: bool,
}

fn open(req: OpenReq) -> Result<OkRes, Error> {
    let OpenReq {
        workspace,
        session_id,
    } = req;
    store::open(&workspace, &session_id).map_err(Error::io)?;
    Ok(OkRes { ok: true })
}

fn create_node(req: CreateNodeReq) -> Result<IdRes, Error> {
    let CreateNodeReq {
        parent_id,
        node_type,
    } = req;
    let id =
        store::mutate(|dag| dag.create_node(&parent_id, &node_type)).map_err(Error::invalid)?;
    Ok(IdRes { id })
}

fn set_node_text(req: SetNodeTextReq) -> Result<OkRes, Error> {
    let SetNodeTextReq { node_id, text } = req;
    store::mutate(|dag| dag.set_node_text(&node_id, &text)).map_err(Error::invalid)?;
    Ok(OkRes { ok: true })
}

fn merge_nodes(req: MergeNodesReq) -> Result<IdRes, Error> {
    let MergeNodesReq {
        node_ids,
        summary_text,
    } = req;
    let id =
        store::mutate(|dag| dag.merge_nodes(&node_ids, &summary_text)).map_err(Error::invalid)?;
    Ok(IdRes { id })
}

fn delete_node(req: NodeReq) -> Result<OkRes, Error> {
    let NodeReq { node_id } = req;
    store::mutate(|dag| dag.delete_node(&node_id)).map_err(Error::invalid)?;
    Ok(OkRes { ok: true })
}

/// The whole graph, in the shape `GetDag` returns and `kernel.dag` returns —
/// the one every existing reader already parses.
fn get(_req: GetReq) -> Result<serde_json::Value, Error> {
    let value = store::read(|dag| serde_json::to_value(dag)).map_err(Error::io)?;
    value.map_err(|e| Error::invalid(format!("could not serialise the graph: {e}")))
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "dag",
    version: "0.1.0",
    methods: {
        "dag.open"          => open,
        "dag.get"           => get,
        "dag.create_node"   => create_node,
        "dag.set_node_text" => set_node_text,
        "dag.merge_nodes"   => merge_nodes,
        "dag.delete_node"   => delete_node,
    }
}
