-- for distributed table
CREATE TABLE logs_local (
    "username" String,
    "channel" String,
    "content" Nullable(String),
    "log_type" Enum8('chat' = 1, 'ban' = 2),
    "created_at" DateTime64(9),
    "user_id" Nullable(String),
    "color" Nullable(String),
    "badges" Array(String)
) ENGINE MergeTree()
PARTITION BY toYYYYMM("created_at")
PRIMARY KEY ("created_at", "username", "channel");

CREATE TABLE logs AS logger.logs_local
ENGINE = Distributed("clickhouse-cluster", logger, logs_local, rand());
