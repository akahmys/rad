# rad 設定・エクステンション管理ガイド

## 1. 設定ファイル (`rad.config.json`)

`rad` の動作要件および各エクステンションの権限制限は、プロジェクトルートの `rad.config.json` で一元管理されます。

```json
{
  "version": "1.0",
  "core": {
    "workspace_dir": ".",
    "snapshot_dir": ".rad/snapshots",
    "log_dir": ".rad/logs"
  },
  "default_timeout": {
    "llm_stream_heartbeat_ms": 15000,
    "process_silent_timeout_ms": 60000
  },
  "extensions": [
    {
      "name": "standard-orchestrator",
      "source": "builtin://standard-orchestrator",
      "enabled": true,
      "permissions": {
        "fs_read_allow": ["*"],
        "fs_write_allow": ["*"],
        "allow_bash": true,
        "allow_network": true
      }
    },
    {
      "name": "thirdparty-analyzer",
      "source": "./ext/analyzer.wasm",
      "enabled": true,
      "permissions": {
        "fs_read_allow": ["src"],
        "fs_write_allow": [],
        "allow_bash": false,
        "allow_network": false
      }
    }
  ]
}
```

---

## 2. エクステンションの管理

`rad` はシステムパッケージマネージャーやRoot権限に依存せず、ユーザー空間のみでクリーンに管理可能です。

*   **追加・更新**: 該当する `.wasm` ファイルやスクリプトを特定のパス（例: `.rad/ext/`）に配置し、`rad.config.json` の `extensions` 配下に要求権限を追記するだけで導入が完了します。
*   **本体の更新**: 単一バイナリを直接差し替えるか、`rad --update` コマンド（セルフアップデート機能）を実行することで、インプレースで最新安定版へ更新されます。
