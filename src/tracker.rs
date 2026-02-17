use chrono::NaiveDateTime;

use crate::models::{Dejection, DejectionType, Feeding, FeedingType};
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

    // --- Feeding ---

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
        Ok(self.store.add_feeding(feeding))
    }

    pub fn update_feeding(
        &mut self,
        id: u64,
        feeding_type: &str,
        amount_ml: Option<f64>,
        duration_minutes: Option<u32>,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, String> {
        let ft = FeedingType::parse(feeding_type)?;
        let ts = parse_timestamp(timestamp)?;
        let updated = Feeding::new("x".to_string(), ft, amount_ml, duration_minutes, notes, ts)?;
        Ok(self.store.update_feeding(id, updated))
    }

    pub fn delete_feeding(&mut self, id: u64) -> bool {
        self.store.delete_feeding(id)
    }

    // --- Dejection ---

    pub fn add_dejection(
        &mut self,
        baby_name: &str,
        dejection_type: &str,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, String> {
        let dt = DejectionType::parse(dejection_type)?;
        let ts = parse_timestamp(timestamp)?;
        let dejection = Dejection::new(baby_name.to_string(), dt, notes, ts)?;
        Ok(self.store.add_dejection(dejection))
    }

    pub fn update_dejection(
        &mut self,
        id: u64,
        dejection_type: &str,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, String> {
        let dt = DejectionType::parse(dejection_type)?;
        let ts = parse_timestamp(timestamp)?;
        let updated = Dejection::new("x".to_string(), dt, notes, ts)?;
        Ok(self.store.update_dejection(id, updated))
    }

    pub fn delete_dejection(&mut self, id: u64) -> bool {
        self.store.delete_dejection(id)
    }

    // --- Timeline ---

    pub fn timeline_for_day(&self, baby_name: Option<&str>, date: &str) -> Result<String, String> {
        let day_start = parse_timestamp(&format!("{}T00:00:00", date))?;
        let day_end = day_start + chrono::Duration::days(1);
        let entries = self.store.timeline_for_day(baby_name, day_start, day_end);
        Ok(serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()))
    }

    // --- Summary ---

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
    fn add_and_list_feeding() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        assert_eq!(id, 1);
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("bottle"));
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
    fn delete_feeding() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", None, None, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.delete_feeding(id));
        assert!(!t.delete_feeding(id));
    }

    #[test]
    fn update_feeding() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", Some(100.0), None, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.update_feeding(id, "solid", Some(200.0), Some(5), Some("Edited".to_string()), "2026-02-15T09:00:00").unwrap());
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("solid"));
        assert!(json.contains("200"));
        assert!(json.contains("Edited"));
    }

    #[test]
    fn update_feeding_invalid_type() {
        let mut t = Tracker::new();
        let id = t.add_feeding("Emma", "bottle", None, None, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.update_feeding(id, "juice", None, None, None, "2026-02-15T08:00:00").is_err());
    }

    // --- Dejections ---

    #[test]
    fn add_dejection() {
        let mut t = Tracker::new();
        let id = t.add_dejection("Emma", "poop", Some("Soft".to_string()), "2026-02-15T10:00:00").unwrap();
        assert_eq!(id, 1);
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("dejection"));
        assert!(json.contains("poop"));
    }

    #[test]
    fn add_dejection_validates_type() {
        let mut t = Tracker::new();
        assert!(t.add_dejection("Emma", "vomit", None, "2026-02-15T10:00:00").is_err());
    }

    #[test]
    fn delete_dejection() {
        let mut t = Tracker::new();
        let id = t.add_dejection("Emma", "urine", None, "2026-02-15T10:00:00").unwrap();
        assert!(t.delete_dejection(id));
        assert!(!t.delete_dejection(id));
    }

    #[test]
    fn update_dejection() {
        let mut t = Tracker::new();
        let id = t.add_dejection("Emma", "urine", None, "2026-02-15T10:00:00").unwrap();
        assert!(t.update_dejection(id, "poop", Some("Changed".to_string()), "2026-02-15T11:00:00").unwrap());
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("poop"));
        assert!(json.contains("Changed"));
    }

    // --- Timeline ---

    #[test]
    fn timeline_merges_types() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T09:00:00").unwrap();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T10:00:00").unwrap();

        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0]["kind"], "feeding");
        assert_eq!(entries[1]["kind"], "dejection");
        assert_eq!(entries[2]["kind"], "feeding");
    }

    #[test]
    fn export_and_load_with_dejections() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T09:00:00").unwrap();

        let json = t.export_data();
        let restored = Tracker::from_json(&json).unwrap();
        let tl = restored.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(tl.contains("feeding"));
        assert!(tl.contains("dejection"));
    }

    #[test]
    fn summary_with_dejections() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "urine", None, "2026-02-15T09:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T10:00:00").unwrap();

        let s = t.get_summary(None, "2026-02-15T00:00:00").unwrap();
        assert!(s.contains("\"total_feedings\":1"));
        assert!(s.contains("\"total_urine\":1"));
        assert!(s.contains("\"total_poop\":1"));
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
