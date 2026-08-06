# ARCHITECTURE-NEXT — 目標アーキテクチャ

`ARCHITECTURE.md` は**現在の実装の仕様**である。本書は**移行先の設計**であり、実装は存在しない。
両者が一致した時点で本書は `ARCHITECTURE.md` に統合され、削除される。

**到達点は現行コードベースの構造に拘束されない** — 本書 §2〜§8 は、いまの実装が
どうなっているかとは無関係に、あるべき形を書いている。

**一方、そこへの行き方は現行コードベースの中を通る**(§9)。カーネルは別に作って
入れ替えるのではなく、既存のCoreに新しいWITパッケージを1つ足したうえで、拡張を
1つずつモジュールへ移し、最後に旧い面を削り落とした結果として現れる。
移行中のどの時点でも rad は動作する。

---

## 1. 目的

**radを、第三者が拡張するプラットフォームにする。**

この一文が他のすべての判断を決めている。拡張性を目的としない設計であれば、
本書の大半は過剰である。

### 1.1 現行構造がその目的に届かない理由

Core と 6つのWasm拡張が、WITで定義された型付き契約を共有している。この構造には
実験で確認した欠陥がある(§2)。要点は、**拡張を1つ足すたび、コマンドを1つ足すたびに、
無関係な拡張がすべて壊れる**こと。バージョンチェックも存在せず、版ずれは起動時の
`Warning` を素通りして最初のタスク実行で初めて失敗する。

自分で全拡張を持っている間は `scripts/build_all.sh` 一発で済むため実害は小さい。
しかし第三者が拡張を書く前提では、**radがリリースするたびに全サードパーティ拡張が壊れる**。
これは受け入れられない。

### 1.2 到達点

**カーネルは wasm ランタイムとディスパッチャに徹し、機能はすべてその上のモジュールとする。**
`rad` = Rust Agent Dispatcher という名前が元々指していた構造である。

---

## 2. 設計の基礎 — 実測で確定した規則

本設計の中核は、2つの実験結果から導かれている。どちらも実際に再現し、完全に復旧済み。

| 変更内容 | 結果 |
|---|---|
| 既存関数の引数型にケースを1つ追加(`ras-rpc-command` に未使用ケース) | **全拡張が壊れる** |
| **新規の独立した関数を追加**(`experimental-probe`) | **既存拡張は無傷** ✅ |

破壊の原因は「型にケースが増えたこと」ではなく、**既存関数の型そのものが変わったこと**である。

```wit
import host-rpc: func(command: ras-rpc-command) -> result<string, string>;
//                             ^^^^^^^^^^^^^^^ ここが変わると host-rpc は別の関数になる
```

コンポーネントモデルは構造的型付けを行うため、`ras-rpc-command` が変われば `host-rpc` は
別関数となり、古い型を要求する拡張はリンクできない。エラーは
`component imports function 'host-rpc', but a matching implementation was not found in the linker`。

現行radは**育ち続ける型を2つ**持っている — `ras-rpc-command`(26ケース)と
`ras-core-event`(12ケース)。どちらも既存関数の引数であり、増えるたびにすべてを壊す。

### 2.1 導かれる二層構造

| 層 | 形式 | 進化の規則 |
|---|---|---|
| **syscall** | 型付きWIT、個別関数 | **追加のみ。既存関数は絶対に変更しない** |
| **dispatch** | 不透明(`string` のみ) | 型が存在しないので進化しない |

syscall を典型的なWITのまま保てるのは、実験で「新規関数の追加は安全」が確認できたためである。
オプションが必要になったら既存関数を変えず、新しい関数を足す(例: `proc-spawn` に環境変数を
渡したくなったら `proc-spawn-env` を新設する)。これは制約ではなく規約とする。

### 2.2 「追加のみ」規則の正確な適用範囲

「追加が安全」なのは**トップレベルの新規関数**に限られる。以下は**すべて破壊的**である。

| 変更 | 影響 |
|---|---|
| 新しいトップレベル関数を追加 | ✅ 安全(実証済み) |
| 既存関数の引数・戻り値の型を変更 | ❌ その関数を使う全モジュールが壊れる |
| **共有 record / variant にフィールドやケースを追加** | ❌ それを使う**全関数**の型が変わる |
| **既存 resource にメソッドを追加** | ❌ その resource を返す/受け取る**全関数**の型が変わる |

3行目が `variant error` を `record error { code, message }` に変えた理由であり(§3.1)、
4行目は `resource process` に後からメソッドを足せないことを意味する。

**resource は最初に確定させ、以後凍結する。** 機能が必要になったら resource を
拡張せず、新しいトップレベル関数を足す。これは不便だが、破壊しないための唯一の道である。

§3.1 で syscall を3つに絞ったため、この規則が監視すべき面積は小さい。
resource も `process` と `byte-stream` の2つだけである。

---

## 3. カーネル

### 3.1 契約

共有WITは**このファイル1つだけ**である。

