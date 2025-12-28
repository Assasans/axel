-- Adds user reserve members table.

drop table if exists user_members_reserve;
create table user_members_reserve
(
  id        bigserial primary key,
  user_id   bigint not null references users (id) on delete restrict,
  member_id bigint not null
);

create index ix_user_members_reserve_user_id_member_id
  on user_members_reserve (user_id, member_id);
