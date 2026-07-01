--liquibase formatted sql
--changeset copilot:0020
COMMENT ON COLUMN public.messages_latest.body IS 'メッセージ本文。';
