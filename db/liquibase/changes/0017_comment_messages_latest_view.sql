--liquibase formatted sql
--changeset copilot:0017 context:local,develop
COMMENT ON VIEW public.messages_latest IS 'public.messagesをそのまま参照する互換ビュー。';
