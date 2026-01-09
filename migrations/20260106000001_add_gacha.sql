-- Adds gacha scheme along with rates and bonus rates tables.

drop schema if exists gacha cascade;
create schema gacha;

drop table if exists gacha.rates;
create table gacha.rates
(
  gacha_id         bigint,
  item_id          bigint        not null,
  -- Client allows percentage up to 3 decimal places.
  probability      numeric(6, 3) not null
    constraint chk_gacha_rates_probability check (probability between 0 and 100),
  -- Probability for 10th draw. Called 'limitrate' in the client. For some reason they like to use 'limit' everywhere.
  probability_pity numeric(6, 3) null
    constraint chk_gacha_rates_probability_pity check (probability_pity between 0 and 100),
  is_rate_up       boolean       not null default false,
  details_priority integer       null     default null,
  constraint pk_gacha_rates primary key (gacha_id, item_id)
);

drop table if exists gacha.bonus_rates;
create table gacha.bonus_rates
(
  gacha_id    bigint,
  pack_id     bigint        not null,
  probability numeric(6, 3) not null check (probability between 0 and 100),
  constraint pk_gacha_bonus_rates primary key (gacha_id, pack_id)
);
