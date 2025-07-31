-- Script to create the alert_server database
-- Run this as postgres superuser: psql -U postgres -h 10.26.64.224 -f create_database.sql

-- Create alert_server database if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_database WHERE datname = 'alert_server') THEN
        PERFORM dblink_exec('dbname=' || current_database(), 'CREATE DATABASE alert_server');
    END IF;
END
$$;

-- Alternative approach using conditional SQL
SELECT 'CREATE DATABASE alert_server'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'alert_server')\gexec