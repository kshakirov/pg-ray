use serde::{Deserialize, Serialize};

/// Паспорт таблицы: базовые физические метрики из pg_class и сгруппированные колонки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMeta {
    pub oid: u32,
    pub table_name: String,
    pub relkind: String,
    /// Оценка количества строк (reltuples из pg_class).
    /// Выступает как эмпирическое N для максентовского веса p_i = 1/N.
    pub estimated_rows: f32,
    /// Количество физических 8KB страниц (relpages из pg_class).
    pub total_pages: i32,
    /// Статистика по колонкам таблицы из pg_stats
    pub columns: Vec<ColumnStats>,
}

/// Статистика отдельной колонки из pg_stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub column_name: String,
    /// Доля NULL-значений (null_frac: 0.0 .. 1.0)
    pub null_frac: f32,
    /// Средняя ширина значения в байтах (avg_width).
    /// Нужна для расчета памяти под выборку: Memory = N * W.
    pub avg_width: i32,
    /// Мощность/число уникальных значений (n_distinct).
    /// > 0: абсолютное число уников; < 0: отрицательная доля от общего N.
    pub n_distinct: f32,

    pub mcv: Option<Vec<String>>,
    pub mcf: Option<Vec<f32>>,
}

use std::collections::HashMap;
use tokio_postgres::{Client, Error};

pub async fn fetch_table_meta(client: &Client) -> Result<Vec<TableMeta>, Error> {
    let query = "
        SELECT 
            c.oid,
            c.relname::text AS table_name,
            c.relkind::text,
            c.reltuples::real AS estimated_rows,
            c.relpages::integer AS total_pages,
            COALESCE(s.attname, '')::text AS column_name,
            COALESCE(s.null_frac, 0.0)::real AS null_frac,
            COALESCE(s.avg_width, 0)::integer AS avg_width,
            COALESCE(s.n_distinct, 0.0)::real AS n_distinct,
         
          CASE
              WHEN s.most_common_vals IS NULL THEN NULL
          ELSE ARRAY(
         SELECT jsonb_array_elements_text(
            to_jsonb(s.most_common_vals)
        )
    )
END AS mcv,
           s.most_common_freqs as mcf

        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        LEFT JOIN pg_stats s ON s.tablename = c.relname AND s.schemaname = n.nspname
        WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
          AND n.nspname NOT LIKE 'pg_toast%'
          AND c.relkind = 'r'
        ORDER BY c.relname, s.attname limit 10;
    ";

    let rows = client.query(query, &[]).await?;
    let mut tables_map: HashMap<u32, TableMeta> = HashMap::new();

    for row in rows {
        let oid: u32 = row.get("oid");
        let column_name: String = row.get("column_name");

        let entry = tables_map.entry(oid).or_insert_with(|| TableMeta {
            oid,
            table_name: row.get("table_name"),
            relkind: row.get("relkind"),
            estimated_rows: row.get("estimated_rows"),
            total_pages: row.get("total_pages"),
            columns: Vec::new(),
        });

        // Если у таблицы есть статистика по колонкам
        if !column_name.is_empty() {
            entry.columns.push(ColumnStats {
                column_name,
                null_frac: row.get("null_frac"),
                avg_width: row.get("avg_width"),
                n_distinct: row.get("n_distinct"),
                mcv: row.get("mcv"),
                mcf: row.get("mcf"),
            });
        }
    }

    Ok(tables_map.into_values().collect())
}

pub async fn get_mcf_by_mcv(client: &Client, mcv: &str) -> Result<f64, Error> {
    let query = format!(
        "
with sel_idx as (
select idx from (
SELECT 
    t.elem as name, 
    t.ord as idx
FROM pg_stats s,
     jsonb_array_elements_text(to_jsonb(s.most_common_vals)) WITH ORDINALITY AS t(elem, ord)
WHERE s.tablename = 'account' 
  AND s.attname = 'first_name'
  ) where name = '{}'
 )
select (to_jsonb(most_common_freqs)->> (select idx from sel_idx)::int)::float as idx  FROM pg_stats 
WHERE tablename = 'account' AND attname = 'first_name'

",
        mcv
    );
    let row = client.query_one(&query, &[]).await?;
    let result: f64 = row.get("idx");
    Ok(result)
}