```wit
package rad:kernel@1.0.0;

interface types {
    // ★ variant にしてはならない。ケースを1つ足すと全syscallの戻り型が変わり、
    //   §2 で断罪した破壊が再現する。code は enum ではなく数値のまま扱う。
    record error { code: u32, message: string }

    // `stream` は WIT の予約語(コンポーネントモデルが async の `stream<T>` に
    // 使う)。実装時に判明した。周囲のエコシステムが足元で動いている具体例であり、
    // カーネルがこの型を自前で持つ理由(§3.1.1)の裏づけでもある。
    resource byte-stream { read: func(max: u32) -> result<list<u8>, error>;
                       write: func(data: list<u8>) -> result<_, error>;
                       close: func(); }
    resource process { stdout: func() -> byte-stream; stderr: func() -> byte-stream;
                       stdin: func() -> byte-stream;
                       wait: func() -> result<s32, error>; kill: func(); }
}

interface syscall {
    use types.{error, process, byte-stream};

    // WASI が既に提供するもの — ファイルシステム、クロック、標準入出力 — は
    // 再実装しない(§3.4.1)。モジュールは `std::fs` / `std::time` /
    // `println!` をそのまま使う。到達範囲はホストが preopen で決める。

    proc-spawn: func(argv: list<string>) -> result<process, error>;
    net-open:   func(url: string, headers: list<tuple<string, string>>,
                     body: list<u8>) -> result<byte-stream, error>;
    log:        func(trace-id: string, level: string, message: string);
}

interface dispatch {
    // 不透明。どちらも引数は string のみで、型は永久に変わらない。
    call: func(target: string, method: string, payload: string) -> result<string, string>;
    post: func(target: string, method: string, payload: string);
}

world module {
    import syscall;
    import dispatch;

    export manifest: func() -> string;                                  // JSON
    export handle:   func(method: string, payload: string) -> result<string, string>;
}
```

**syscall は3つしかない。** 当初案は `fs-read` / `fs-write` / `fs-open` / `fs-list` /
`term-write` / `term-read-line` / `clock-ms` を含む10個だったが、いずれも WASI の
再実装であり削除した(§3.4.1)。`proc-spawn` は WASI p2 にプロセス起動が存在しないため
必須、`log` はトレースID伝搬(§8のデバッグ性課題)のために残す。

これにより §2.2 の「追加のみ」規則が監視すべき面積は桁違いに小さくなる。
`resource file` も消え、残る resource は `process` と `byte-stream` のみである。

#### 3.1.1 `net-open` は `wasi:http` に置き換えない(決定済み)

契約上は代替可能である。`incoming-body.stream() -> input-stream` があるので、
SSEの逐次読み出しは `wasi:http` で表現できる。**それでも採用しない。**

**WASI 0.3.0(2026-06-11リリース)が `wasi:io` パッケージを丸ごと削除した。**
pollable / input-stream / output-stream はコンポーネントモデルの Canonical ABI に
吸収され、`wasi:http` は `stream<T>` / `future<T>` を使う形に再設計された。
ネットワークのケイパビリティ受け渡しも変わっている。対応には Wasmtime 43+ が要る
(radは現在29)。

つまり `wasi:http@0.2` を import したモジュールは、**0.3 への移行時に全部同時に壊れる**。
これは §2 で断罪した破壊そのものであり、しかも**発生源がプロジェクトの外側**にあるため
こちらでは制御できない。

**カーネルの役割は、まさにこの断絶を吸収することである。** rad が自前で
`net-open -> stream` を定義していれば、WASI の版が変わってもカーネル内部の実装を
差し替えるだけで済み、モジュールは一行も変わらない。

#### 3.1.2 導かれる規則 — WASI を使う条件

§3.4.1 で fs / clock / stdio の syscall を削除したのと矛盾しない。境界は**Rust の std
が挟まっているか**である。

| | 経路 | WASI 0.3 の影響 |
|---|---|---|
| ファイル・時刻・標準入出力 | `std::fs` / `std::time` / `println!` | **std が吸収する。** モジュールのソースは不変 |
| HTTP・プロセス起動 | std に相当APIがない → WITを直接 import することになる | **モジュールが直撃する** |

> **規則: Rust の std が抽象化してくれるものは WASI に任せる。
> std に無いものは rad の syscall として自前で定義し、カーネルが版差を吸収する。**

この規則により syscall は3つで確定する — `proc-spawn` / `net-open` / `log`。

### 3.2 world が1つになる

現行は4つのworld(`rad-extension` / `rad-orchestrator` / `rad-security-guard` /
`rad-tool-provider`)を持ち、役割ごとに別の契約を定義している。新しい役割を作るには
WITを増やす必要がある。

不透明ディスパッチではその必要がない。モジュールは**自分が何であるかを実行時に申告する**。

```json
{
  "name": "context",
  "version": "0.3.1",
  "abi": "1.0",
  "provides": ["context.optimize", "context.digest"]
}
```

`provides` はルーティング表の構築に使う。当初案にあった `requires`(必要な権限の申告)は
**削除した** — ケイパビリティ強制をやめた時点で読む者がいなくなり、飾りになったためである。

カーネルは `manifest()` を読み終えるまでモジュールに何もさせないため、
**`manifest()` は syscall も dispatch も行ってはならない**。

world は `module` ひとつ。**役割を追加してもWITは不変**であり、
誰でも任意の役割のモジュールを書ける。

### 3.3 イベントもディスパッチを通る

```
handle("event.llm-chunk", payload)
handle("event.tool-result", payload)
```

新しいイベント種別の追加が**WITに一切触れない**。現行の `ras-core-event` 問題が
構造的に消滅する。

### 3.4 信頼モデル

**カーネルはケイパビリティ機構を持たない。設定に載っているモジュールは全syscallを使える。**

これは手抜きではなく、検討の結果としての判断である。

#### 3.4.1 なぜ持たないのか

**決定的な事実: 現行radの権限マスクは、そもそも機能していない。**

`src/wasm/loader.rs` は全拡張に対し `.` と `$HOME` を
`DirPerms::all()` + `FilePerms::all()` で preopen している。したがって拡張は
`std::fs` を呼ぶだけで、`RasRpcCommand::FileWrite` も security-guard の `verify-rpc` も
`fs_write_allow` マスクも**一切通らずに**ホームディレクトリ全体を読み書きできる。

実証済み — `mcp-tool-provider` に3行の `std::fs::write` を仕込んで
`$HOME` へのファイル作成に成功した(実験は完全に復旧済み)。

**FSのRPC経路は「強制」ではなく「慣習」だった。** 拡張が行儀よく使っていただけである。
ケイパビリティ機構を捨てるという判断は、**守っていないものを捨てるだけ**である。

