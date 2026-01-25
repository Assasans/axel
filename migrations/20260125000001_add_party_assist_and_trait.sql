-- Adds assist and trait columns to user parties table.

alter table user_parties
  drop column if exists assist_id,
  drop column if exists trait_id;
alter table user_parties
  add column assist_id bigint null default 0,
  add column trait_id  bigint null default 0;
