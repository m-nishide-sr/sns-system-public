--liquibase formatted sql
--changeset copilot:0008
COMMENT ON COLUMN public.messages.cognito_id IS '投稿ユーザーのCognitoサブジェクトID。';
