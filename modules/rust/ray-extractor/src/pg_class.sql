SELECT 
    c.oid,
    c.relname AS table_name,
    c.relkind,
    c.reltuples AS estimated_rows,
    c.relpages AS total_pages,
    (c.relpages::bigint * 8192) / (1024 * 1024) AS size_mb
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
  AND n.nspname NOT LIKE 'pg_toast%'
  AND c.relkind = 'r'
ORDER BY c.relpages DESC;
