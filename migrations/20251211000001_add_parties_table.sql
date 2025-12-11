-- Adds parties and party forms table.

drop table if exists user_parties cascade;
create table user_parties
(
  user_id  bigint not null references users (id) on delete restrict,
  party_id bigint not null,
  name     text   not null,
  primary key (user_id, party_id)
);

drop table if exists user_party_forms cascade;
create table user_party_forms
(
  user_id        bigint not null references users (id) on delete restrict,
  party_id       bigint not null,
  form_id        bigint not null,
  main_member_id bigint null default 0,
  sub1_member_id bigint null default 0,
  sub2_member_id bigint null default 0,
  weapon_id      bigint null default 0,
  accessory_id   bigint null default 0,
  primary key (user_id, party_id, form_id),
  foreign key (user_id, party_id) references user_parties (user_id, party_id) on delete restrict
);

create or replace function enforce_forms_per_party() returns trigger as
$$
begin
  if (select count(*) from user_party_forms where user_id = new.user_id and party_id = new.party_id) != 5 then
    raise exception 'A party must have exactly 5 forms, got %.', (select count(*)
                                                                  from user_party_forms
                                                                  where user_id = new.user_id
                                                                    and party_id = new.party_id);
  end if;
  return new;
end;
$$ language plpgsql;

create constraint trigger trg_enforce_forms_per_party
  after insert or update
  on user_party_forms
  for each row
execute function enforce_forms_per_party();

-- insert into user_parties(user_id, party_id, name)
-- values (1, 1, 'Chunchunmaru');
--
-- insert into user_party_forms(user_id, party_id, form_id, main_member_id, sub1_member_id, sub2_member_id, weapon_id,
--                              accessory_id)
-- values (1, 1, 1, 1, 6, 11, 0, 0),
--        (1, 1, 2, 2, 7, 12, 0, 0),
--        (1, 1, 3, 3, 8, 13, 0, 0),
--        (1, 1, 4, 4, 9, 14, 0, 0),
--        (1, 1, 5, 5, 10, 15, 0, 0);

-- with ins_party
--        as ( insert into user_parties (user_id, party_id, name) values (1, 2, 'CTE Party') returning user_id, party_id)
-- insert
-- into user_party_forms(user_id, party_id, form_id, main_member_id, sub1_member_id, sub2_member_id, weapon_id,
--                       accessory_id)
-- values ((select user_id from ins_party), (select party_id from ins_party), 1, 1, 0, 0, 0, 0),
--        ((select user_id from ins_party), (select party_id from ins_party), 2, 2, 0, 0, 0, 0),
--        ((select user_id from ins_party), (select party_id from ins_party), 3, 3, 0, 0, 0, 0),
--        ((select user_id from ins_party), (select party_id from ins_party), 4, 4, 0, 0, 0, 0),
--        ((select user_id from ins_party), (select party_id from ins_party), 5, 5, 0, 0, 0, 0);

select up.user_id,
       up.party_id,
       up.name,
       upf.form_id,
       upf.main_member_id,
       upf.sub1_member_id,
       upf.sub2_member_id,
       upf.weapon_id,
       upf.accessory_id
from user_parties up
       join user_party_forms upf on up.user_id = upf.user_id and up.party_id = upf.party_id
where up.user_id = 1
order by up.party_id, upf.form_id;
