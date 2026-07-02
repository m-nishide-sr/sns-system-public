# Rust コーディング規約

## 1. 基本方針

本プロジェクトでは、Rust の型安全性・所有権モデル・テスト容易性を最大限に活用し、保守性・可読性・信頼性の高いコードを実装することを目的とする。

実装にあたっては、以下を重視する。

* 小さく明確な責務を持つモジュール設計
* 実装内容に基づいた適切なファイル分割
* クリーンアーキテクチャに基づく依存方向の制御
* 単体テスト・結合テスト・プロパティテストによる品質保証
* `rustfmt` / `cargo clippy` による機械的な品質維持

***

## 2. ファイル分割・モジュール構成

### 2.1 実装に基づく適切なファイル分割

1ファイルに過度な責務を持たせず、実装内容・責務・変更理由に基づいてファイルを分割すること。

避けるべき例:

* `main.rs` / `lib.rs` に大量の処理を直接記述する
* 1つの `service.rs` に複数ユースケースを詰め込む
* DBアクセス、業務ロジック、HTTPハンドラを同一ファイルに混在させる

推奨する分割例:

```text
src/
  main.rs
  lib.rs
  domain/
    mod.rs
    user.rs
    message.rs
  application/
    mod.rs
    use_cases/
      create_message.rs
      get_messages.rs
  infrastructure/
    mod.rs
    dynamodb/
      message_repository.rs
  interface/
    mod.rs
    http/
      handlers.rs
      routes.rs
```

### 2.2 モジュールの責務

各モジュールは、次のように責務を明確にする。

* `domain`
  * エンティティ
  * 値オブジェクト
  * ドメインルール
  * ドメイン固有のエラー
* `application`
  * ユースケース
  * トランザクション単位の処理
  * Repository trait の利用
* `infrastructure`
  * DB、外部API、ファイル、クラウドサービスなどの具体実装
* `interface`
  * HTTP、GraphQL、CLI、Lambda handler などの入出力境界

***

## 3. クリーンアーキテクチャ

### 3.1 依存方向

依存方向は必ず内側に向けること。

```text
interface
   ↓
application
   ↓
domain

infrastructure
   ↓
application / domain
```

`domain` は、`infrastructure` や `interface` に依存してはならない。

### 3.2 trait による依存性逆転

DBアクセスや外部サービス呼び出しは、直接ユースケース内に書かず、trait として抽象化すること。

```rust
pub trait MessageRepository {
    fn save(&self, message: Message) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: MessageId) -> Result<Option<Message>, RepositoryError>;
}
```

ユースケースは trait に依存し、具体実装は `infrastructure` 側で提供する。

これにより、テスト時にはモックやインメモリ実装に差し替え可能とし、テスト可能な構造を維持する。

***

## 4. テスト方針

### 4.1 単体テスト

ドメインロジック、値オブジェクト、ユースケースは単体テストを作成すること。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_valid_message() {
        let message = Message::new("hello".to_string());

        assert!(message.is_ok());
    }
}
```

### 4.2 結合テスト

外部I/O、DB、HTTP layer、Lambda handler など複数モジュールをまたぐ処理については、`tests/` ディレクトリ配下に結合テストを配置する。

```text
tests/
  create_message_test.rs
  get_messages_test.rs
```

### 4.3 proptest によるプロパティテスト

境界値、ランダム入力、組み合わせ入力に対して成立すべき性質がある処理については、`proptest` によるプロパティテストを実施すること。

`proptest` は任意入力に対して性質が成立するかを検査し、失敗時には再現しやすい最小ケースへ縮小する property-based testing framework である。 [\[github.com\]](https://github.com/proptest-rs/proptest)

例:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn message_length_is_never_negative(input in ".*") {
        let message = Message::new(input);

        if let Ok(message) = message {
            prop_assert!(message.len() >= 0);
        }
    }
}
```

プロパティテストの対象例:

* パーサー
* バリデーション
* 状態遷移
* ID生成
* ソート・フィルタ処理
* 金額・数量・日時計算
* encode / decode
* serialize / deserialize

***

## 5. フォーマット規約

### 5.1 rustfmt の使用

作業終了前には必ず `cargo fmt` を実行し、Rustコードを整形すること。

```bash
cargo fmt
```

