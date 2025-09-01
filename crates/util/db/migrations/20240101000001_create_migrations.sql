CREATE TABLE IF NOT EXISTS migrations (
    id BIGINT PRIMARY KEY,
    timestamp BIGINT NOT NULL,
    name TEXT NOT NULL
);
