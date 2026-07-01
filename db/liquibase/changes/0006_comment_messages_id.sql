--liquibase formatted sql
--changeset copilot:0006
COMMENT ON COLUMN public.messages.id IS '主キー。デフォルトでUUIDv4を採番する。';
