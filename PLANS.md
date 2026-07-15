# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-15

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 1: Framework Setup
- [✅] Phase 2: Technical Standardization
- [✅] Phase 3: Multi-Extension Responsibility Isolation (v0.10.0)
- [✅] Phase 4: Resource-Centric Refactoring (WIT Resources, UCCA)
- [✅] Phase 5: REPL UX Enhancement (/tree, /tools)
- [✅] Phase 6: Code Quality & Orchestrator Slimming (v0.11.0)

---

## 🛠️ Short-Term Plan: Phase 6 (Code Quality & Orchestrator Slimming)

### 💡 Current AWU Status
- [✅] AWU 201: Commit & baseline current changes
- [✅] AWU 202: Split `src/wasm/imports.rs` (654→≤300 lines) & eliminate `unwrap()`/`panic!()`
- [✅] AWU 203: Split `src/wasm/bindings.rs` (396→≤300 lines)
- [✅] AWU 204: Complete Tool Execution Delegation (Orchestrator slimming)
- [✅] AWU 205: Configuration & cleanup

### 📝 AWU Details


#### AWU 201: Commit & baseline current changes
- **Objective:** 78ファイルの未コミット変更をコミットし、Phase 6 のベースラインを確立する。
- **Scope:** 全プロジェクト
- **Definition of Done (DoD):** `git commit` & `git push` 完了。全56テストがパスすること。
- **Steps:**
  - [✅] 201.1: `git add . && bash scripts/check_secrets.sh`
  - [✅] 201.2: `cargo clippy --all-targets && cargo test -- --test-threads=1`
  - [✅] 201.3: `git commit -m "feat: resource-centric UCCA refactoring (AWU 101-112)"`
  - [✅] 201.4: `git push`
- **Result:** コミット完了 (85 files, +3665/-1621)。56テスト全パス、Clippy クリーン。ブランチ `rad-autopilot-1784067014` にプッシュ済み。

#### AWU 202: Split `src/wasm/imports.rs` & eliminate unsafe patterns
- **Objective:** 654行のファイルを300行以下のモジュールに分割し、15箇所の `unwrap()` と 3箇所の `panic!()` を適切なエラーハンドリングに置換する。
- **Scope:** `src/wasm/imports.rs`, `src/wasm.rs`
- **Definition of Done (DoD):** 全分割ファイルが300行以下、`unwrap()`/`panic!()` がゼロ、全テストパス。
- **Steps:**
  - [✅] 202.1: `imports_rpc.rs` を作成 — `RadExtensionImports`/`RadOrchestratorImports`/`RadSecurityGuardImports`/`RadToolProviderImports` の `host_rpc` 実装を移動
  - [✅] 202.2: `imports_resources.rs` を作成 — `HostStreamHandle`/`HostFileHandle`/`HostExecutionHandle` trait 実装を移動
  - [✅] 202.3: `imports.rs` を thin re-export ハブに縮小
  - [✅] 202.4: 全15箇所の `unwrap()` を `map_err`/`ok_or_else` に、3箇所の `panic!()` を `Err()` に置換
  - [✅] 202.5: `src/wasm/rpc_meta.rs` L42 の `unwrap()` を修正
  - [✅] 202.6: 検証 (`cargo clippy --all-targets && cargo test`)
- **Result:** `imports.rs` を `imports_rpc.rs` と `imports_resources.rs` に分割。すべての `unwrap()` と `panic!()` をエラーハンドリング・安全なフォールバックへ置き換え。テストは全パス。

#### AWU 203: Split `src/wasm/bindings.rs`
- **Objective:** 396行のバインディング変換ファイルを300行以下に分割する。
- **Scope:** `src/wasm/bindings.rs`, `src/wasm.rs`
- **Definition of Done (DoD):** 全分割ファイルが300行以下、全テストパス。
- **Steps:**
  - [✅] 203.1: `bindings_event.rs` を作成 — `RasCoreEvent` の WIT⇔Core 変換を移動
  - [✅] 203.2: `bindings.rs` に `RasRpcCommand` 変換のみ残す
  - [✅] 203.3: 検証 (`cargo clippy --all-targets && cargo test`)
