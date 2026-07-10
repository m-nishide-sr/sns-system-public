--liquibase formatted sql
--changeset copilot:0004 context:local,develop
CREATE TABLE public.messages (
  id uuid NOT NULL DEFAULT gen_random_uuid(),
  user_name text NOT NULL DEFAULT '',
  cognito_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  body text NOT NULL DEFAULT '',
  row_log text NOT NULL,
  is_from_user boolean NOT NULL,
  CONSTRAINT pk_messages PRIMARY KEY (id)
);
