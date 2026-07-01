--liquibase formatted sql
--changeset copilot:0011
COMMENT ON COLUMN public.messages.row_log IS '生ログ。不具合などの調査時にのみ参照される想定。';
