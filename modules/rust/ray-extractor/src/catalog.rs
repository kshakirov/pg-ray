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
}


use tokio_postgres::{Client, Error};
use std::collections::HashMap;

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
            COALESCE(s.n_distinct, 0.0)::real AS n_distinct
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        LEFT JOIN pg_stats s ON s.tablename = c.relname AND s.schemaname = n.nspname
        WHERE n.nspname NOT IN ('pg_catalog', 'information_schema')
          AND n.nspname NOT LIKE 'pg_toast%'
          AND c.relkind = 'r'
        ORDER BY c.relname, s.attname limit 2;
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
            });
        }
    }

    Ok(tables_map.into_values().collect())
}
