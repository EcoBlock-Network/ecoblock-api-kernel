-- 0000_create_role.sql
-- Create the application role expected by later migrations
-- WARNING: this creates a role with LOGIN and a simple password for local dev only.

DO $$
BEGIN
   IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'ecoblock') THEN
      CREATE ROLE ecoblock WITH LOGIN PASSWORD 'ecoblock';
   END IF;
END$$;

GRANT CONNECT ON DATABASE ecoblock TO ecoblock;
