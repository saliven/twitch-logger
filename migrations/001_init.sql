-- Tables

CREATE TABLE logs (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    username varchar(32) NOT NULL,
    channel varchar(32) NOT NULL,
    content text,
    log_type varchar(16) NOT NULL DEFAULT 'message',
    created_at timestamp WITH TIME ZONE DEFAULT NOW()
);

-- Indexes

CREATE INDEX logs_created_at_idx ON logs (created_at);
