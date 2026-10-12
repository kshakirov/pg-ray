use tokio_postgres::{Client, Error};

/// PostgreSQL inputs used for the disk component of a sequential scan cost.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeqScanDiskCost {
    pub relation_pages: f64,
    pub seq_page_cost: f64,
}

impl SeqScanDiskCost {
    /// Calculate the disk component in PostgreSQL planner cost units.
    pub fn total(self) -> f64 {
        self.relation_pages * self.seq_page_cost
    }
}

/// Read the current PostgreSQL inputs for the disk component of a Seq Scan.
///
/// The relation is resolved by schema and table name without interpolating
/// either value into SQL. `relation_pages` is PostgreSQL's `pg_class.relpages`
/// estimate; it is not a live count of physical reads performed by a query.
pub async fn fetch_seq_scan_disk_cost(
    client: &Client,
    schema: &str,
    table: &str,
) -> Result<SeqScanDiskCost, Error> {
    let row = client
        .query_one(
            "
            SELECT
                c.relpages::double precision AS relation_pages,
                current_setting('seq_page_cost')::double precision AS seq_page_cost
            FROM pg_catalog.pg_class AS c
            JOIN pg_catalog.pg_namespace AS n ON n.oid = c.relnamespace
            WHERE n.nspname = $1
              AND c.relname = $2
            ",
            &[&schema, &table],
        )
        .await?;

    Ok(SeqScanDiskCost {
        relation_pages: row.try_get("relation_pages")?,
        seq_page_cost: row.try_get("seq_page_cost")?,
    })
}

#[cfg(test)]
mod tests {
    use super::SeqScanDiskCost;

    #[test]
    fn calculates_seq_scan_disk_cost() {
        let inputs = SeqScanDiskCost {
            relation_pages: 8_608.0,
            seq_page_cost: 1.0,
        };

        assert_eq!(inputs.total(), 8_608.0);
    }
}
