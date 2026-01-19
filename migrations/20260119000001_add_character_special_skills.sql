-- Adds characters and members tables. Likely not final schema.

drop table if exists user_character_special_skills cascade;
create table user_character_special_skills
(
  user_id      bigint not null references users (id) on delete restrict,
  character_id bigint not null,
  skill_id     bigint not null,
  level        int    not null default 1,
  primary key (user_id, character_id, skill_id)
);

alter table user_party_forms
  drop column if exists special_skill_id;
-- Server will populate NULL values with default skill, later NULL values should be disallowed. Or not?
-- Maybe all columns that are set to NULL could be populated with default values by server.
alter table user_party_forms
  add column special_skill_id bigint null default null;
