-- Adds user equipment items table

drop table if exists user_items_equipment;
create table user_items_equipment
(
  -- We can have duplicate equipment, even with the same level
  id        bigserial primary key,
  user_id   bigint  not null references users (id) on delete restrict,
  item_type bigint  not null
    constraint chk_user_items_equipment_type check (item_type = 4 or item_type = 5),
  item_id   bigint  not null,
  level     integer not null default 1,
  is_locked boolean not null default false
);

create index ix_user_items_equipment_user_id_item_id
  on user_items_equipment (user_id);

-- Forbid equipment items in the user_items table
alter table user_items
  drop constraint if exists chk_user_items_no_equipment;
alter table user_items
  add constraint chk_user_items_no_equipment check (item_type != 4 and item_type != 5);
