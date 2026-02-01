-- Adds home screen related tables.

drop table if exists user_home_illustrations cascade;
create table user_home_illustrations
(
  user_id   bigint  not null references users (id) on delete restrict,
  slot      integer not null
    constraint chk_user_home_illustrations_slot check (slot between 1 and 5),
  member_id bigint  null unique
    constraint chk_user_home_illustrations_member_id check (member_id is null or member_id > 0),
  primary key (user_id, slot)
);

alter table users
  drop column if exists home_current_illustration_id;
alter table users
  add column home_current_illustration_id bigint null;