この事実は2つの帰結を持つ。

**帰結1: syscall層の大半が WASI の重複である。** マスクを強制しないなら、
ファイルシステム・クロック・標準入出力を再実装する理由がない。WASI が提供するものを
そのまま使い、到達範囲はホストが preopen で決める。syscall は10個から3個になった(§3.1)。
これは **§5.3(拡張を書く敷居を下げる)にも直接効く** — 拡張作者は rad 固有の syscall を
覚えず、普通の Rust (`std::fs` / `std::time` / `println!`) を書けばよい。

**帰結2: 選別が守れる範囲はもともと狭い。** `proc-spawn` を持てば実質フルユーザー権限であり、
`mcp-bridge` はそれを必要とする。精緻な機構を積んでも `mcp-bridge` 経由で迂回できる。
さらに**MCPサーバのプロセス権限はradが制約できない**(§3.4.4)。

**隔離そのものはwasmから無料で得られる。** メモリ隔離も、カーネルが実装していない操作を
行えないことも、wasmtimeが自動的に保証する。ケイパビリティ機構が追加で提供するのは
「そのsyscallを配るか否か」の選別だけであり、上記の通りその価値は小さい。

**脅威の実体は悪意あるモジュールではない。** 現実的な脅威はプロンプトインジェクション —
LLMが信用できない入力(Webページ、ファイル内容、ツール出力)を読んで行動を決めること。
この経路は `agent-loop` → `mcp-bridge` であり、**どちらもファーストパーティで協力的**である。
一方、悪意あるモジュールは**利用者がインストールしたもの**であり、
「インストールした = 信頼した」という二値の判断は設定ファイルに書いてある時点で
表現済みである。ACLを重ねても判断の質は上がらない。

**拡張を書く敷居と衝突する。** `requires` 宣言、設定の許可、メソッド単位の粒度、
呼び出し元ごとのツールリスト — これらは拡張作者が理解し記述しなければならない機構であり、
§5.3 の目的と正面から衝突する。

参考: pi-coding-agent のTypeScript拡張はプロセス内でNodeの全権限を持つ。
radのwasmモジュールは、ケイパビリティ機構ゼロでも**piより隔離されている**。

#### 3.4.2 強制が欲しい人のための拡張点

カーネルには専用のフックを設けない。**モジュール構成そのものが拡張点だからである。**

強制が必要なら、`mcp-bridge` を自分のゲート付き実装に差し替えればよい。
ルーティングは `manifest().provides` で決まるため、設定を1行変えるだけで置き換わる。

当初案は `syscall-gate` ロールを設ける形だったが、**破棄した**。
syscall を監視しても (a) ファイルシステムは WASI で迂回され、
(b) 危険の本体である「起動済みMCPサーバへのツール呼び出し」は syscall として現れないため、
何も守れない。§3.4.1 で捨てた機構と同じ欠陥を、小さくして持ち込むだけだった。

#### 3.4.3 `policy` は協力的なモジュール

人間への承認確認は**プロンプトインジェクションに対する実効的な防御**である。
悪意あるモジュールを封じ込める機構ではなく、**LLMの暴走を人が止める**ためのもの。

`mcp-bridge` がツール実行前に `policy` へ問い合わせる、という**1箇所のフック**で足りる。
skills の `allowed_tools`(§4.5.2)もカーネル機能ではなく `policy` の仕事として載る。

強制力はない。協力的なモジュールにしか効かない。それでよい — 守る対象がLLMの判断であり、
モジュールの悪意ではないためである。

#### 3.4.4 明記すべき限界

> **ツール呼び出しがMCPサーバに届いた後、radはそのプロセスの権限を制約できない。**
> サーバは独立したOSプロセスとして利用者の全権限で動作する。
> 実効的な防御は「どのMCPサーバを登録するか」という利用者の判断であり、radの外にある。

これは新設計の欠陥ではなくMCPを採用することの帰結である。
現行 `ARCHITECTURE.md` §1.3 はSecurity Guardがプロンプトインジェクション被害を防ぐと
述べているが、これは過大な主張である。**防げているふりをしないこと。**

#### 3.4.5 帰結

権限の非対称が存在しないため、**confused deputy 問題は発生しない**
(モジュールAがBを踏み台にしても、AとBの権限は同一である)。
あわせて `manifest().requires` は削除した(§3.2)。

### 3.5 バージョニング

- WITパッケージに版を持たせる — `rad:kernel@1.0.0`
- モジュールは `manifest()` で対応ABIを申告する
- カーネルはロード時に照合し、**不一致なら明確なエラーで停止する**
  (現行の「Warningを素通りして最初のタスクで失敗」を修正する)。
  ただし効果を過大に見ないこと — 不透明ディスパッチではWITが変わらないため、
  プロトコル進化によるリンク失敗はそもそも起きない。この検査が効くのは
  「新しいモジュールが、古いカーネルに存在しない syscall を要求した」場合の
  **エラーメッセージが分かりやすくなる**ことであって、安全性ではない
- payloadスキーマは加算的に進化させる(serde の `#[serde(default)]`)。
  破壊的変更が要る場合はメソッド名を変える(`context.optimize` → `context.optimize2`)

型検査はリンク時から呼び出し時へ移る。これは弱体化ではない。リンク時検査は
**全か無か**であり、1つの変更が全モジュールを落とす。呼び出し時検証は粒度が細かく、
「そのメソッドは非対応」と応答して処理を継続できる。

### 3.6 実行モデル

**カーネルが唯一のスケジューラである。どのモジュールも `main` を所有しない。**

#### 3.6.1 モジュールごとに独立Store + async wasmtime

`Config::async_support(true)` を有効にする(wasmtime 29.0.1 に存在することを確認済み。
現行radは同期のみで未使用)。ブロックするsyscall — stream読み、プロセス待ち、
`term-read-line` — は**呼び出したモジュールだけをサスペンド**し、OSスレッドを解放する。

