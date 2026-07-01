--liquibase formatted sql
--changeset copilot:0018
COMMENT ON COLUMN public.messages_latest.user_name IS '投稿ユーザーのemailアドレスの@以前の文字列。';
