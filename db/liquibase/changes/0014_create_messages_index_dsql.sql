--liquibase formatted sql
--changeset copilot:0014 context:develop
CREATE INDEX ASYNC idx_messages_user_timeline ON public.messages (created_at) INCLUDE (user_name, body, is_from_user);
