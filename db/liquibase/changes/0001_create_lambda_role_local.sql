--liquibase formatted sql
--changeset copilot:0001 context:local
CREATE ROLE lambda WITH LOGIN PASSWORD 'lambda';
