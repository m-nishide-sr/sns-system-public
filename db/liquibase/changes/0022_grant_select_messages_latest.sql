--liquibase formatted sql
--changeset copilot:0022 context:local,develop
GRANT SELECT ON public.messages_latest TO lambda;
