--liquibase formatted sql
--changeset copilot:0007 context:local,develop
COMMENT ON COLUMN public.messages.user_name IS '投稿ユーザーのemailアドレスの@以前の文字列。';
