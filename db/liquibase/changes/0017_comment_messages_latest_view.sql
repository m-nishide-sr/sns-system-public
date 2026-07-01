--liquibase formatted sql
--changeset copilot:0017
COMMENT ON VIEW public.messages_latest IS 'public.messagesをそのまま参照する互換ビュー。';
