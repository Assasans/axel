-- Adds characters and members tables. Likely not final schema.

drop table if exists user_characters cascade;
create table user_characters
(
  user_id        bigint not null references users (id) on delete restrict,
  character_id   bigint not null,
  intimacy       int    not null default 0,
  -- Skills, probably isn't worth normalizing
  -- special_skills jsonb  not null,
  primary key (user_id, character_id)
);

drop table if exists user_character_skills cascade;
-- create table user_character_skills
-- (
--   user_id      bigint not null,
--   character_id bigint not null,
--   skill_id     bigint not null,
--   level        int    not null default 1,
--   primary key (user_id, character_id, skill_id),
--   foreign key (user_id, character_id) references user_characters (user_id, character_id) on delete restrict
-- );

drop table if exists user_character_pieces cascade;
-- create table user_character_pieces
-- (
--   user_id      bigint not null,
--   character_id bigint not null,
--   board_id     bigint not null,
--   -- Current stage (blessing) of the board (blessing path)
--   stage_id     bigint not null,
--   primary key (user_id, character_id, board_id),
--   foreign key (user_id, character_id) references user_characters (user_id, character_id) on delete restrict
-- );

drop table if exists user_members cascade;
create table user_members
(
  user_id         bigint not null references users (id) on delete restrict,
  member_id       bigint not null,
  -- Leveling
  xp              int    not null default 0,
  promotion_level int    not null default 0,
  -- Skills, probably isn't worth normalizing
  -- active_skills   jsonb  not null,
  primary key (user_id, member_id)
);

drop table if exists user_member_skills cascade;
-- create table user_member_skills
-- (
--   user_id   bigint not null references users (id) on delete restrict,
--   member_id bigint not null,
--   skill_id  bigint not null,
--   level     int    not null default 1,
--   value     int    not null,
--   primary key (user_id, member_id, skill_id),
--   foreign key (user_id, member_id) references user_members (user_id, member_id) on delete restrict
-- );
