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
    content_hash        bytea not null,
    size_               bigint not null,
    nonce               bytea not null,
    access_password     integer not null unique references "auth" ("id") on delete cascade,
    deletion_password   integer unique references "auth" ("id") on delete cascade,
    download_limit      integer,
    expire_date         timestamp with time zone not null,
    date_created        timestamp with time zone not null default now()
);

create table upload (
    id                  serial primary key,
    uuid_               uuid unique not null,
    content_hash        bytea not null,
    size_               bigint not null,
    file_name           text not null,
    file_path           text not null,
    nonce               bytea not null,
    access_password     integer not null unique references "auth" ("id") on delete cascade,
    deletion_password   integer unique references "auth" ("id") on delete cascade,
    download_limit      integer,
    expire_date         timestamp with time zone not null,
    deleted             boolean default false,
    date_created        timestamp with time zone not null default now()
);

create table init_download (
    id              serial primary key,
    uuid_           uuid unique not null,
    usage           text not null,
    upload          integer not null references "upload" ("id") on delete cascade,
    date_created    timestamp with time zone not null default now()
);

create table download (
    id                  serial primary key,
    upload              integer not null references "upload" ("id") on delete cascade,
    date_created        timestamp with time zone not null default now()
);

create table status (
    id              serial primary key,
    upload_count    bigint not null,
    total_bytes     bigint not null,
    date_modified   timestamp with time zone not null
);

