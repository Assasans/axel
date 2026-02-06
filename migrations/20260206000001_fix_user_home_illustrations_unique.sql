-- Fixed user_home_illustrations UNIQUE constraint to allow duplicate member_id values for different users.

alter table user_home_illustrations
  drop constraint if exists user_home_illustrations_member_id_key;
alter table user_home_illustrations
  add constraint user_home_illustrations_member_id_key unique (user_id, member_id);
