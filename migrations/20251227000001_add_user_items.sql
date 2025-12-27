-- Adds user items table.

drop table if exists user_items;
create table user_items
(
  user_id   bigint  not null references users (id) on delete restrict,
  item_type bigint  not null,
  item_id   bigint  not null,
  quantity  integer not null default 1,
  primary key (user_id, item_type, item_id)
);

-- Give some default items if those items do not already exist
-- insert into user_items (user_id, item_type, item_id, quantity)
-- select 1 as user_id,
--        v.item_type,
--        v.item_id,
--        v.quantity
-- from (select item_type, item_id, sum(quantity) as quantity
--       from (values (18::bigint, 1::bigint, 1000::integer),
--                    (18::bigint, 2::bigint, 2000::integer),
--                    (18::bigint, 3::bigint, 3000::integer)) as t(item_type, item_id, quantity)
--       group by item_type, item_id) as v
-- on conflict (user_id, item_type, item_id) do nothing;
