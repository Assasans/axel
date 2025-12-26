-- Adds self-maintenance user along with Row Level Security policies.

-- Enable Row Level Security
alter table users
  enable row level security;
alter table user_devices
  enable row level security;
alter table user_characters
  enable row level security;
alter table user_members
  enable row level security;
alter table user_parties
  enable row level security;
alter table user_party_forms
  enable row level security;

-- RLS policies
create schema if not exists app;
create or replace function app.current_user_id()
  returns bigint
  language sql
  stable
as
$$
select current_setting('app.user_id', true)::bigint
$$;

--- users (SU)
drop policy if exists user_self_read on users;
create policy user_self_read on users for select using (id = app.current_user_id());

drop policy if exists user_self_update on users;
create policy user_self_update on users for update using (id = app.current_user_id()) with check (id = app.current_user_id());

--- user_devices (SUID)
drop policy if exists user_device_self_read on user_devices;
create policy user_device_self_read on user_devices for select using (user_id = app.current_user_id());

drop policy if exists user_device_self_update on user_devices;
create policy user_device_self_update on user_devices for update using (user_id = app.current_user_id()) with check (user_id = app.current_user_id());

drop policy if exists user_device_self_insert on user_devices;
create policy user_device_self_insert on user_devices for insert with check (user_id = app.current_user_id());

drop policy if exists user_device_self_delete on user_devices;
create policy user_device_self_delete on user_devices for delete using (user_id = app.current_user_id());

-- user_characters (SUID)
drop policy if exists user_character_self_read on user_characters;
create policy user_character_self_read on user_characters for select using (user_id = app.current_user_id());

drop policy if exists user_character_self_update on user_characters;
create policy user_character_self_update on user_characters for update using (user_id = app.current_user_id()) with check (user_id = app.current_user_id());

drop policy if exists user_character_self_insert on user_characters;
create policy user_character_self_insert on user_characters for insert with check (user_id = app.current_user_id());

drop policy if exists user_character_self_delete on user_characters;
create policy user_character_self_delete on user_characters for delete using (user_id = app.current_user_id());

-- user_members (SUID)
drop policy if exists user_member_self_read on user_members;
create policy user_member_self_read on user_members for select using (user_id = app.current_user_id());

drop policy if exists user_member_self_update on user_members;
create policy user_member_self_update on user_members for update using (user_id = app.current_user_id()) with check (user_id = app.current_user_id());

drop policy if exists user_member_self_insert on user_members;
create policy user_member_self_insert on user_members for insert with check (user_id = app.current_user_id());

drop policy if exists user_member_self_delete on user_members;
create policy user_member_self_delete on user_members for delete using (user_id = app.current_user_id());

-- user_parties (SUID)
drop policy if exists user_party_self_read on user_parties;
create policy user_party_self_read on user_parties for select using (user_id = app.current_user_id());

drop policy if exists user_party_self_update on user_parties;
create policy user_party_self_update on user_parties for update using (user_id = app.current_user_id()) with check (user_id = app.current_user_id());

drop policy if exists user_party_self_insert on user_parties;
create policy user_party_self_insert on user_parties for insert with check (user_id = app.current_user_id());

drop policy if exists user_party_self_delete on user_parties;
create policy user_party_self_delete on user_parties for delete using (user_id = app.current_user_id());

-- user_party_forms (SUID)
drop policy if exists user_party_form_self_read on user_party_forms;
create policy user_party_form_self_read on user_party_forms for select using (user_id = app.current_user_id());

drop policy if exists user_party_form_self_update on user_party_forms;
create policy user_party_form_self_update on user_party_forms for update using (user_id = app.current_user_id()) with check (user_id = app.current_user_id());

drop policy if exists user_party_form_self_insert on user_party_forms;
create policy user_party_form_self_insert on user_party_forms for insert with check (user_id = app.current_user_id());

drop policy if exists user_party_form_self_delete on user_party_forms;
create policy user_party_form_self_delete on user_party_forms for delete using (user_id = app.current_user_id());

-- Create user
-- drop owned by axel_edit;
-- drop user if exists axel_edit;
-- create
--   user axel_edit with password 'axel-edit';
revoke all on schema public from axel_edit;
grant usage on schema public to axel_edit;

-- Grant permissions
grant select, update on users to axel_edit;
grant select, update, insert, delete on user_devices to axel_edit;
grant select, update, insert, delete on user_characters to axel_edit;
grant select, update, insert, delete on user_members to axel_edit;
grant select, update, insert, delete on user_parties to axel_edit;
grant select, update, insert, delete on user_party_forms to axel_edit;
