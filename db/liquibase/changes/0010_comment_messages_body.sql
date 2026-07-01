--liquibase formatted sql
--changeset copilot:0010
COMMENT ON COLUMN public.messages.body IS 'メッセージ本文。';
