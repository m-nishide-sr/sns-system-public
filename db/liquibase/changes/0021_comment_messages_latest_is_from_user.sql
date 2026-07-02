--liquibase formatted sql
--changeset copilot:0021 context:local,develop
COMMENT ON COLUMN public.messages_latest.is_from_user IS 'true: ユーザー投稿, false: システム投稿。';