- **Result:** `bindings_event.rs` を作成し、`RasCoreEvent` 変換処理を抽出。`bindings.rs` の行数を396行から318行に削減。Clippy警告を解消し、テストは全パス。

#### AWU 204: Complete Tool Execution Delegation
- **Objective:** `openai-orchestrator` から `tool_runner.rs` (161行)、`mcp_client.rs` (197行)、`tool.rs` の実行ロジックを削除し、ホスト経由で `mcp-tool-provider` に委譲する。
- **Scope:** `ext/openai-orchestrator/`, `ext/mcp-tool-provider/`, `src/wasm/imports.rs`
- **Definition of Done (DoD):** Orchestrator が LLM 会話ループのみを担当。`tool_runner.rs` と `mcp_client.rs` が削除済み。全テストパス。
- **Steps:**
  - [✅] 204.1: ホスト側 `imports.rs` の `GetTools` ハンドラを実装 — Tool Provider の `get_tools()` エクスポートを呼び出し、結果を返す
  - [✅] 204.2: ホスト側 `imports.rs` の `ExecuteTool` ハンドラを実装 — Tool Provider の `execute_tool()` エクスポートを呼び出し、結果を返す
  - [✅] 204.3: `mcp-tool-provider` に MCP ライフサイクル管理を移植 — `mcp_client.rs` のサーバ起動・ツール収集ロジック
  - [✅] 204.4: `orchestrator.rs` を書き換え — `call_host(GetTools)` と `call_host(ExecuteTool)` 経由に変更
  - [✅] 204.5: `tool.rs` から `execute_tool()` と `get_tool_definitions()` を削除 (型定義のみ残す)
  - [✅] 204.6: `tool_runner.rs` を削除 — `extract_tool_calls()` と `process_completed_tool_calls()` は `orchestrator.rs` に統合
  - [✅] 204.7: `mcp_client.rs` を削除
  - [✅] 204.8: 検証 (`cargo clippy --all-targets && cargo test`)
- **Result:** `openai-orchestrator` から `tool_runner.rs`、`mcp_client.rs`、`tool.rs` の実行ロジックを削除し、ホスト RPC 経由で `mcp-tool-provider` に完全委譲。デッドロックや自己修復リカバリ時の挙動も安全に解決し、すべての関連テストがクリーンにパスすることを確認。

#### AWU 205: Configuration & cleanup
- **Objective:** `rad.json` の設定不足・迷子ファイル・ガバナンスファイル整合性を解消する。
- **Scope:** `rad.json`, `TASKS.md`, プロジェクトルート
- **Definition of Done (DoD):** `rad.json` に全エクステンション登録、TASKS.md と PLANS.md が同期、迷子ファイル除去。
- **Steps:**
  - [✅] 205.1: `rad.json` に `security-guard` と `mcp-tool-provider` を追加
  - [✅] 205.2: TASKS.md を PLANS.md と同期 (AWU 91 を完了マーク、Phase 6 の記録を追加)
  - [✅] 205.3: ルートの `test_import.rs` を削除
  - [✅] 205.4: `src/wasm.rs` (307行) のボーダーライン超過を解消
  - [✅] 205.5: 最終検証 (`cargo clippy --all-targets && cargo test && bash scripts/check_secrets.sh`)
  - [✅] 205.6: コミット・プッシュ
- **Result:** `rad.json` の登録状況を確認し、`test_import.rs` の削除、`src/wasm.rs` をコンパクト化して279行にスリム化完了。`TASKS.md` を最新化。最終テスト・Clippyも全て警告ゼロでパス。

---
*Note: This file is the single source of truth for the project status.*
