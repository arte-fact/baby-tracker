use chrono::NaiveDateTime;

use crate::models::{Dejection, DejectionType, Feeding, FeedingType, Weight};
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

    // --- Weight ---

    pub fn add_weight(
        &mut self,
        baby_name: &str,
        weight_kg: f64,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, String> {
        let ts = parse_timestamp(timestamp)?;
        let weight = Weight::new(baby_name.to_string(), weight_kg, notes, ts)?;
        Ok(self.store.add_weight(weight))
    }

    pub fn update_weight(
        &mut self,
        id: u64,
        weight_kg: f64,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, String> {
        let ts = parse_timestamp(timestamp)?;
        let updated = Weight::new("x".to_string(), weight_kg, notes, ts)?;
        Ok(self.store.update_weight(id, updated))
    }

    pub fn delete_weight(&mut self, id: u64) -> bool {
        self.store.delete_weight(id)
    }

    // --- Timeline ---

    pub fn timeline_for_day(&self, baby_name: Option<&str>, date: &str) -> Result<String, String> {
        let day_start = parse_timestamp(&format!("{}T00:00:00", date))?;
        let day_end = day_start + chrono::Duration::days(1);
        let entries = self.store.timeline_for_day(baby_name, day_start, day_end);
        Ok(serde_json::to_string(&entries).unwrap_or_else(|_| "[]".to_string()))
    }

    // --- Summary (day-bounded) ---

    pub fn get_summary(&self, baby_name: Option<&str>, date: &str) -> Result<String, String> {
        let since = parse_timestamp(&format!("{}T00:00:00", date))?;
        let until = since + chrono::Duration::days(1);
        let summary = self.store.summary(baby_name, since, until);
        Ok(serde_json::to_string(&summary).unwrap_or_else(|_| "{}".to_string()))
    }

    // --- Report (date range) ---

    pub fn report(&self, baby_name: Option<&str>, start_date: &str, end_date: &str) -> Result<String, String> {
        let start = parse_timestamp(&format!("{}T00:00:00", start_date))?;
        let end = parse_timestamp(&format!("{}T00:00:00", end_date))?;
        let reports = self.store.report(baby_name, start, end);
        Ok(serde_json::to_string(&reports).unwrap_or_else(|_| "[]".to_string()))
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

    // --- Weight ---

    #[test]
    fn add_weight() {
        let mut t = Tracker::new();
        let id = t.add_weight("Emma", 3.5, None, "2026-02-15T08:00:00").unwrap();
        assert_eq!(id, 1);
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("weight"));
        assert!(json.contains("3.5"));
    }

    #[test]
    fn add_weight_validates() {
        let mut t = Tracker::new();
        assert!(t.add_weight("", 3.5, None, "2026-02-15T08:00:00").is_err());
        assert!(t.add_weight("Emma", 0.0, None, "2026-02-15T08:00:00").is_err());
        assert!(t.add_weight("Emma", 3.5, None, "bad-date").is_err());
    }

    #[test]
    fn update_weight() {
        let mut t = Tracker::new();
        let id = t.add_weight("Emma", 3.5, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.update_weight(id, 4.0, Some("Grew!".to_string()), "2026-02-15T10:00:00").unwrap());
        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(json.contains("4.0"));
        assert!(json.contains("Grew!"));
    }

    #[test]
    fn delete_weight() {
        let mut t = Tracker::new();
        let id = t.add_weight("Emma", 3.5, None, "2026-02-15T08:00:00").unwrap();
        assert!(t.delete_weight(id));
        assert!(!t.delete_weight(id));
    }

    // --- Timeline ---

    #[test]
    fn timeline_merges_all_types() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T09:00:00").unwrap();
        t.add_weight("Emma", 3.5, None, "2026-02-15T10:00:00").unwrap();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T11:00:00").unwrap();

        let json = t.timeline_for_day(None, "2026-02-15").unwrap();
        let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        assert_eq!(entries.len(), 4);
        assert_eq!(entries[0]["kind"], "feeding");
        assert_eq!(entries[1]["kind"], "dejection");
        assert_eq!(entries[2]["kind"], "weight");
        assert_eq!(entries[3]["kind"], "feeding");
    }

    #[test]
    fn export_and_load_with_all_types() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T09:00:00").unwrap();
        t.add_weight("Emma", 3.5, None, "2026-02-15T10:00:00").unwrap();

        let json = t.export_data();
        let restored = Tracker::from_json(&json).unwrap();
        let tl = restored.timeline_for_day(None, "2026-02-15").unwrap();
        assert!(tl.contains("feeding"));
        assert!(tl.contains("dejection"));
        assert!(tl.contains("weight"));
    }

    // --- Summary (day-bounded) ---

    #[test]
    fn summary_is_day_bounded() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(100.0), None, None, "2026-02-14T20:00:00").unwrap();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-15T08:00:00").unwrap();
        t.add_dejection("Emma", "urine", None, "2026-02-15T09:00:00").unwrap();
        t.add_dejection("Emma", "poop", None, "2026-02-15T10:00:00").unwrap();
        t.add_weight("Emma", 3.5, None, "2026-02-15T11:00:00").unwrap();
        t.add_feeding("Emma", "bottle", Some(90.0), None, None, "2026-02-16T06:00:00").unwrap();

        let s = t.get_summary(None, "2026-02-15").unwrap();
        assert!(s.contains("\"total_feedings\":1"));
        assert!(s.contains("\"total_ml\":120"));
        assert!(s.contains("\"total_urine\":1"));
        assert!(s.contains("\"total_poop\":1"));
        assert!(s.contains("\"latest_weight_kg\":3.5"));
    }

    // --- Report ---

    #[test]
    fn report_returns_per_day_data() {
        let mut t = Tracker::new();
        t.add_feeding("Emma", "bottle", Some(120.0), None, None, "2026-02-14T08:00:00").unwrap();
        t.add_feeding("Emma", "bl", None, Some(15), None, "2026-02-15T10:00:00").unwrap();

        let r = t.report(None, "2026-02-14", "2026-02-16").unwrap();
        let days: Vec<serde_json::Value> = serde_json::from_str(&r).unwrap();
        assert_eq!(days.len(), 2);
        assert_eq!(days[0]["date"], "2026-02-14");
        assert_eq!(days[0]["total_feedings"], 1);
        assert_eq!(days[0]["total_ml"], 120.0);
        assert_eq!(days[1]["date"], "2026-02-15");
        assert_eq!(days[1]["total_feedings"], 1);
        assert_eq!(days[1]["total_minutes"], 15);
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
