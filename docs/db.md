# SNSシステム DBサブシステム

## 概要

これは、SNSシステムのDB開発用サブシステムです。

## 構成と主なファイルの各説明

- / ： リポジトリルート
  - /.devcontainer/api/devcontainer.json ： API開発で利用するdevcontainer
  - /.github/workflows/sns-system-db-cicd.yaml ： このプロジェクトのDBのCI/CDのGitHub Actions定義
  - /README.md ： システム全体の概要説明
  - /AGENTS.md ： AI向けプロンプトを記述する。上記README.mdを参照することを明記。
  - /api ： APIのルート
  - /db ： DBのルート
    - /db/AGENTS.md ： AI向けプロンプトを記述
    - /db/template.yaml ： DBのIaC
    - /db/liquibase ： Liquibaseによるマイグレーションを管理するディレクトリ
      - /db/liquibase/changelog.xml ： Liquibaseの設定。設定内容として`<includeAll path="changes" relativeToChangelogFile="true"/>`とし、`changes`ディレクトリを辞書順で参照する。
      - /db/liquibase/changes/*.sql ： マイグレーション用SQL。ファイル名は先頭を`0001_`から辞書順で並ぶように作成する。
    - /db/sea_orm_entities ： DB定義用パッケージ
      - /db/sea_orm_entities/Cargo.toml ： パッケージのマニフェストファイル。
      - /db/sea_orm_entities/src/lib.rs ： `sea-orm-cli`により自動生成される。ドキュメンテーションコメントと`pub mod entity;`だけ書かれている。
      - /db/sea_orm_entities/src/entity/*.rs ： ORM定義(自動生成)。`sea-orm-cli generate entity`で出力されたものを格納する。
  - /docs ： ドキュメントのルート
    - /docs/db.md ： 人間とAI向けにdbの詳細の説明を記述
  - /frontend ： フロントエンドのルート
  - /review ： レビュー資料デプロイのルート

## 実装方針

* 主キーは`UUID`型を使用し、プログラム側でuuidv7を生成する。
* 日時項目は`timestamptz`型を使用する。
* ユニークインデックスはAurora DSQLではパフォーマンスが落ちるため、通常のインデックスを主に利用する。
* INDEXを作成する場合にはINCLUDEで必要な項目のみ参照する。これはDynamoDBがGSIに値を持っていない場合にベーステーブルへのルックアップに伴うN+1問題が同様にAurora DSQLでも発生してしまうため、その回避を目的とする。
* `COMMENT ON`で詳細なコメントを記載する。
* VIEWについては`MATERIALIZED VIEW`は利用できないため、`CREATE VIEW`文で永続ビューを作成する。

## DB定義

### Liquibaseで使用するProperty

- `${Stage}` ： `local` or `develop` or `release`
- `${AWS_ACCOUNT_ID}` ： AWSアカウントID

### ロール

#### `lambda`

- `lambda` ： `arn:aws:iam::<AWSアカウントID>:role/sns-db-${Stage}-lambda-role`
  - local: `CREATE ROLE lambda WITH LOGIN PASSWORD 'lambda';`
  - develop/release: 
    - `CREATE ROLE lambda WITH LOGIN;`
    - `AWS IAM GRANT lambda TO 'arn:aws:iam::${AWS_ACCOUNT_ID}:role/sns-db-${Stage}-lambda-role';`

### テーブル

#### `public.messages`

チャットメッセージを保持するテーブル。

| カラム名 | 型 | NULL | DEFAULT | 説明 |
|--|--|--|--|--|
| id | uuid | NOT NULL | gen_random_uuid() | 主キー。デフォルトでUUIDv4を採番する |
| user_name | text | NOT NULL | '' | 投稿ユーザーのemailアドレスの@以前の文字列 |
| cognito_id | uuid | NOT NULL | '00000000-0000-0000-0000-000000000000' | 投稿ユーザーのCognitoサブジェクトID |
| created_at | timestamptz | NOT NULL | CURRENT_TIMESTAMP | メッセージ作成日時 |
| body | text | NOT NULL | '' | メッセージ本文 |
| row_log | text | NOT NULL | なし(必須) | 生ログ。不具合などの調査時にのみ参照される想定 |
| is_from_user | boolean | NOT NULL | なし(必須) | `true`: ユーザー投稿, `false`: システム投稿 |

主キー：

- `pk_messages (id)`

インデックス：

- `idx_messages_user_timeline (created_at) INCLUDE (user_name, body, is_from_user)`
  - local: `CREATE INDEX idx_messages_user_timeline ON messages (created_at) INCLUDE (user_name, body, is_from_user)`
  - develop/release: `CREATE INDEX ASYNC idx_messages_user_timeline ON messages (created_at) INCLUDE (user_name, body, is_from_user)`

権限：

- lambda
  - 可 ： INSERT
  - 不可 ： SELECT・UPDATE・DELETE
  - `GRANT INSERT ON public.messages TO lambda;`

### ビュー

#### `public.messages_latest`

`public.messages`をそのまま参照する互換ビュー。

| カラム名 | 型 | NULL | 説明 |
|--|--|--|--|
| user_name | text | NOT NULL | 投稿ユーザーのemailアドレスの@以前の文字列 |
| created_at | timestamptz | NOT NULL | メッセージ作成日時。デフォルトは`CURRENT_TIMESTAMP` |
| body | text | NOT NULL | メッセージ本文 |
| is_from_user | boolean | NOT NULL | `true`: ユーザー投稿, `false`: システム投稿 |

権限：

- lambda
  - 可 ： SELECT
  - 不可 ： INSERT・UPDATE・DELETE
  - `GRANT SELECT ON public.messages_latest TO lambda;`

### DBのマイグレーション

DBのマイグレーションにはLiquibaseを利用する。

* LiquibaseのAurora DSQLのサポートが完全でないため、SQLファイルによりマイグレーションを管理する。
* `/db/liquibase/changelog.xml`にてLiquibaseの設定を記述する。設定内容として`<includeAll path="changes" relativeToChangelogFile="true"/>`とする。
* `/db/liquibase/changes/*.sql`にマイグレーションのSQLファイルを、`001_*.sql`から辞書順になるようにファイル名を命名し作成する。
* INDEXは、ローカルのPostgreSQL向けの`CREATE INDEX`と、Aurora DSQL向けの`CREATE INDEX ASYNC`を作成し、Liquibase側ではcontextの値で分岐させる。contextは`local`,`develop`,`release`の3つの値とする。
* ROLEは、Aurora DSQLでは`AWS IAM GRANT *** TO 'arn:aws:iam::<AWSアカウントID>:role/sns-db-${Stage}-lambda-role';`と記載する必要がある。AWSロールの物理名は`/db/template.yaml`に記載しOutputsにExportする。これらの各値は`liquibase update`実行時にコマンドライン引数で指定する。

### DBのORM定義

Rustで型安全にDBを参照するため`sea_orm`を利用する。

- DBのORM定義パッケージのマニフェストファイル ： `/db/sea_orm_entities/Cargo.toml`

APIのプロジェクトから参照されることを意識して作成する。
`[dependencies]`には`sea-orm = { version = "1", features = ["sqlx-postgres", "runtime-tokio-rustls", "macros"] }`のみ追加する。

### DBのORMのCI/CD

- CI/CDの最後に`sea-orm-cli`でentityを自動生成し、pushする。
Liquibaseの管理用の`databasechangelog`テーブルと`databasechangeloglock`テーブルを除外するため、以下のようなコマンドを実行する。
```bash
sea-orm-cli generate entity -u "$DATABASE_URL" -o infrastructure/sea_orm/src/entity --ignore-tables databasechangelog --ignore-tables databasechangeloglock
```

### Aurora DSQLの料金例

#### Amazon Aurora DSQL

- USD 10.00 per 1M DPU Units
- USD 0.40 per GB-month

#### DPUについて(実行計画の例)

##### INSERT例

純粋にINSERTをした場合のDPU消費量 ： 約0.03626DPU (=約0.0000003626USD/INSERT処理) (=約0.000058987768円/INSERT処理)

```sql
EXPLAIN ANALYZE VERBOSE INSERT INTO messages (cognito_id, body, is_from_user)
VALUES (
    '12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d', -- 外部から渡されたcognito_id
    'test',
    true                                     -- 利用者からの送信
);

Insert on public.messages  (cost=0.00..0.01 rows=0 width=0) (actual time=0.066..0.067 rows=0 loops=1)
  ->  Result  (cost=0.00..0.01 rows=1 width=73) (actual time=0.019..0.020 rows=1 loops=1)
        Output: gen_random_uuid(), '12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d'::uuid, CURRENT_TIMESTAMP, 'test'::text, true
Query Identifier: dgrwqzcrwg49h
Planning Time: 0.022 ms
Execution Time: 0.656 ms
Statement DPU Estimate:
  Compute: 0.00083 DPU
  Read: 0.00047 DPU (Transaction minimum: 0.00375)
  Write: 0.01748 DPU (Transaction minimum: 0.05000)
  Multi-Region Write: 0.01748 DPU (Transaction minimum: 0.05000)
  Total: 0.03626 DPU
```

##### SELECT例

主キーを条件として、INDEXにINCLUDEされているカラムだけをSELECTした場合のDPU消費量 ： 約0.00182DPU (=約0.0000000182USD/SELECT処理) (=約0.000002960776円/SELECT処理)

```sql
EXPLAIN ANALYZE VERBOSE SELECT cognito_id, created_at, body, is_from_user FROM public.messages  WHERE cognito_id = '12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d' ORDER BY created_at DESC LIMIT 100;

Limit  (cost=100.12..104.13 rows=1 width=66) (actual time=0.694..0.733 rows=2 loops=1)
  Output: cognito_id, created_at, body, is_from_user
  ->  Index Only Scan Backward using idx_messages_user_timeline on public.messages  (cost=100.12..104.13 rows=1 width=66) (actual time=0.693..0.732 rows=2 loops=1)
        Output: cognito_id, created_at, body, is_from_user
        Index Cond: (messages.cognito_id = '12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d'::uuid)
        -> Storage Scan on idx_messages_user_timeline  (cost=100.12..104.13 rows=1 width=66 loops=1) (actual rows=2 loops=1)
            Projections: cognito_id, created_at, body, is_from_user
            Limit: 100
            -> B-Tree Scan on idx_messages_user_timeline  (cost=100.12..104.13 rows=1 width=66 loops=1) (actual rows=2 loops=1)
                Index Cond: (messages.cognito_id = '12345678-abcd-7a8b-9c0d-1e2f3a4b5c6d'::uuid)
Query Identifier: si9m3uq6f74a1
Planning Time: 0.110 ms
Execution Time: 0.770 ms
Statement DPU Estimate:
  Compute: 0.00109 DPU
  Read: 0.00073 DPU (Transaction minimum: 0.00375)
  Write: 0.00000 DPU
  Total: 0.00182 DPU
```

## DB インフラ構成

- /db/template.yaml
  - Parameters
    - Stage ： デプロイステージ
      - Type: String
      - AllowedValues:
        - develop
        - release
    - SubSystem ： サブシステム分類名(ここでは`db`のみ)
      - Type: String
      - Default: db
      - AllowedValues:
        - db
  - Resources
    - Aurora DSQL ： `Type: AWS::DSQL::Cluster`
      - `DeletionPolicy` ： `$Stage`が`release`なら`Retain`、`develop`なら`Delete`
    - Role ： `Type: AWS::IAM::Role`
      - 上記のAurora DSQLリソースへの`dsql:DbConnect`権限
      - Role名 ： `sns-db-${Stage}-lambda-role`

## CI/CD

- jobs
  - validate
    - `sam validate --lint`で`/api/template.yaml`を検証する。
    - `cargo check`でコンパイルチェック
    - `cargo test`で軽量テスト実行
    - `cargo clippy`で静的解析
    - テスト実行用環境構築
      - PostgreSQL v16の立ち上げ
        ```bash
        docker run --rm --name testdb-postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -p 5432:5432 -d postgres:16
        until docker exec testdb-postgres pg_isready -U postgres -d postgres; do
          sleep 1
        done
        ```
      - Liquibase v4.33.0の立ち上げ・マイグレーションの実施
        ```bash
        docker run --rm -e LIQUIBASE_HUB_MODE=off --network host -v "${{ github.workspace }}:/workspace" liquibase/liquibase:4.33.0 --search-path=/workspace --changelog-file=db/liquibase/changelog.xml --contexts=local --url=jdbc:postgresql://localhost:5432/postgres --username=postgres --password=postgres --add-define=Stage=local --add-define=AWS_ACCOUNT_ID=123456789012 update
        ```
    - `cargo test --all-features -- --include-ignored`で全数テスト実行
  - migrate
    - AWSのクレデンシャルの設定
    - `sam deploy`でデプロイ
    - Liquibaseでupdateの実施
      ```bash
      STAGE=${{ github.ref == 'refs/heads/release' && 'release' || 'develop' }}
      DSQL_ENDPOINT=$(aws cloudformation describe-stacks --stack-name <スタック名> --query "Stacks[0].Outputs[?ExportName=='<スタック名>-DSQLEndpoint'].OutputValue" --output text)
      echo "::add-mask::$DSQL_ENDPOINT"
      TOKEN=$(aws dsql generate-db-connect-admin-auth-token --hostname $DSQL_ENDPOINT --region ap-northeast-3)
      echo "::add-mask::$TOKEN"
      AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query "Account" --output text)
      echo "::add-mask::$AWS_ACCOUNT_ID"
      docker run --rm -e LIQUIBASE_HUB_MODE=off -v "${{ github.workspace }}:/workspace" liquibase/liquibase:4.33.0 --search-path=/workspace --changelog-file=db/liquibase/changelog.xml --contexts=$STAGE --url="jdbc:postgresql://$DSQL_ENDPOINT/postgres" --username="admin" --password="$TOKEN" --add-define=Stage=${{ github.ref_name }} --add-define=AWS_ACCOUNT_ID=$AWS_ACCOUNT_ID update
      ```
  - generate_orm
    - AWSのクレデンシャルの設定
    - `sea-orm-cli generate entity`を実施しormエンティティを生成
      ```bash
      TOKEN=$(aws dsql generate-db-connect-admin-auth-token --region "${{ env.AWS_REGION }}" --expires-in 3600 --hostname ${{ env.DSQL_ENDPOINT}})
      echo "::add-mask::$TOKEN"
      ENCODED_TOKEN=$(printf '%s' "$TOKEN" | jq -rR @uri)
      DATABASE_URL="postgres://admin:${ENCODED_TOKEN}@${{ env.DSQL_ENDPOINT}}:5432/postgres?sslmode=require"
      sea-orm-cli generate entity -u "$DATABASE_URL" -o db/sea_orm_entities/src/entity --ignore-tables databasechangelog --ignore-tables databasechangeloglock
      ```
    - `gh pr create`によりプルリクエストを作成

## AWS Aurora DSQLについて

ローカルの開発環境ではPostgreSQL、develop環境およびrelease環境はAmazon Aurora DSQLをDBに利用する。

Aurora DSQLはほぼPostgreSQL互換だが、以下に注意。

* 外部キー（FOREIGN KEY）が使えない: 参照整合性の担保はアプリケーション層で実装するか、非正規化して持たせる必要あり。 
* ON DELETE CASCADE が使えない: 依存データの削除は、アプリ側で複数クエリを発行するかソフトデリート（論理削除）で対応。
* SERIAL / BIGSERIAL が使えない: 自動採番は `AS IDENTITY`もしくはシーケンスオブジェクトを利用。
* JSONB / JSON 型の直接定義が不可: カラムとしては TEXT 型で保存し、クエリ実行時に JSON/JSONB へキャストして処理。
* PL/pgSQL・トリガー・ストアドファンクションが使えない: DB側のロジック（手続き型処理）は、すべてアプリケーション層か AWS Lambda 等へ追い出す必要あり。
* 拡張機能（Extensions）はサポート外: PostGIS、pgvector などの一般的な拡張モジュールは利用不可。
* 一時テーブル（CREATE TEMP TABLE）が使えない: 複雑な中間データ処理は、共通テーブル式（CTE）やサブクエリで代替。
* DDL と DML の混在不可: 1つのトランザクション内でテーブル定義変更（DDL）とデータ操作（DML）は同時実行不可。また、1つのトランザクションに含められる DDL は1文のみ。
* 分離レベルは Repeatable Read 固定。
* ロック動作の変更（楽観的同時実行制御 / OCC）: SELECT ... FOR UPDATE などの構文は使用可能だが行はロックされず、競合はコミット時に検出され、シリアライズエラーとなるため、アプリ側に「エラー時のリトライ処理」の実装が必須。
* ユニークインデックスを使用するとパフォーマンスが大幅に低下するため、原則使用しない。
* `GENERATED ALWAYS AS IDENTITY (CACHE 1)`はパフォーマンスが低下するため、大量のINSERTが発生するテーブルでは使用しない。
* `CREATE SEQUENCE ~~~ CACHE 1`はパフォーマンスが低下するため、同時に何回も参照される場合は使用しない。

## 固定設定

### AWS SAMで使用する設定

| 設定名 | 設定値 |
|--|--|
| ステージ名 | `develop` or `release` |
| サブシステム名 | `api` or `auth` or `db` or `frontend` or `review` |
| スタック名 | sns-${SubSystem}-${Stage} |
| Lambdaの実行用AWSロール名 | sns-db-${Stage}-lambda-role |

### AWS SAMのOutputsでExportする値

| 概要 | Export名 | Value |
|--|--|
| DSQLのエンドポイント | sns-${SubSystem}-${Stage}-DSQLEndpoint | !GetAtt <`Type: AWS::DSQL::Cluster`のリソース>.Endpoint |
| DSQLへのアクセス権限Role | sns-${SubSystem}-${Stage}-LambdaRoleArn | !GetAtt <`Type: AWS::IAM::Role`のリソース>.Arn |

### GitHub Actionsで使用する設定

| 設定名 | 設定値 |
|--|--|
| AWS_DEPLOY_ROLE_ARN | GitHub Actionsで`aws-actions/configure-aws-credentials@v6`の`role-to-assume`に指定するARN |
| SAM_DEPLOY_ROLE_ARN | `sam deploy --role-arn`で指定するCloudFormation実行ARN |
| secrets.ALLOW_DOMAIN | ユーザー登録できるメールアドレスのドメイン部("@"は含まない) |
