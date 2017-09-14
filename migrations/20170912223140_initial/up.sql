create table auth (
    id              serial primary key,
    salt            bytea not null,
    hash            bytea not null,
    date_created    timestamp with time zone not null default now()
);

create table init_upload (
    id                  serial primary key,
    uuid_               uuid unique not null,
    file_name           text not null,
    iv                  bytea not null,
    access_password     integer not null unique references "auth" ("id") on delete cascade,
    date_created        timestamp with time zone not null default now()
);

create table upload (
    id                  serial primary key,
    uuid_               uuid unique not null,
    content_hash        bytea not null,
    file_name           text not null,
    file_path           text not null,
    iv                  bytea not null,
    access_password     integer not null unique references "auth" ("id") on delete cascade,
    date_created        timestamp with time zone not null default now()
);
