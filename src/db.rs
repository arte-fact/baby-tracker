use std::path::PathBuf;

use chrono::NaiveDateTime;
use rusqlite::{params, Connection, Result};

use crate::models::{Feeding, FeedingType};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &PathBuf) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS feedings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                baby_name TEXT NOT NULL,
                feeding_type TEXT NOT NULL,
                amount_ml REAL,
                duration_minutes INTEGER,
                notes TEXT,
                timestamp TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn add_feeding(&self, feeding: &Feeding) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO feedings (baby_name, feeding_type, amount_ml, duration_minutes, notes, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                feeding.baby_name,
                feeding.feeding_type.to_db_str(),
                feeding.amount_ml,
                feeding.duration_minutes,
                feeding.notes,
                feeding.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_feedings(
        &self,
        baby_name: Option<&str>,
        limit: usize,
    ) -> Result<Vec<Feeding>> {
        let (sql, baby_filter): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match baby_name {
            Some(name) => (
                format!(
                    "SELECT id, baby_name, feeding_type, amount_ml, duration_minutes, notes, timestamp
                     FROM feedings WHERE baby_name = ?1 ORDER BY timestamp DESC LIMIT {}",
                    limit
                ),
                vec![Box::new(name.to_string())],
            ),
            None => (
                format!(
                    "SELECT id, baby_name, feeding_type, amount_ml, duration_minutes, notes, timestamp
                     FROM feedings ORDER BY timestamp DESC LIMIT {}",
                    limit
                ),
                vec![],
            ),
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(baby_filter.iter()), |row| {
            let ts_str: String = row.get(6)?;
            let timestamp = NaiveDateTime::parse_from_str(&ts_str, "%Y-%m-%d %H:%M:%S")
                .unwrap_or_default();
            let ft_str: String = row.get(2)?;
            Ok(Feeding {
                id: row.get(0)?,
                baby_name: row.get(1)?,
                feeding_type: FeedingType::from_db_str(&ft_str),
                amount_ml: row.get(3)?,
                duration_minutes: row.get(4)?,
                notes: row.get(5)?,
                timestamp,
            })
        })?;

        let mut feedings = Vec::new();
        for row in rows {
            feedings.push(row?);
        }
        Ok(feedings)
    }

    pub fn delete_feeding(&self, id: i64) -> Result<bool> {
        let count = self.conn.execute("DELETE FROM feedings WHERE id = ?1", params![id])?;
        Ok(count > 0)
    }

    pub fn get_summary(
        &self,
        baby_name: Option<&str>,
        days: i64,
    ) -> Result<Summary> {
        let since = chrono::Local::now().naive_local() - chrono::Duration::days(days);
        let since_str = since.format("%Y-%m-%d %H:%M:%S").to_string();

        let (where_clause, filter_params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) =
            match baby_name {
                Some(name) => (
                    "WHERE baby_name = ?1 AND timestamp >= ?2".to_string(),
                    vec![Box::new(name.to_string()), Box::new(since_str.clone())],
                ),
                None => (
                    "WHERE timestamp >= ?1".to_string(),
                    vec![Box::new(since_str.clone())],
                ),
            };

        let sql = format!(
            "SELECT COUNT(*), COALESCE(SUM(amount_ml), 0), COALESCE(SUM(duration_minutes), 0)
             FROM feedings {}",
            where_clause
        );

        let mut stmt = self.conn.prepare(&sql)?;
        let (total_feedings, total_ml, total_minutes): (i64, f64, i64) =
            stmt.query_row(rusqlite::params_from_iter(filter_params.iter()), |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;

        // Count by type
        let type_sql = format!(
            "SELECT feeding_type, COUNT(*) FROM feedings {} GROUP BY feeding_type",
            where_clause
        );

        let filter_params2: Vec<Box<dyn rusqlite::types::ToSql>> = match baby_name {
            Some(name) => vec![Box::new(name.to_string()), Box::new(since_str)],
            None => vec![Box::new(since.format("%Y-%m-%d %H:%M:%S").to_string())],
        };

        let mut stmt2 = self.conn.prepare(&type_sql)?;
        let type_rows =
            stmt2.query_map(rusqlite::params_from_iter(filter_params2.iter()), |row| {
                let ft: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((ft, count))
            })?;

        let mut by_type = Vec::new();
        for row in type_rows {
            let (ft, count) = row?;
            by_type.push((FeedingType::from_db_str(&ft), count));
        }

        Ok(Summary {
            days,
            total_feedings,
            total_ml,
            total_minutes,
            by_type,
        })
    }
}

pub struct Summary {
    pub days: i64,
    pub total_feedings: i64,
    pub total_ml: f64,
    pub total_minutes: i64,
    pub by_type: Vec<(FeedingType, i64)>,
}