- モジュールは直線的なコードを書ける(すべてを状態機械に潰さなくてよい)
- OSスレッドは1本で足りる。モジュールごとにスレッドを立てない
- Storeが独立なので、1モジュールのクラッシュが他へ波及しない

#### 3.6.2 `call` と `post`

**イベントはすべて `post` で送る。これがデッドロックを構造的に防ぐ。**

`agent-loop` が `llm-transport.generate` を `call` すると agent-loop はサスペンドする。
そこで transport がチャンクを `call` で返そうとすると、agent-loop は自分の呼び出しの
戻りを待っているため**再入デッドロック**になる。`post` ならチャンクはキューに入り、
agent-loop の処理が返ってから配送される。

**規則: 長時間かかる操作は必ず `post` + イベント返しにする。**

#### 3.6.3 呼び出し循環はハングではなくエラー

カーネルは `call` のスタックを追跡し、既にスタック上にあるモジュールへの `call` を拒否する。

```
Err("dispatch cycle: agent-loop -> llm-transport -> agent-loop")
```

プラットフォームである以上、**第三者のモジュールがradを凍結できてはならない**。

#### 3.6.4 ストリーミングの流れ

```
1. ui-repl        term-read-line                    (asyncでサスペンド。他は動く)
2. ui-repl        post → agent-loop     "turn.start"
3. agent-loop     call → context        "optimize"   (同期。結果が要る)
4. agent-loop     post → llm-transport  "generate"   ★ post であること
5. llm-transport  net-open → stream読みループ        (自分だけサスペンド)
                  チャンクごとに post → agent-loop "event.llm-chunk"
6. agent-loop     post → ui-repl        "render.chunk"
7. llm-transport  post → agent-loop     "event.llm-done"
8. agent-loop     post → mcp-bridge     "tools.call"
```

4を `call` にすると agent-loop が生成完了まで固まり、中断も効かなくなる。

#### 3.6.5 中断とハング防止

`Config::epoch_interruption(true)` を有効にし、カーネルがepochを進めることで
**暴走したモジュールをプリエンプトする**。

現行の `src/esc_abort.rs` はフラグ方式で、モジュールが協力しなければ止まらない。
第三者モジュールが無限ループし得るプラットフォームでは、強制的に止められる必要がある。
(`consume_fuel` も利用可能だが epoch の方が安価。)

#### 3.6.6 モジュールクラッシュ時の自己修復

wasmtimeのtrapをカーネルが捕捉 → 当該Storeを破棄 → 再ロード →
`post("lifecycle.rehydrate")`。現行の自己修復を**カーネルの一般機能に格上げする**。
Storeが独立なため他モジュールは無傷である。

#### 3.6.7 ブートストラップ

```
1. カーネル起動、設定読み込み
2. 各モジュールをロードし manifest() を呼ぶ → ルーティング表を構築
3. 全モジュールに post("lifecycle.start")
4. カーネルのスケジューラループへ  ← これが main
```

`ui-repl` は他モジュールと同格であり、`term-read-line` にサスペンドしているだけである。

設定は既存の `extensions` とは**別の配列**にする。移行中は両方が同時に存在するため
(§9.1)、混ぜると「これは旧拡張か新モジュールか」を毎回判定することになる。

```json
"modules": [
  { "name": "context", "source": "~/.rad/wasm/context.wasm", "enabled": true,
    "config": { } }
]
```

`#[serde(default)]` なので既存の設定ファイルは無変更で動く。旧 `extensions` にあった
`role` と `permissions` は持たない — 役割は `manifest().provides` が申告し(§3.2)、
権限機構は存在しない(§3.4)。

`config` はカーネルが保持し、モジュールは `dispatch.call("kernel", "kernel.config", …)`
で取得する。**カーネル自身がディスパッチ先の一つとして登録される**ため、モジュールから
見れば他のモジュールと区別する必要がない。

#### 3.6.8 ルーティングの衝突

2つのモジュールが同じメソッドを `provides` に申告した場合、**起動時エラーとする**。
設定で優先順位を明示した場合のみ解決する。暗黙の先勝ちにはしない — 
どちらが有効かが起動順に依存するのは、プラットフォームとして受け入れられない。

#### 3.6.9 未検証のコスト

- **モジュールごとのStore分のメモリ。** 現行の最大は `mcp_tool_provider.wasm` の21MBだが、
  これはコンパイル済みファイルサイズであり、インスタンスのlinear memoryとは別物である。
  **実測が必要な未知数として残る**
- async wasmtime は同期実行より若干のオーバーヘッドがある
- 直線コードが書ける利点はあるが、`post` を受ける側(`agent-loop`)は結局状態機械になる。
  ここは現行と変わらない

### 3.7 カーネルの規模

残るのは wasmtimeランタイム、syscall実装、ルータ、スケジューラ、設定ブートストラップ。

現行Coreは9,508行。うち置き換え対象は `src/wasm/` 2,966行 + syscall相当
(`fs` 585 + `process` 453 + `http` 380)= 約4,384行。ディスパッチ化で
`rpc_meta_*` 系が縮む一方、スケジューラと循環検出が新たに要る。

**現実的な目標は3,500〜5,000行**とする。当初見積もった2,000〜3,000行は楽観的である。

カーネルはツールもプロンプトもLLMもエージェントも知らない。

---

## 4. モジュール

### 4.1 一覧

| モジュール | 責務 |
|---|---|
| `agent-loop` | 推論・ツールループ。サーキットブレーカー、L3バックオフ、再水和 |
| `llm-transport-openai` | dialect表 + `/v1/chat/completions`(クラウド・汎用) |
| `llm-transport-raw` | minijinja + `/completion`(ローカル・高機能) |
| `context` | 圧縮。windowing / stale tool-result clearing / relevance retention / digest |
| `policy` | 承認・拒否ポリシー(協力的。強制力は持たない — §3.4.3) |
| `dag` | 履歴DAG + スナップショット |
| `skills` | `SKILL.md` の発見と提供(§4.5) |
| `mcp-bridge` | 外部MCPサーバの起動・管理・橋渡し |
| `ui-repl` | 端末UI |

