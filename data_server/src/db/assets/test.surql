BEGIN TRANSACTION;
DEFINE NS {app_name}_ns;
USE NS {app_name}_ns;
DEFINE DB {app_name}_db;
USE DB {app_name}_db;

DEFINE TABLE object_table SCHEMAFUL;
DEFINE FIELD published ON object_table
    TYPE bool
    VALUE $value OR false;
DEFINE FIELD system on object_table
    TYPE bool
    VALUE $value OR false
    PERMISSIONS
        FOR create, update NONE;
DEFINE FIELD settings.* ON object_table TYPE object;
DEFINE FIELD name ON object_table TYPE string;

DEFINE TABLE object_field SCHEMAFUL;
DEFINE FIELD settings.* ON object_field TYPE object;
DEFINE FIELD name ON object_field TYPE string;

CREATE object_table:object_table SET system = true;
CREATE object_field:object_field;
RELATE object_table:object_table -> has_field -> object_field:object_field;

DEFINE LOGIN rush_root_db_user ON DATABASE PASSWORD 'test';

DEFINE INDEX unique_relationships
    ON TABLE has_field 
    COLUMNS in, out UNIQUE;

DEFINE EVENT publish_object ON object_table WHEN $after.published = true THEN (
    http::post("http://127.0.0.1:8080/api/objects/publish", (SELECT *, ->has_field.out.* AS object_fields FROM object_table WHERE id = $after.id))
);

COMMIT TRANSACTION;