alter table upload
  add column file_name_hash bytea;

alter table upload
  drop column file_name;

alter table init_upload
  add column  file_name_hash bytea;

alter table init_upload
  drop column file_name;