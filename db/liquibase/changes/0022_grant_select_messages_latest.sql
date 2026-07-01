--liquibase formatted sql
--changeset copilot:0022
GRANT SELECT ON public.messages_latest TO lambda;
