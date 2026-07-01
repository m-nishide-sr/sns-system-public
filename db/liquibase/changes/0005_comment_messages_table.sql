--liquibase formatted sql
--changeset copilot:0005
COMMENT ON TABLE public.messages IS 'チャットメッセージを保持するテーブル。';
