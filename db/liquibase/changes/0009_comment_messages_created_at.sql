--liquibase formatted sql
--changeset copilot:0009 context:local,develop
COMMENT ON COLUMN public.messages.created_at IS 'メッセージ作成日時。';
