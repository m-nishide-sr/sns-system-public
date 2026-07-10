--liquibase formatted sql
--changeset copilot:0010 context:local,develop
COMMENT ON COLUMN public.messages.body IS 'メッセージ本文。';
