--liquibase formatted sql
--changeset copilot:0003 context:develop
AWS IAM GRANT lambda TO 'arn:aws:iam::${AWS_ACCOUNT_ID}:role/sns-db-${Stage}-lambda-role';
