# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-15

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 1: Framework Setup
- [✅] Phase 2: Technical Standardization
- [✅] Phase 3: Multi-Extension Responsibility Isolation (v0.10.0)
- [✅] Phase 4: Resource-Centric Refactoring (WIT Resources, UCCA)
- [✅] Phase 5: REPL UX Enhancement (/tree, /tools)
- [✅] Phase 6: Code Quality & Orchestrator Slimming (v0.11.0)
- [🔄] Phase 7: Orchestrator Slimming & Abstraction (v0.12.0)

---

## 🛠️ Short-Term Plan: Phase 7 (Orchestrator Slimming & Abstraction)

### 💡 Current AWU Status
- [✅] AWU 301: Orchestrator Slimming Architectural Investigation
- [✅] AWU 302: Define `llm-connector.wit` Interface
- [✅] AWU 303: Update Core Host to support LLM Connector Extensions
- [✅] AWU 304: Implement Core Wasm-to-Wasm Event Routing
- [✅] AWU 305: Create `openai-connector` Extension Boilerplate
- [✅] AWU 306: Migrate HTTP/SSE logic to `openai-connector`
- [✅] AWU 307: Refactor `openai-orchestrator` to call the Connector
- [✅] AWU 308: Integration Testing & Verification

### 📝 AWU Details

#### AWU 301: Orchestrator Slimming Architectural Investigation
- **Objective:** Orchestratorの責任過多を解消するための具体的な新アーキテクチャ設計案を作成する。
- **Scope:** 設計ドキュメント作成
- **Definition of Done (DoD):** `orchestrator_slimming_design.md` を作成し、新インタフェースおよびデータフロー設計を提案する。
- **Result:** `orchestrator_slimming_design.md` を作成し、Option B (WASM Connector Plugins) の採用と詳細設計をドキュメント化。`ARCHITECTURE.md` に反映完了。

#### AWU 302: Define `llm-connector.wit` Interface
- **Objective:** OrchestratorとLLM Connector間でやり取りする標準メッセージ形式・イベント形式を定義した `wit` ファイルを作成する。
- **Scope:** `wit/llm-connector.wit` の作成・定義。
- **Definition of Done (DoD):** `llm-connector.wit` が定義され、各クレートから参照可能な状態であること。
- **Result:** `wit/llm-connector.wit` を作成し、標準メッセージ (`message`)、ツール (`tool`)、およびストリーミングイベント (`llm-event`, `event-stream` リソース) の定義を完了。

#### AWU 303: Update Core Host to support LLM Connector Extensions
- **Objective:** ホスト（Core）が `llm-connector` タイプのWasm拡張をロードし、そのエクスポート関数を認識できるようにする。
- **Scope:** `src/wasm.rs`, `rad.json`
- **Definition of Done (DoD):** `rad.json` に `llm-connector` 役割の拡張を定義でき、Core起動時に正しくインスタンス化されること。
- **Result:** `src/wasm.rs` および `bindings.rs` を拡張し、`llm-connector` 役割のコンポーネントをサポートしました。`WasmState` への Host トレイト実装や `open_http_stream` の委譲を完了し、Clippy の警告なしでコンパイルおよび既存のテストを通過させました。

#### AWU 304: Implement Core Wasm-to-Wasm Event Routing
- **Objective:** OrchestratorからConnectorへの `generate-stream` 呼び出しおよび逆方向のイベントストリームをCore経由で仲介・ルーティングする機構を実装する。
- **Scope:** `src/wasm/rpc.rs`, `src/wasm/imports.rs`
- **Definition of Done (DoD):** Orchestratorからのリクエストが指定されたコネクタにルーティングされ、結果を受け取れること。
- **Result:** `wit/rad.wit` および `rad-models` に `GenerateLlmStream` / `LlmConnectorEvent` を追加。ホスト RPC 処理で `llm-connector` 拡張を検索し、型マッピングを行った上でコンポーネントを呼び出し、イベントをバックグラウンドスレッド経由で Orchestrator にブロードキャストするイベントルーティング機構を実装しました。

#### AWU 305: Create `openai-connector` Extension Boilerplate
- **Objective:** 新規エクステンション `ext/openai-connector` クレートを立ち上げ、`llm-connector.wit` のボイラープレートを生成する。
- **Scope:** `ext/openai-connector/`
- **Definition of Done (DoD):** `cargo build --target wasm32-wasip2` が通り、空のレスポンスを返すモジュールがビルドできること。
- **Result:** `ext/openai-connector` クレートを新規作成し、`llm-connector.wit` を用いてコンポーネントとしての土台を構築、`rad.json` に `llm-connector` ロールとして登録しました。

#### AWU 306: Migrate HTTP/SSE logic to `openai-connector`
- **Objective:** 現行の `openai-orchestrator` からOpenAI API向けのリクエスト生成と、受信したHTTP/SSEチャンクのパース（文字抽出・ツールコール抽出）ロジックをコネクタに移転する。
- **Scope:** `ext/openai-connector/src/`
- **Definition of Done (DoD):** コネクタ単体でCoreの `OpenHttpStream` を叩き、戻り値を標準 `LlmEvent` としてパースできること。
- **Result:** `openai-orchestrator` から OpenAI Chat Completion リクエストのシリアライズロジック、および SSE のデシリアライズ、`LlmEvent`（ContentChunk, ReasoningChunk, ToolCallChunk, CompletionComplete, Error）への変換ロジックをコネクタへ完全移転し、安全な実装が正常にコンパイルできることを確認しました。

#### AWU 307: Refactor `openai-orchestrator` to call the Connector
- **Objective:** `openai-orchestrator` から生HTTPリクエストやSSE解析ロジックを完全に削除し、新設したConnectorを呼び出すシンプルな対話管理ロジックに書き換える。
- **Scope:** `ext/openai-orchestrator/src/`
- **Definition of Done (DoD):** `openai-orchestrator` がコンパイル可能で、内部にOpenAI固有のHTTP/SSE/JSONパーサーを持たないこと。
- **Result:** `openai-orchestrator` から生HTTPリクエスト送信やSSEパーサーを完全に削除し、ホスト経由で `openai-connector` からブロードキャストされる `LlmConnectorEvent` を受け取って状態管理を行うようにリファクタリングを完了しました。

#### AWU 308: Integration Testing & Verification
- **Objective:** 全体を結合し、REPL上でのエージェント対話ループ、ツール実行、エラーリカバリの動作確認テストを行う。
- **Scope:** プロジェクト全体、統合テスト実行
- **Definition of Done (DoD):** すべての既存ユニットテストおよび統合テスト（`cargo test -- --test-threads=1`）がパスすること。
- **Result:** 全てのエクステンションをリリースビルドし、統合テスト `cargo test -- --test-threads=1` が全て正常に通過（33個のユニットテスト、および全統合・E2Eテストがパス）することを確認しました。

---
*Note: This file is the single source of truth for the project status.*
