-- migration: init_schema
-- created: 2025-12-21 15:24:36.149483400 +02:00

DEFINE TABLE user SCHEMALESS;
DEFINE FIELD name ON TABLE user TYPE string;
DEFINE FIELD email ON TABLE user TYPE string ASSERT string::is::email($value);
DEFINE INDEX email_unique ON TABLE user COLUMNS email UNIQUE;