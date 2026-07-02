--liquibase formatted sql
--changeset copilot:0012 context:local,develop
COMMENT ON COLUMN public.messages.is_from_user IS 'true: ユーザー投稿, false: システム投稿。';
