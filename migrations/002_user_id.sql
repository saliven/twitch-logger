BEGIN;

-- Add column user_id to logs table
ALTER TABLE logs ADD COLUMN user_id varchar(32) DEFAULT NULL;

-- Create index on user_id
CREATE INDEX IF NOT EXISTS logs_user_id_idx ON logs (user_id);

-- Create enum for log_type
DROP TYPE IF EXISTS log_type;

CREATE TYPE log_type AS ENUM ('chat', 'ban');

-- Update log_type column to use enum
ALTER TABLE logs
	ALTER COLUMN log_type DROP DEFAULT,
	ALTER COLUMN log_type TYPE log_type USING log_type::log_type,
	ALTER COLUMN log_type SET DEFAULT 'chat';

COMMIT;
