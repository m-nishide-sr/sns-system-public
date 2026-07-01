--liquibase formatted sql
--changeset copilot:0015
GRANT INSERT ON public.messages TO lambda;
