use chrono::NaiveDateTime;

use crate::models::{Feeding, FeedingType};
use crate::store::Store;

pub struct Tracker {
    store: Store,
}

impl Tracker {
    pub fn new() -> Self {
        Tracker {
            store: Store::new(),
        }
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        Ok(Tracker {
            store: Store::from_json(json)?,
        })
    }

    pub fn export_data(&self) -> String {
        self.store.to_json()
    }

    pub fn add_feeding(
        &mut self,
        baby_name: &str,
        feeding_type: &str,
        amount_ml: Option<f64>,
        duration_minutes: Option<u32>,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, String> {
        let ft = FeedingType::parse(feeding_type)?;
        let ts = parse_timestamp(timestamp)?;
        let feeding = Feeding::new(baby_name.to_string(), ft, amount_ml, duration_minutes, notes, ts)?;
        Ok(self.store.add(feeding))
    }

    pub fn delete_feeding(&mut self, id: u64) -> bool {
        self.store.delete(id)
    }

    pub fn list_feedings(&self, baby_name: Option<&str>, limit: usize) -> String {
        let feedings = self.store.list(baby_name, limit);
        serde_json::to_string(&feedings).unwrap_or_else(|_| "[]".to_string())
    }

    /// List feedings for a single day (date string "YYYY-MM-DD"), returned in chronological order.
    pub fn list_feedings_for_day(&self, baby_name: Option<&str>, date: &str) -> Result<String, String> {
        let day_start = parse_timestamp(&format!("{}T00:00:00", date))?;
        let day_end = day_start + chrono::Duration::days(1);
        let feedings = self.store.list_day(baby_name, day_start, day_end);
        Ok(serde_json::to_string(&feedings).unwrap_or_else(|_| "[]".to_string()))
    }

    pub fn get_summary(&self, baby_name: Option<&str>, since: &str) -> Result<String, String> {
        let ts = parse_timestamp(since)?;
        let summary = self.store.summary(baby_name, ts);
        Ok(serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string()))
    }
}

pub fn parse_timestamp(s: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M"))
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
        .or_else(|_| NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M"))
        .map_err(|_| format!("Invalid timestamp: '{}'. Use YYYY-MM-DDTHH:MM:SS", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_list() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        assert_eq!(id, 1);

        let json = t.list_feedings(None, 10);
        assert!(json.contains("Emma"));
        assert!(json.contains("120"));
    }

    #[test]
    fn add_validates_type() {
        let mut t = Tracker::new();
        assert!(t.add_feeding("Emma", "juice", None, None, None, "2026-02-15T08:00:00").is_err());
    }

    #[test]
    fn add_validates_name() {
        let mut t = Tracker::new();
        assert!(t.add_feeding("", "bottle", None, None, None, "2026-02-15T08:00:00").is_err());
    }

    #[test]
    fn add_validates_timestamp() {
        let mut t = Tracker::new();
        assert!(t.add_feeding("Emma", "bottle", None, None, None, "not-a-date").is_err());
    }

    #[test]
    fn delete() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", None, None, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.delete_feeding(id));
        assert!(!t.delete_feeding(id));
    }

    #[test]
    fn export_and_load() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bl", None, Some(15), Some("test".to_string()), "2026-02-15T08:00:00").unwrap();
        t.add_feeding("Noah", "bottle", Some(90.0), None, None, "2026-02-15T09:00:00").unwrap();

        let json = t.export_data();
        let restored = Tracker::from_json(&json).unwrap();
        let list = restored.list_feedings(None, 100);
        assert!(list.contains("Emma"));
        assert!(list.contains("Noah"));
    }

    #[test]
    fn summary() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        t.add_feeding("Emma", "breast-left", None, Some(15), None, "2026-02-15T10:00:00").unwrap();

        let s = t.get_summary(Some("Emma"), "2026-02-15T00:00:00").unwrap();
        assert!(s.contains("\"total_feedings\":2"));
        assert!(s.contains("120"));
    }

    #[test]
    fn list_feedings_for_day() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-14T22:00:00").unwrap();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T08:00:00").unwrap();
        t.add_feeding("Emma", "bottle", Some(90.0), None, None, "2026-02-15T14:00:00").unwrap();
        t.add_feeding("Emma", "solid", None, None, None, "2026-02-16T07:00:00").unwrap();

        let json = t.list_feedings_for_day(None, "2026-02-15").unwrap();
        let feedings: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(feedings.len(), 2);
        // chronological order
        assert!(feedings[0]["timestamp"].as_str().unwrap() < feedings[1]["timestamp"].as_str().unwrap());
    }

    #[test]
    fn parse_various_formats() {
        assert!(parse_timestamp("2026-02-15T08:00:00").is_ok());
        assert!(parse_timestamp("2026-02-15T08:00").is_ok());
        assert!(parse_timestamp("2026-02-15 08:00:00").is_ok());
        assert!(parse_timestamp("2026-02-15 08:00").is_ok());
        assert!(parse_timestamp("bad").is_err());
    }
}