### 4.2 LLMトランスポートが2つある理由

**これはモジュール構成が具体的に報われる最初の事例である。** 単一のコネクタしか
持てない現行構造では、以下は「どちらかを捨てる」選択になる。

**`llm-transport-openai`** — プロバイダ差分を `const` 構造体で保持する。設定ファイル化はしない。

```rust
pub struct Dialect {
    pub path:           &'static str,   // {model} 展開可
    pub auth_header:    &'static str,
    pub auth_format:    &'static str,   // "Bearer {key}"
    pub extra_headers:  &'static [(&'static str, &'static str)],
    pub content_ptr:    &'static str,   // JSON Pointer
    pub reasoning_ptr:  Option<&'static str>,
    pub tool_calls_ptr: &'static str,
}

pub const OPENAI: Dialect = Dialect {
    path: "/v1/chat/completions", auth_header: "Authorization",
    auth_format: "Bearer {key}", extra_headers: &[],
    content_ptr:    "/choices/0/delta/content",
    reasoning_ptr:  Some("/choices/0/delta/reasoning_content"),
    tool_calls_ptr: "/choices/0/delta/tool_calls",
};

pub const GEMINI: Dialect = Dialect { path: "/v1beta/openai/chat/completions", ..OPENAI };
pub const AZURE:  Dialect = Dialect {
    path: "/openai/deployments/{model}/chat/completions?api-version=2024-10-21",
    auth_header: "api-key", auth_format: "{key}", ..OPENAI };
```

構造体更新構文 `..OPENAI` が const で成立することは確認済み。差分だけ書けるため
serde の default より明示的で、かつコンパイル時に検査される。

**`llm-transport-raw`** — チャットテンプレートを自前で描画する。

`/v1/chat/completions` ではテンプレート適用がサーバ側のブラックボックスになる。
生の `/completion` に降りることで得られるもの:

- **assistant prefill** — 応答の冒頭を強制できる(`<think>` の注入など)。
  chat/completions では原理的に不可能で、小型モデルのツール呼び出し信頼性に効く
- **ツール呼び出し形式の完全な制御** — 学習時の形式を正確に再現できる
- **thinking タグの決定論的な扱い** — `reasoning_content` は DeepSeek/llama.cpp の
  慣習に過ぎず、標準ではない
- GBNF/grammar制約との相性、および送信文字列の可視性

**モデル毎のコードは不要である。** GGUFは `chat_template` を内包し、llama.cppは
`/props` で公開している(radは既にコンテキスト長検出で `/props` を叩いている)。
取得したテンプレートを minijinja で描画すればよい。

**実機で検証済み。** 稼働中の llama.cpp の `/props` は `chat_template` を返し
(実測 16,934文字)、その実テンプレートが正しく描画されることを確認した。
出力にはモデル固有のツール宣言形式と `<|channel>thought` ブロックが含まれる。
wasm32-wasip2 でのビルドも確認済み(rlib 188KB)。

**依存指定には注意が要る。** 実テンプレートは Jinja の高度な機能を使う。

```toml
minijinja         = { version = "2", features = ["loop_controls", "json"] }
minijinja-contrib = { version = "2", features = ["pycompat"] }
```

```rust
env.set_unknown_method_callback(minijinja_contrib::pycompat::unknown_method_callback);
```

- `default-features = false` にすると `{% macro %}` が使えず、**テンプレート1行目で
  パースに失敗する**(実テンプレートはマクロを多用する)
- `minijinja-contrib` の `pycompat` が無いと `message.get('reasoning')` で失敗する。
  Pythonのdictメソッドであり、**llama.cpp が独自の `minja` を書いた理由がこれである**
- `namespace()` は本体が対応済み

制約: **クラウドAPIは生completionを公開していない。** このモジュールはローカル専用である。

### 4.3 minijinja の適用範囲

| 対象 | 機構 | 理由 |
|---|---|---|
| チャットテンプレート | **minijinja** | モデル自身が Jinja2 で持っている |
| システムプロンプト | **minijinja** | 利用者が再ビルドなしに構造を差し替えられる |
| リクエストボディ | serde 構造体 | **JSONをテキスト展開で組むのはエスケープ事故の温床** |
| レスポンス解析 | JSON Pointer | 宣言的で十分。現行でも `.pointer()` を使用中 |

### 4.4 `mcp-bridge` がモジュールである理由

カーネルにMCPクライアントを持たせない。`proc-spawn` と `stream` があればモジュールとして
書ける。カーネルのトランスポートは wasm 1本に保たれ、それでいて外部MCPサーバは
そのまま利用できる。

MCPサーバのコストは**コンテキスト消費のみ**である。サーバは起動時に一度spawnされ
stdio上で常駐するため呼び出し毎のプロセス起動はなく、JSON-RPC往復は実測0.06〜0.08ms。
実体は構造化された入出力を持つCLIに過ぎない。

**ただしコンテキスト消費は実在する。** 実測: core-utilities 15ツール 7,362文字 +
web-access 4ツール 1,530文字 = **8,892文字**。システムプロンプト(約200文字)の約44倍。

| context長 | 予算 | スキーマの占有 |
|---|---|---|
| 65536 | 232,244 文字 | 3.8% |
| 32768 | 114,280 文字 | 7.8% |
| **8192** | 25,808 文字 | **34.5%** |

**現行はツールスキーマを予算計算に入れていない**(`messages_json` と `tools_json` が
別経路で、圧縮はメッセージのみを見る)。新設計では `agent-loop` が予算に算入する。

