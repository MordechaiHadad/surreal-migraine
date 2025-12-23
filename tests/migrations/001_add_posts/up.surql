-- migration: add_posts
-- created: 2025-12-21 15:25:14.247012800 +02:00

DEFINE TABLE post SCHEMALESS;
DEFINE FIELD title ON TABLE post TYPE string;
DEFINE FIELD content ON TABLE post TYPE string;
DEFINE FIELD author ON TABLE post TYPE record<user>;