BEGIN;

CREATE TABLE logs (
    id uuid DEFAULT gen_random_uuid(),
    username varchar(32) NOT NULL,
    channel varchar(32) NOT NULL,
    content text,
    log_type varchar(16) NOT NULL DEFAULT 'message',
    created_at timestamp WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX logs_created_at_idx ON logs (created_at);
CREATE INDEX logs_username_idx ON logs (username);
CREATE INDEX logs_channel_idx ON logs (channel);

SELECT create_hypertable('logs', 'created_at');

COMMIT;