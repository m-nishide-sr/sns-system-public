--liquibase formatted sql
--changeset copilot:0005 context:local,develop
COMMENT ON TABLE public.messages IS 'チャットメッセージを保持するテーブル。';