`cargo fmt` は現在のクレートの bin / lib ファイルを `rustfmt` でフォーマットする Cargo サブコマンドである。 [\[doc.rust-lang.org\]](https://doc.rust-lang.org/cargo/commands/cargo-fmt.html)

CIでは以下のコマンドにより、フォーマット漏れを検出する。

```bash
cargo fmt -- --check
```

### 5.2 手動フォーマットの禁止

原則として、手動で独自スタイルに整えることを禁止する。

フォーマットに関する議論は避け、`rustfmt` の結果を正とする。

***

## 6. Lint 規約

### 6.1 cargo clippy の使用

作業終了前には必ず `cargo clippy` を実行し、警告・エラーが出ない状態にすること。

```bash
cargo clippy
```

Clippy は Rust コードの一般的な誤りや改善点を検出する lint 集であり、Cargo サブコマンドとして `cargo clippy` で実行できる。 [\[doc.rust-lang.org\]](https://doc.rust-lang.org/stable/clippy/usage.html), [\[doc.rust-lang.org\]](https://doc.rust-lang.org/clippy/)

CIでは警告も失敗扱いとするため、以下を実行する。

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

`-D warnings` を指定すると、Clippy や rustc の警告がビルド失敗扱いになる。 [\[doc.rust-lang.org\]](https://doc.rust-lang.org/stable/clippy/usage.html)

### 6.2 allow の使用ルール

`#[allow(...)]` は原則として多用しない。

使用する場合は、理由をコメントで明記すること。

```rust
// 外部API仕様により、この命名を維持する必要があるため許可する。
#[allow(non_snake_case)]
pub struct ExternalApiResponse {
    pub userId: String,
}
```

***

## 7. エラーハンドリング

### 7.1 unwrap / expect の制限

本番コードでは、原則として `unwrap()` を使用しない。

使用可能な例:

* テストコード
* 明らかに失敗しない初期化処理
* 失敗時に即時停止すべき起動時検証

ただし、`expect()` を使用する場合は、失敗理由が分かるメッセージを記述すること。

```rust
let config = Config::load().expect("設定ファイルの読み込みに失敗しました");
```

### 7.2 Result による明示的な失敗表現

失敗し得る処理は `Result<T, E>` で表現し、呼び出し元で適切に処理すること。

***

## 8. 命名規約

Rust の標準的な命名規則に従う。

| 対象    |                     規約 | 例                    |
| ----- | ---------------------: | -------------------- |
| 変数    |            snake\_case | `user_name`          |
| 関数    |            snake\_case | `create_message`     |
| モジュール |            snake\_case | `message_repository` |
| 型     |         UpperCamelCase | `MessageId`          |
| trait |         UpperCamelCase | `MessageRepository`  |
| 定数    | SCREAMING\_SNAKE\_CASE | `MAX_MESSAGE_LENGTH` |

***

## 9. コメント・ドキュメント

### 9.1 コメント方針

コメントは「何をしているか」ではなく、「なぜそうしているか」を説明するために記述する。

避ける例:

```rust
// ユーザーIDを取得する
let user_id = user.id();
```

望ましい例:

```rust
// 外部システムでは退会済みユーザーも参照されるため、ここでは存在チェックのみ行う。
let user_id = user.id();
```

### 9.2 public API のドキュメント

公開する構造体、関数、trait には必要に応じて doc comment を付与する。

```rust
/// メッセージを永続化するためのRepository。
pub trait MessageRepository {
    /// メッセージを保存する。
    fn save(&self, message: Message) -> Result<(), RepositoryError>;
}
```

***

## 10. 作業完了条件

実装作業は、以下をすべて満たした時点で完了とする。

```bash
cargo fmt
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --all-features -- --include-ignored
```

加えて、対象機能に性質ベースで検証すべきロジックがある場合は、`proptest` によるプロパティテストを追加・実行すること。

***

## 11. Pull Request 前チェックリスト

Pull Request 作成前に、以下を確認する。

* [ ] 実装内容に基づいて適切にファイル分割されている
* [ ] 1つのモジュール・関数に過剰な責務がない
* [ ] クリーンアーキテクチャの依存方向を破っていない
* [ ] DB・外部APIなどの依存は trait で抽象化されている
* [ ] ユースケースが単体テスト可能な構造になっている
* [ ] 単体テストを追加・更新した
* [ ] 必要に応じて結合テストを追加・更新した
* [ ] 必要に応じて `proptest` によるプロパティテストを追加した
* [ ] `cargo fmt` を実行した
* [ ] `cargo fmt -- --check` が成功する
* [ ] `cargo clippy --all-targets --all-features -- -D warnings` が成功する
* [ ] `cargo test` が成功する
* [ ] `unwrap()` / `expect()` の使用理由が妥当である
* [ ] 不要な `#[allow(...)]` が残っていない
* [ ] コメントは「なぜ」を説明している
* [ ] public API に必要なドキュメントコメントがある

***

## 12. 推奨 CI コマンド

CIでは最低限、以下を実行する。

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features -- --include-ignored
```
