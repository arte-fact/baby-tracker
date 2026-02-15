use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::{Feeding, FeedingType};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    feedings: Vec<Feeding>,
    next_id: u64,
}

impl Store {
    pub fn new() -> Self {
        Store {
            feedings: Vec::new(),
            next_id: 1,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid data: {}", e))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Store serialization should never fail")
    }

    pub fn add(&mut self, mut feeding: Feeding) -> u64 {
        feeding.id = self.next_id;
        self.next_id += 1;
        let id = feeding.id;
        self.feedings.push(feeding);
        id
    }

    pub fn delete(&mut self, id: u64) -> bool {
        let before = self.feedings.len();
        self.feedings.retain(|f| f.id != id);
        self.feedings.len() < before
    }

    pub fn list(&self, baby_name: Option<&str>, limit: usize) -> Vec<&Feeding> {
        let mut result: Vec<&Feeding> = self
            .feedings
            .iter()
            .filter(|f| baby_name.map_or(true, |name| f.baby_name == name))
            .collect();
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        result.truncate(limit);
        result
    }

    pub fn summary(&self, baby_name: Option<&str>, since: NaiveDateTime) -> Summary {
        let filtered: Vec<&Feeding> = self
            .feedings
            .iter()
            .filter(|f| {
                f.timestamp >= since && baby_name.map_or(true, |name| f.baby_name == name)
            })
            .collect();

        let total_feedings = filtered.len() as u64;
        let total_ml: f64 = filtered.iter().filter_map(|f| f.amount_ml).sum();
        let total_minutes: u32 = filtered.iter().filter_map(|f| f.duration_minutes).sum();

        let mut by_type: Vec<(FeedingType, u64)> = Vec::new();
        for ft in &[
            FeedingType::BreastLeft,
            FeedingType::BreastRight,
            FeedingType::Bottle,
            FeedingType::Solid,
        ] {
            let count = filtered.iter().filter(|f| f.feeding_type == *ft).count() as u64;
            if count > 0 {
                by_type.push((ft.clone(), count));
            }
        }

        Summary {
            total_feedings,
            total_ml,
            total_minutes,
            by_type,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_feedings: u64,
    pub total_ml: f64,
    pub total_minutes: u32,
    pub by_type: Vec<(FeedingType, u64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Feeding, FeedingType};
    use chrono::NaiveDate;

    fn ts(day: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, day)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    fn make_feeding(name: &str, ft: FeedingType, ml: Option<f64>, dur: Option<u32>, day: u32, h: u32) -> Feeding {
        Feeding::new(name.to_string(), ft, ml, dur, None, ts(day, h, 0)).unwrap()
    }

    // --- Store basics ---

    #[test]
    fn new_store_is_empty() {
        let store = Store::new();
        assert_eq!(store.list(None, 100).len(), 0);
    }

    #[test]
    fn add_assigns_incrementing_ids() {
        let mut store = Store::new();
        let id1 = store.add(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        let id2 = store.add(make_feeding("Emma", FeedingType::Bottle, Some(90.0), None, 15, 12));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn list_returns_all_in_reverse_chronological() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 14));
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 11));

