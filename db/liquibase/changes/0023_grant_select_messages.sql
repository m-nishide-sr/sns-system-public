--liquibase formatted sql
--changeset m-nishide-sr:0023 context:local,develop
GRANT SELECT (id, user_name, cognito_id, created_at, body, row_log, is_from_user) ON public.messages TO lambda;