### 4.5 `skills`

`.agents/skills/<name>/SKILL.md`(プロジェクトローカル)と
`~/.rad/skills/<name>/SKILL.md`(ユーザーグローバル)を発見し、モデルに提供する。
現行の `skill-tool-provider` を移植するが、3点変更する。

**① `mode: subagent` を削除する**

サブエージェントを持たないと決めたため(§6)、予約値ごと消す。現行は
「not yet implemented」を返す仕様で `CONFIG.md` §2.5 にも記載があるため、
**利用者に見える変更**である。移行時にドキュメントを更新すること。

**② `echo` ハックが不要になる**

現行 `execute_tool` の戻り値はWITの `execution-handle` 型であるため、
スキル本文を返すのに `open_process(echo ...)` を経由している。その副作用で、
ファイルを読むだけの拡張に**bash実行権限が必要**になっていた。

不透明ディスパッチでは `handle()` が文字列を直接返すので、ハックも余計な権限も消える。
設計変更が既存の問題を1つ解消する事例である。

**③ ツール表現を「1ツール + 索引」に変える**

現行は**スキル1つ = ツール1つ**として公開している。ツールスキーマは実測平均468文字
(§4.4)であり、スキルが増えるほどコンテキストを線形に消費する。

```
skill(name, args?)          ← ツールスキーマは1つ
  description: "利用可能なスキル:
     review-pr  — PRをレビューする
     deploy     — ステージングへデプロイする
     ..."
```

スキル10個で概算 4,700文字 → 1,200文字程度。**本文が呼び出し時にのみ読まれる**という
段階的開示は現行のまま維持される。

トレードオフ: 個別ツールの方がモデルにとって発見しやすく引数に型が付く。
ただしClaude Code自身が単一 `Skill` ツール方式を採っており、実用上は成立している。

#### 4.5.1 CLIツールの説明層としての役割

pi-coding-agent は "Build CLI tools with READMEs (**see Skills**)" と述べており、
**スキルをCLIツールの説明層として位置づけている**。

radでは bash が `core-utilities-mcp` の `execute_command` から来るため、
SKILL.md に「`mytool --foo` を実行せよ」と書けばそのまま成立する。
**MCPサーバを1つ書く代わりに、CLIツール + SKILL.md で済ませる選択肢**が、
追加の設計なしに存在する。§7 で述べるMCPのコンテキスト消費を避けたい場合の逃げ道になる。

#### 4.5.2 `allowed_tools`

`allowed_tools` は現行ではパースされるのみで強制されていない。
新設計では **`policy` モジュールの仕事**として実装する(§3.4.3)。

`mcp-bridge` はツール実行前に `policy` へ問い合わせる。`policy` は
「いまスキル X の実行中である」という文脈を持ち、許可されていないツールを拒否する。
`skills` モジュール側に強制機構を持たせない。

これはカーネルの機能ではなく、**協力的なモジュール間の取り決め**である。
強制力がないことは意図的であり、守る対象がLLMの判断であってモジュールの悪意ではない
ためである(§3.4.3)。

---

## 5. ワークスペース構成

```
rad/
├── Cargo.toml                  # 仮想マニフェスト(ルートにパッケージを置かない)
├── wit/
│   └── kernel.wit              # 唯一の共有契約
│
├── crates/
│   ├── rad-kernel/             # bin: wasmtime + syscall + router + bootstrap
│   ├── rad-abi/                # ディスパッチ封筒 / manifestスキーマ / 版定数
│   │                           #   native と wasm32-wasip2 の両方でビルドされる
│   └── rad-sdk/                # ゲスト側SDK(モジュール作者が依存する唯一のcrate)
│
├── modules/                    # ファーストパーティのモジュール
│   ├── agent-loop/  llm-transport-openai/  llm-transport-raw/
│   ├── context/     policy/    dag/
│   └── mcp-bridge/  ui-repl/
│
├── templates/
│   └── module-rust/            # ★ワークスペースメンバー(後述)
│
└── xtask/                      # build/test/package の統括(build_all.sh を置換)
```

```toml
[workspace]
resolver = "3"
members  = ["crates/*", "modules/*", "templates/module-rust", "xtask"]

[workspace.package]
edition      = "2024"
rust-version = "1.85"

[workspace.dependencies]
rad-abi    = { path = "crates/rad-abi" }
rad-sdk    = { path = "crates/rad-sdk" }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 5.1 テンプレートのドリフトが構造的に不可能になる

現行の `templates/rust` と `templates/go` は `wit/rad.wit` の**コピー**を持つため、
3回ドリフトした(3回目を防ぐゲートを `build_all.sh` に追加済み)。

新構成ではテンプレートを**ワークスペースメンバーとし、共通の `wit/kernel.wit` を参照させる**。
`cargo build --workspace` が必ずコンパイルするため、**検出する問題ではなく発生し得ない問題**になる。

### 5.2 MCPサーバはワークスペースに入れない

`core-utilities-mcp` と `web-access-mcp` は**別リポジトリのまま維持する**。

cargoはワークスペースメンバーがルート配下にあることを要求するため
(`workspace member ... is not hierarchically below the workspace root`、実測で確認)、
「別リポジトリのまま単一ワークスペース」は成立しない。選択肢はモノレポ化か分離の二択である。

**分離を選ぶ理由:**

- **共有する型がない。** radとMCPサーバは別プロセスでJSON-RPCでしか通信せず、
  コンパイル時に一度もリンクされない。依存バージョンがずれても無害であり、
  ワークスペースが解決する問題(依存統一・単一 `Cargo.lock`)がこの組み合わせには存在しない。
  MCPスキーマも公開crate `rust-mcp-schema` を介するため私的な共有crateも不要
- **設計の主張と衝突する。** §7 は「任意のMCPサーバが動くこと」を強みとしている。
  rad自身のサーバをradのリポジトリに置くと、それらが特権的な一部に見える
- **`core-utilities-mcp` は単独で価値がある。** Claude Code、Cursor、Zed でそのまま使える。
  radのリポジトリに埋めると、それ自身の採用を損なう
- リリース周期を独立に保てる

**利便性は `xtask` で回収する。** `xtask` はRustプログラムなので、兄弟ディレクトリに
チェックアウトされたリポジトリへ `cargo build` を発行できる。

```
cargo xtask build --with-servers
cargo xtask test  --with-servers
```

パスは設定または環境変数で与え、**見つからなければ黙ってスキップする**。
radだけを触る利用者に影響を与えないこと。

横断的な変更が2コミットに分かれる点は残るが(例: `isError` 伝播をrad側、
`edit_file` の内容ベース化をサーバ側で行った変更)、これは別リリースサイクルを
持つことの正当な帰結であり、解消すべき不都合とは扱わない。

### 5.3 `rad-sdk` — 拡張を書く敷居を下げる

pi-coding-agent の拡張はTypeScriptのプロセス内モジュールで、書く敷居が極めて低い。
radがプラットフォームを名乗るなら、ここが勝負どころになる。

`rad-sdk` が生のWITバインディングを包み、`manifest()` を生成し、payloadのserde変換を
隠す。モジュール作者が書くのはこれだけになる。

```rust
rad_sdk::module! {
    name:    "context",
    version: "0.3.1",
    methods: {
        "context.optimize" => optimize,
        "context.digest"   => digest,
    }
}

