--liquibase formatted sql
--changeset copilot:0016 context:local,develop
CREATE VIEW public.messages_latest AS
SELECT
  user_name,
  created_at,
  body,
  is_from_user
FROM public.messages;
