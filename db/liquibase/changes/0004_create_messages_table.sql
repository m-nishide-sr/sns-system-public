--liquibase formatted sql
--changeset copilot:0004
CREATE TABLE public.messages (
  id uuid NOT NULL DEFAULT gen_random_uuid(),
  user_name text NOT NULL DEFAULT '',
  cognito_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
  body text NOT NULL DEFAULT '',
  row_log text NOT NULL,
  is_from_user boolean NOT NULL,
  CONSTRAINT pk_messages PRIMARY KEY (id)
);
