--liquibase formatted sql
--changeset copilot:0006 context:local,develop
COMMENT ON COLUMN public.messages.id IS '主キー。デフォルトでUUIDv4を採番する。';
