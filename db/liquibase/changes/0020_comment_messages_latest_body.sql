--liquibase formatted sql
--changeset copilot:0020 context:local,develop
COMMENT ON COLUMN public.messages_latest.body IS 'メッセージ本文。';
