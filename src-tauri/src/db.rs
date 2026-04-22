use tauri_plugin_sql::{Migration, MigrationKind};

pub fn migrations() -> Vec<Migration> {
    vec![Migration {
        version: 1,
        description: "initial schema",
        sql: "
            CREATE TABLE IF NOT EXISTS plugins (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                version     TEXT NOT NULL,
                repo        TEXT,
                path        TEXT NOT NULL,
                permissions TEXT,
                enabled     INTEGER DEFAULT 1,
                created_at  INTEGER DEFAULT (unixepoch())
            );
            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
        ",
        kind: MigrationKind::Up,
    }]
}
