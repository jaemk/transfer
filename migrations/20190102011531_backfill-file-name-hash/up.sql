update init_upload set file_name_hash = '0x00' where file_name_hash is null;
update upload set file_name_hash = '0x00' where file_name_hash is null;
