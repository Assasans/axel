-- Adds user member skills table.

drop table if exists user_member_skills cascade;
create table user_member_skills
(
  user_id   bigint not null references users (id) on delete restrict,
  member_id bigint not null,
  -- Skill ID should be in sync with master data,
  skill_id  bigint not null,
  level     int    not null default 1,
  primary key (user_id, member_id, skill_id)
);
