--liquibase formatted sql

--changeset liquibase:0001_create_messages_table runOnChange:false
--comment: チャットメッセージテーブルの作成
CREATE TABLE messages (
    id          UUID        NOT NULL PRIMARY KEY,
    cognito_id  UUID        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    body        TEXT        NOT NULL,
    is_from_user BOOLEAN    NOT NULL
);

COMMENT ON TABLE  messages              IS 'チャットメッセージ';
COMMENT ON COLUMN messages.id           IS 'メッセージID（UUID v7）';
COMMENT ON COLUMN messages.cognito_id   IS 'CognitoユーザーのサブジェクトID';
COMMENT ON COLUMN messages.created_at   IS 'メッセージ作成日時';
COMMENT ON COLUMN messages.body         IS 'メッセージ本文';
COMMENT ON COLUMN messages.is_from_user IS 'ユーザーからのメッセージかどうか（true: ユーザー, false: システム）';

--changeset liquibase:0001_create_idx_messages_user_timeline_local context:local runOnChange:false
--comment: ユーザータイムライン取得用インデックス（ローカルPostgreSQL向け）
CREATE INDEX idx_messages_user_timeline
    ON messages (cognito_id, created_at DESC)
    INCLUDE (body, is_from_user);

--changeset liquibase:0001_create_idx_messages_user_timeline_develop context:develop runOnChange:false
--comment: ユーザータイムライン取得用インデックス（Aurora DSQL develop環境向け）
CREATE INDEX ASYNC idx_messages_user_timeline
    ON messages (cognito_id, created_at DESC)
    INCLUDE (body, is_from_user);

--changeset liquibase:0001_create_idx_messages_user_timeline_release context:release runOnChange:false
--comment: ユーザータイムライン取得用インデックス（Aurora DSQL release環境向け）
CREATE INDEX ASYNC idx_messages_user_timeline
    ON messages (cognito_id, created_at DESC)
    INCLUDE (body, is_from_user);

--changeset liquibase:0001_grant_lambda_local context:local runOnChange:false
--comment: lambdaロールへの権限付与（ローカルPostgreSQL向け）
GRANT SELECT, INSERT ON messages TO postgres;

--changeset liquibase:0001_grant_lambda_develop context:develop runOnChange:false
--comment: lambdaロールへの権限付与（Aurora DSQL develop環境向け）
AWS IAM GRANT SELECT, INSERT ON messages TO 'arn:aws:iam::${AWSAccountId}:role/sns-db-develop-lambda-role';

--changeset liquibase:0001_grant_lambda_release context:release runOnChange:false
--comment: lambdaロールへの権限付与（Aurora DSQL release環境向け）
AWS IAM GRANT SELECT, INSERT ON messages TO 'arn:aws:iam::${AWSAccountId}:role/sns-db-release-lambda-role';
