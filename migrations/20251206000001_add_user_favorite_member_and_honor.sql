-- Adds users.favorite_member and users.honor columns.
-- Favorite member defaults to 1001100 "Kazuma (Beginner)".
-- Honor defaults to 60000000 "Newbie Adventurer".

alter table users
  add column favorite_member bigint default 1001100  not null,
  add column honor           bigint default 60000000 not null;
