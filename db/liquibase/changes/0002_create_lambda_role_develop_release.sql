--liquibase formatted sql
--changeset copilot:0002 context:develop,release
CREATE ROLE lambda WITH LOGIN;
