--liquibase formatted sql

-- ロール作成 (local)
--changeset db:0001-01 context:local
CREATE ROLE lambda WITH LOGIN PASSWORD 'lambda';

-- ロール作成 (develop/release)
--changeset db:0001-02 context:develop,release
CREATE ROLE lambda WITH LOGIN;

-- AWS IAM GRANTによるロールへのIAMロール紐付け (develop/release)
--changeset db:0001-03 context:develop,release
AWS IAM GRANT lambda TO 'arn:aws:iam::${AWS_ACCOUNT_ID}:role/sns-db-${Stage}-lambda-role';

-- messagesテーブルの作成
--changeset db:0001-04
CREATE TABLE public.messages (
    id          uuid        NOT NULL DEFAULT gen_random_uuid(),
    user_name   text        NOT NULL DEFAULT '',
    cognito_id  uuid        NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    created_at  timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    body        text        NOT NULL DEFAULT '',
    row_log     text        NOT NULL,
    is_from_user boolean    NOT NULL,
    CONSTRAINT pk_messages PRIMARY KEY (id)
);
COMMENT ON TABLE  public.messages              IS 'チャットメッセージを保持するテーブル';
COMMENT ON COLUMN public.messages.id           IS '主キー。デフォルトでUUIDv4を採番する';
COMMENT ON COLUMN public.messages.user_name    IS '投稿ユーザーのemailアドレスの@以前の文字列';
COMMENT ON COLUMN public.messages.cognito_id   IS '投稿ユーザーのCognitoサブジェクトID';
COMMENT ON COLUMN public.messages.created_at   IS 'メッセージ作成日時';
COMMENT ON COLUMN public.messages.body         IS 'メッセージ本文';
COMMENT ON COLUMN public.messages.row_log      IS '生ログ。不具合などの調査時にのみ参照される想定';
COMMENT ON COLUMN public.messages.is_from_user IS 'true: ユーザー投稿, false: システム投稿';

-- messagesテーブルへのINSERT権限付与
--changeset db:0001-05
GRANT INSERT ON public.messages TO lambda;

-- messagesテーブルのインデックス作成 (local)
--changeset db:0001-06 context:local
CREATE INDEX idx_messages_user_timeline ON public.messages (created_at) INCLUDE (user_name, body, is_from_user);

-- messagesテーブルのインデックス作成 (develop/release)
--changeset db:0001-07 context:develop,release
CREATE INDEX ASYNC idx_messages_user_timeline ON public.messages (created_at) INCLUDE (user_name, body, is_from_user);

-- messages_latestビューの作成
--changeset db:0001-08
CREATE VIEW public.messages_latest AS
    SELECT user_name, created_at, body, is_from_user
    FROM public.messages;
COMMENT ON VIEW public.messages_latest IS 'public.messagesをそのまま参照する互換ビュー';

-- messages_latestビューへのSELECT権限付与
--changeset db:0001-09
GRANT SELECT ON public.messages_latest TO lambda;
