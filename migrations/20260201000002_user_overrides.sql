-- Adds user overrides table.

drop table if exists user_overrides cascade;
create table user_overrides
(
  user_id  bigint not null references users (id) on delete restrict,
  key      text   not null,
  override text   not null,
  primary key (user_id, key)
);
