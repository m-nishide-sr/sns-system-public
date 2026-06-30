--liquibase formatted sql

--changeset sns-system-db:0001-create-messages-table
CREATE TABLE IF NOT EXISTS public.messages (
    id uuid NOT NULL,
    cognito_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    body text NOT NULL,
    is_from_user boolean NOT NULL,
    CONSTRAINT pk_messages PRIMARY KEY (id)
);

COMMENT ON TABLE public.messages IS 'チャットメッセージを保持するテーブル';
COMMENT ON COLUMN public.messages.id IS 'メッセージID。uuidv7をアプリケーション側で採番して保存する';
COMMENT ON COLUMN public.messages.cognito_id IS '投稿ユーザーのCognitoサブジェクトID';
COMMENT ON COLUMN public.messages.created_at IS 'メッセージ作成日時';
COMMENT ON COLUMN public.messages.body IS 'メッセージ本文';
COMMENT ON COLUMN public.messages.is_from_user IS 'true: ユーザー投稿, false: システム投稿';

--changeset sns-system-db:0002-create-messages-index-local context:local
CREATE INDEX IF NOT EXISTS idx_messages_user_timeline
    ON public.messages (cognito_id, created_at DESC)
    INCLUDE (body, is_from_user);

--changeset sns-system-db:0003-create-messages-index-dsql context:develop,release
CREATE INDEX ASYNC IF NOT EXISTS idx_messages_user_timeline
    ON public.messages (cognito_id, created_at DESC)
    INCLUDE (body, is_from_user);

--changeset sns-system-db:0004-create-messages-view
CREATE OR REPLACE VIEW public.messages_latest AS
SELECT
    id,
    cognito_id,
    created_at,
    body,
    is_from_user
FROM public.messages;

COMMENT ON VIEW public.messages_latest IS '最新メッセージ取得向けの互換ビュー';

--changeset sns-system-db:0005-grant-lambda-role context:develop,release
AWS IAM GRANT SELECT, INSERT ON TABLE public.messages TO '${LambdaRoleArn}';
