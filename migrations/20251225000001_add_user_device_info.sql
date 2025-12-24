-- Adds device information to the user_devices table.

alter table user_devices
  add column device_name  text null,
  add column os           text null,
  add column game_version text null,
  add column language     text null,
  add column country      text null;
