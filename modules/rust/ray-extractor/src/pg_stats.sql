SELECT 
    tablename,
    attname AS column_name,
    null_frac,
    avg_width,
    n_distinct,
    most_common_vals,
    most_common_freqs,
    histogram_bounds
FROM pg_stats
WHERE schemaname NOT IN ('pg_catalog', 'information_schema') li
ORDER BY tablename, attname ;




SELECT *
FROM postgres_air.flight
WHERE departure_airport = 'JFK' limit 10;
----

EXPLAIN
SELECT flight_no
FROM postgres_air.flight
WHERE departure_airport = 'JFK';



----


EXPLAIN ANALYZE
SELECT flight_no
FROM postgres_air.flight
WHERE departure_airport = 'JFK';


------

CREATE INDEX flight_departure_airport_idx
ON postgres_air.flight(departure_airport);


SELECT relpages
FROM pg_class
WHERE oid = 'postgres_air.flight'::regclass;

SHOW seq_page_cost;


SELECT reltuples
FROM pg_class
WHERE oid = 'postgres_air.flight'::regclass;

SHOW cpu_tuple_cost;
