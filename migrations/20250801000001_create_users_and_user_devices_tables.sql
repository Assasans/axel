drop table if exists users cascade;
create table users
(
  id                bigserial
    constraint user_pk primary key,
  username          text        default null  null,
  about_me          text        default null  null,
  created_at        timestamptz default now() not null,
  tutorial_progress int         default 0     not null
);

-- One user can have multiple devices which can be used to log in into the same account.
drop table if exists user_devices cascade;
create table user_devices
(
  id         bigserial
    constraint user_device_pk primary key,
  user_id    bigint                    not null
    constraint user_device_fk_user_id references users (id) on delete restrict,
  token      text                      not null
    constraint user_device_ak_token unique,
  created_at timestamptz default now() not null,
  last_used  timestamptz default null  null
);
