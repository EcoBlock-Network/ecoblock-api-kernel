-- 0003_grant_privs.sql
-- Grant required privileges to the application role (ecoblock)

-- NOTE: replace 'ecoblock' with your app DB user if different.
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE public.blogs TO ecoblock;
GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE public.stories TO ecoblock;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO ecoblock;
