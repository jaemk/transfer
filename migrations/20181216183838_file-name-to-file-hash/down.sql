alter table upload
  drop column file_name_hash;

alter table upload
  add column file_name text;

alter table init_upload
  drop column file_name_hash;

alter table init_upload
  add column file_name text;