fn optimize(req: OptimizeReq) -> Result<OptimizeRes, rad_sdk::Error> { ... }
fn digest(req: DigestReq) -> Result<DigestRes, rad_sdk::Error> { ... }
```

マクロが生成するもの:

- `manifest()` — `name` / `version` / `abi` と、`methods` のキーから導いた `provides`。
  **手で書いた `provides` と実装がずれる余地をなくす**
- `handle(method, payload)` — メソッド名で分岐し、payloadを `serde_json` で
  デシリアライズして関数に渡し、戻り値をシリアライズして返す。未知のメソッドは
  「そのメソッドは非対応」として返す(§3.5 の「粒度が細かく継続できる」検証)
- `wit_bindgen` の `export!` 配線

**宣言的マクロ (`macro_rules!`) で実装する。** 手続きマクロにすれば
`#[rad::module]` 属性形式にできて見た目は良くなるが、proc-macroクレートと `syn` 依存が
増える。上の形で目的は足りているので、属性形式は必要になってから検討する
(CODING.md §3「投機的な実装をしない」)。

#### 5.3.1 `rad-abi` — 共有するのは manifest だけ

初版は `rad-abi` を「ディスパッチ封筒 / manifestスキーマ / 版定数」としていたが、
**不透明ディスパッチではカーネルは payload を一切解釈しない**ため、封筒は共有物ではない。

実際に両側が合意する必要があるのは **`manifest()` のスキーマひとつ**である。ゲストが
生成し、カーネルがルーティング表を組むために読む。ここがずれると起動時に静かに壊れる。

したがって `rad-abi` は manifest 型だけの小さなクレートとして始める。
`serde` のみに依存し、native と wasm32-wasip2 の両方でビルドされる。

ファーストパーティのモジュール同士が共有する payload 型(`OptimizeReq` など)は、
**実際に2つ目の利用者が現れてから** `rad-abi` に置く。最初から置くのは、
まだ存在しない共有のための投機である。

---

## 6. 意図的に持たないもの

- **サブエージェント** — pi-coding-agent も意図的に持たない(必要なら tmux で複数インスタンス)。
  最小主義を突き詰めた先人が複雑さに見合わないと判断した領域である。
  同時に走るエージェントは常に1つとし、状態は単一に保つ
- **plan mode**
- **カーネル側のツール** — capability ツール(ファイル・シェル・ネットワーク)を
  カーネルは一切提供しない

---

## 7. pi-coding-agent との関係

radは pi-coding-agent(Mario Zechner, MIT, `earendil-works/pi`)の最小主義に影響を受けている。
先人との相違を意識的な選択として記録する。

| | pi | rad(本設計) |
|---|---|---|
| 組み込みツール | 4つ(read/write/edit/bash) | 0(すべてモジュール/MCP) |
| MCP | **非対応(明示的に拒否)** | `mcp-bridge` 経由で対応 |
| サブエージェント | 持たない | 持たない(追随) |
| 拡張 | TypeScript、プロセス内 | wasmモジュール、言語自由 |
| プロバイダ | 15+、OAuth含む | dialect表 + `/completion`。**OAuthは未対応** |
| システムプロンプト | ~150語 | 1文(約30語) |

**piがMCPを拒否した論拠は「READMEを持つCLIツールをbashで叩けば足りる」である。**
radがMCPを採る論拠は**構造化された結果**にある。`isError` による成否判定は
連続失敗サーキットブレーカーの前提であり、bashの文字列出力からは同じ判定ができない。

なお「拡張を書くコストが高い」ことはMCPの欠点にならない。MCPエコシステムには
2万規模のサーバが存在し(mcp.so 20,222 / PulseMCP 15,930+ / Smithery 7,000+ /
公式レジストリ約2,000)、**大半の場合は書く必要がない**。MCPを拒否したpiはこれを
消費できない。

---

## 8. 未解決の論点

正直に記録する。

- **サブスクリプション認証 (OAuth)** — Claude Pro/Max、ChatGPT Plus/Pro、GitHub Copilot。
  dialect表では届かない(トークンの取得・保存・更新とブラウザフローが必要)。
  カーネルが解決済みトークンを渡す形にすれば `llm-transport-*` は無改造で済む。
  **実装前に各社の利用規約を確認すること。** piが対応していることは許諾の根拠にならない。
  GitHub Copilot はデバイスフローが公式にドキュメント化されており最も素直
