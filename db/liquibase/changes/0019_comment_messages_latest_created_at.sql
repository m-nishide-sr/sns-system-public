--liquibase formatted sql
--changeset copilot:0019
COMMENT ON COLUMN public.messages_latest.created_at IS 'メッセージ作成日時。';
