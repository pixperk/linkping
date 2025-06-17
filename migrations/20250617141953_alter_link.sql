-- Add migration script here
ALTER TABLE links ADD COLUMN expires_at TIMESTAMP WITH TIME ZONE;