- **デバッグ性** — wasm境界をまたぐスタックトレースが繋がらない。モジュール数に比例して
  悪化する。`log` syscall にトレースIDを持たせ、伝搬を最初から設計に入れる
- **`dag` モジュールの信頼性** — クラッシュ復旧の土台がモジュールでよいか。
  状態がディスクにあるため再ロードで回復可能だが、意図的な判断として記録しておく
- **型安全性の実効** — ファーストパーティ間は `rad-abi` の共有型でコンパイル時検査を保てる。
  サードパーティとの境界のみ実行時検証になる。この線引きが実運用で十分かは未検証
- **モジュールごとのStoreのメモリ実測** — §3.6.9 参照
- **WASI 0.3 への追随時期** — 0.3 は `wasi:io` を削除し Wasmtime 43+ を要求する
  (radは29)。§3.1.2 の規則により**モジュールは影響を受けない**が、カーネル内部の
  syscall実装と `std::fs` 等が乗る WASI 版は更新が必要になる。急がないが、
  カーネルを書く際に「WASIの版差はカーネルが吸収する」前提を実装に織り込むこと
- **信頼モデルの妥当性** — §3.4 でケイパビリティ機構を持たないと決めた。
  この判断は「利用者がインストールしたモジュールは信頼する」という前提に依存している。
  第三者モジュールの流通が現実に始まった時点で、強制機構の要否を再評価すること
  (現時点の拡張点は「`mcp-bridge` を差し替える」であり、カーネルのフックではない — §3.4.2)。
  **判断の前提が変わったら判断も見直す**

---

## 9. 実現への道筋

### 9.1 原則 — その場で、1つずつ

初版は「新規ディレクトリで並行して作り、段階5で入れ替える」としていた。**これは誤りである。**
並行案の唯一の論拠は「現行radを壊さない」ことだったが、壊さない方法が他にあるため成立しない。

**根拠は §2 の実験2そのものである。** 新規の独立した関数を足しても既存拡張は無傷だった。
これは新しい**パッケージ**を足す場合も同じで、実際 Core は既に3つのWITパッケージから
6つの world を同時にホストしている。

```
wit/rad.wit                     → rad-extension / rad-orchestrator /
                                   rad-security-guard / rad-tool-provider
wit/connector/llm-connector.wit → llm-connector
wit/context-tools.wit           → context-tools-extension
```

`wit/kernel.wit` を7つ目として足すのは、この繰り返しにすぎない。**`wit/rad.wit` に
一切触れないため、既存6拡張は壊れない。** したがって新旧を同一プロセス内で共存させられる。

> **カーネルは「作って入れ替える」ものではなく、旧い皮を1枚ずつ剥いだ結果として現れる。**

移行中、Coreは旧RPC面と新dispatch面の両方を抱えるため、縮む前に一時的に膨らむ。
これは限定的で可視的なコストであり、2つのワークスペース・2系統のCI・2つの設定を
恒久的に維持するより軽い。

### 9.2 並行案と比べて

| | 並行ディレクトリ(破棄) | その場で段階的(採用) |
|---|---|---|
| 移行中 rad は動くか | 段階5まで新側は動かない | **全段階で動く。毎日使える** |
| ワークスペース | 2つ | 1つ |
| CI | 2系統 | 既存の `--workspace` が全部カバー |
| 設定ファイル | 共有できない | 1つのまま |
| 入れ替え | 大きな一発勝負 | **存在しない** |
| 各段階の価値 | 段階5まで出荷不能 | **1つ移すごとに完結** |

### 9.3 段階

| # | 内容 | 完了時点の状態 |
|---|---|---|
| 0 | **dialect構造体**(`ext/llm-connector` 内で完結、共有WITに触れない) | ✅ 完了 (AWU 948) |
| 1 | `wit/kernel/kernel.wit` を**新パッケージとして追加**、`rad-abi` と `rad-sdk` を作る | 6拡張は無変更で動作(AWU 949 で実証済み) |
| 2 | Core に dispatch / router / スケジューラを実装(§3.6)。旧RPC面と共存 | 同上。新面はまだ誰も使わない |
| 3 | `context-tools` → module へ移す(既存ロジックが最も素直に載る) | 5拡張 + 1モジュール |
| 4 | `skill-tool-provider` → module(§4.5 の3変更を同時に反映) | 4拡張 + 2モジュール |
| 5 | `mcp-tool-provider` → `mcp-bridge` module | 3拡張 + 3モジュール |
| 6 | `llm-connector` → `llm-transport-openai` module | 2拡張 + 4モジュール |
| 7 | `security-guard` → `policy` module | 1拡張 + 5モジュール |
| 8 | `rad-orchestrator` → `agent-loop` module | **旧world・旧RPC面・`models/` の変換マクロを削除** |
| 9 | `dag` / `ui-repl` を Core から module へ切り出す | 残ったCoreがカーネル |
| 10 | `llm-transport-raw`(minijinja)、`templates/module-rust`、`cargo xtask new-module` | 拡張作者向けが揃う |

**段階3〜8は毎回同じ形の作業である** — 1つの拡張を module world に移し、
旧worldへの登録を外し、動作を確認する。順序は依存の少ない順に並べてある。

`dag` と `ui-repl` が最後(段階9)なのは、現在Coreがこれらを所有しているためである。
端末とDAGの所有権を手放すのは他のすべてが移り終わってからでよい。

### 9.4 各段階の不変条件

- **`wit/rad.wit` は移行の最後まで変更しない。** 変更すれば残っている拡張が全部壊れる(§2)
- **各段階の終わりに rad は動作する。** 動かない状態で次に進まない
- CI(`--workspace` + wasm32-wasip2)が全段階を通して緑であること
