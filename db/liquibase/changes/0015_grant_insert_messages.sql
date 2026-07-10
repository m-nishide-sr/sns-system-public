--liquibase formatted sql
--changeset copilot:0015 context:local,develop
GRANT INSERT ON public.messages TO lambda;