        let list = store.list(None, 100);
        assert_eq!(list.len(), 3);
        assert!(list[0].timestamp > list[1].timestamp);
        assert!(list[1].timestamp > list[2].timestamp);
    }

    #[test]
    fn list_filters_by_baby_name() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add(make_feeding("Noah", FeedingType::Bottle, None, None, 15, 9));
        store.add(make_feeding("Emma", FeedingType::Solid, None, None, 15, 10));

        let emma = store.list(Some("Emma"), 100);
        assert_eq!(emma.len(), 2);
        assert!(emma.iter().all(|f| f.baby_name == "Emma"));

        let noah = store.list(Some("Noah"), 100);
        assert_eq!(noah.len(), 1);
    }

    #[test]
    fn list_respects_limit() {
        let mut store = Store::new();
        for h in 0..10 {
            store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, h));
        }
        assert_eq!(store.list(None, 3).len(), 3);
    }

    #[test]
    fn delete_existing_returns_true() {
        let mut store = Store::new();
        let id = store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        assert!(store.delete(id));
        assert_eq!(store.list(None, 100).len(), 0);
    }

    #[test]
    fn delete_nonexistent_returns_false() {
        let mut store = Store::new();
        assert!(!store.delete(999));
    }

    #[test]
    fn delete_only_removes_target() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        let id2 = store.add(make_feeding("Emma", FeedingType::Solid, None, None, 15, 10));
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 12));

        store.delete(id2);
        let list = store.list(None, 100);
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|f| f.id != id2));
    }

    // --- JSON persistence ---

    #[test]
    fn json_roundtrip_preserves_data() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::BreastLeft, None, Some(15), 15, 8));
        store.add(make_feeding("Noah", FeedingType::Bottle, Some(120.0), None, 15, 9));

        let json = store.to_json();
        let restored = Store::from_json(&json).unwrap();

        let original_list = store.list(None, 100);
        let restored_list = restored.list(None, 100);
        assert_eq!(original_list.len(), restored_list.len());
        assert_eq!(restored_list[0].baby_name, original_list[0].baby_name);
        assert_eq!(restored_list[1].baby_name, original_list[1].baby_name);
    }

    #[test]
    fn json_roundtrip_preserves_next_id() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 9));

        let json = store.to_json();
        let mut restored = Store::from_json(&json).unwrap();
        let id3 = restored.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 10));
        assert_eq!(id3, 3);
    }

    #[test]
    fn from_json_invalid_returns_error() {
        assert!(Store::from_json("not json").is_err());
    }

    // --- Summary ---

    #[test]
    fn summary_counts_feedings() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        store.add(make_feeding("Emma", FeedingType::BreastLeft, None, Some(15), 15, 10));
        store.add(make_feeding("Emma", FeedingType::Solid, None, None, 15, 12));

        let s = store.summary(None, ts(15, 0, 0));
        assert_eq!(s.total_feedings, 3);
        assert_eq!(s.total_ml, 120.0);
        assert_eq!(s.total_minutes, 15);
    }

    #[test]
    fn summary_filters_by_since() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, Some(100.0), None, 14, 8));
        store.add(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));

        let s = store.summary(None, ts(15, 0, 0));
        assert_eq!(s.total_feedings, 1);
        assert_eq!(s.total_ml, 120.0);
    }

    #[test]
    fn summary_filters_by_baby_name() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        store.add(make_feeding("Noah", FeedingType::Bottle, Some(90.0), None, 15, 9));

        let s = store.summary(Some("Noah"), ts(15, 0, 0));
        assert_eq!(s.total_feedings, 1);
        assert_eq!(s.total_ml, 90.0);
    }

    #[test]
    fn summary_by_type_breakdown() {
        let mut store = Store::new();
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 10));
        store.add(make_feeding("Emma", FeedingType::BreastLeft, None, None, 15, 12));

        let s = store.summary(None, ts(15, 0, 0));
        assert_eq!(s.by_type.len(), 2);

        let bottle_count = s.by_type.iter().find(|(ft, _)| *ft == FeedingType::Bottle).unwrap().1;
        assert_eq!(bottle_count, 2);

        let bl_count = s.by_type.iter().find(|(ft, _)| *ft == FeedingType::BreastLeft).unwrap().1;
        assert_eq!(bl_count, 1);
    }

    #[test]
    fn summary_empty_store() {
        let store = Store::new();
        let s = store.summary(None, ts(15, 0, 0));
        assert_eq!(s.total_feedings, 0);
        assert_eq!(s.total_ml, 0.0);
        assert_eq!(s.total_minutes, 0);
        assert!(s.by_type.is_empty());
    }
}
