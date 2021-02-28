create table if not exists main.ships (
    id integer not null primary key,
    name varchar not null,
    warp_speed integer not null,
    faction varchar
);