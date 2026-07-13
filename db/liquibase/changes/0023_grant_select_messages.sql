--liquibase formatted sql
--changeset m-nishide-sr:0023 context:local,develop
GRANT SELECT ON public.messages TO lambda;
