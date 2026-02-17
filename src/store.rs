use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::models::{Dejection, DejectionType, Feeding, FeedingType, TimelineEntry, Weight};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    feedings: Vec<Feeding>,
    #[serde(default)]
    dejections: Vec<Dejection>,
    #[serde(default)]
    weights: Vec<Weight>,
    next_id: u32,
}

impl Store {
    pub fn new() -> Self {
        Store {
            feedings: Vec::new(),
            dejections: Vec::new(),
            weights: Vec::new(),
            next_id: 1,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid data: {}", e))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Store serialization should never fail")
    }

    // --- Feeding CRUD ---

    pub fn add_feeding(&mut self, mut feeding: Feeding) -> u32 {
        feeding.id = self.next_id;
        self.next_id += 1;
        let id = feeding.id;
        self.feedings.push(feeding);
        id
    }

    pub fn delete_feeding(&mut self, id: u32) -> bool {
        let before = self.feedings.len();
        self.feedings.retain(|f| f.id != id);
        self.feedings.len() < before
    }

    pub fn update_feeding(&mut self, id: u32, updated: Feeding) -> bool {
        if let Some(f) = self.feedings.iter_mut().find(|f| f.id == id) {
            f.feeding_type = updated.feeding_type;
            f.amount_ml = updated.amount_ml;
            f.duration_minutes = updated.duration_minutes;
            f.notes = updated.notes;
            f.timestamp = updated.timestamp;
            true
        } else {
            false
        }
    }

    pub fn list_feedings(&self, baby_name: Option<&str>, limit: usize) -> Vec<&Feeding> {
        let mut result: Vec<&Feeding> = self
            .feedings
            .iter()
            .filter(|f| baby_name.map_or(true, |name| f.baby_name == name))
            .collect();
        result.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        result.truncate(limit);
        result
    }

    // --- Dejection CRUD ---

    pub fn add_dejection(&mut self, mut dejection: Dejection) -> u32 {
        dejection.id = self.next_id;
        self.next_id += 1;
        let id = dejection.id;
        self.dejections.push(dejection);
        id
    }

    pub fn delete_dejection(&mut self, id: u32) -> bool {
        let before = self.dejections.len();
        self.dejections.retain(|d| d.id != id);
        self.dejections.len() < before
    }

    pub fn update_dejection(&mut self, id: u32, updated: Dejection) -> bool {
        if let Some(d) = self.dejections.iter_mut().find(|d| d.id == id) {
            d.dejection_type = updated.dejection_type;
            d.notes = updated.notes;
            d.timestamp = updated.timestamp;
            true
        } else {
            false
        }
    }

    // --- Weight CRUD ---

    pub fn add_weight(&mut self, mut weight: Weight) -> u32 {
        weight.id = self.next_id;
        self.next_id += 1;
        let id = weight.id;
        self.weights.push(weight);
        id
    }

    pub fn delete_weight(&mut self, id: u32) -> bool {
        let before = self.weights.len();
        self.weights.retain(|w| w.id != id);
        self.weights.len() < before
    }

    pub fn update_weight(&mut self, id: u32, updated: Weight) -> bool {
        if let Some(w) = self.weights.iter_mut().find(|w| w.id == id) {
            w.weight_kg = updated.weight_kg;
            w.notes = updated.notes;
            w.timestamp = updated.timestamp;
            true
        } else {
            false
        }
    }

    // --- Unified timeline ---

    pub fn timeline_for_day(
        &self,
        baby_name: Option<&str>,
        day_start: NaiveDateTime,
        day_end: NaiveDateTime,
    ) -> Vec<TimelineEntry> {
        let mut entries: Vec<TimelineEntry> = Vec::new();

        for f in &self.feedings {
            if f.timestamp >= day_start
                && f.timestamp < day_end
                && baby_name.map_or(true, |name| f.baby_name == name)
            {
                entries.push(TimelineEntry::from_feeding(f));
            }
        }

        for d in &self.dejections {
            if d.timestamp >= day_start
                && d.timestamp < day_end
                && baby_name.map_or(true, |name| d.baby_name == name)
            {
                entries.push(TimelineEntry::from_dejection(d));
            }
        }

        for w in &self.weights {
            if w.timestamp >= day_start
                && w.timestamp < day_end
                && baby_name.map_or(true, |name| w.baby_name == name)
            {
                entries.push(TimelineEntry::from_weight(w));
            }
        }

        entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        entries
    }

    // --- Summary (bounded by since..until) ---

    pub fn summary(
        &self,
        baby_name: Option<&str>,
        since: NaiveDateTime,
        until: NaiveDateTime,
    ) -> Summary {
        let in_range = |ts: NaiveDateTime| ts >= since && ts < until;

        let filtered: Vec<&Feeding> = self
            .feedings
            .iter()
            .filter(|f| in_range(f.timestamp) && baby_name.map_or(true, |name| f.baby_name == name))
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

        let dejection_filtered: Vec<&Dejection> = self
            .dejections
            .iter()
            .filter(|d| in_range(d.timestamp) && baby_name.map_or(true, |name| d.baby_name == name))
            .collect();

        let total_urine = dejection_filtered
            .iter()
            .filter(|d| d.dejection_type == DejectionType::Urine)
            .count() as u64;
        let total_poop = dejection_filtered
            .iter()
            .filter(|d| d.dejection_type == DejectionType::Poop)
            .count() as u64;

        let latest_weight_kg = self
            .weights
            .iter()
            .filter(|w| in_range(w.timestamp) && baby_name.map_or(true, |name| w.baby_name == name))
            .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
            .map(|w| w.weight_kg);

        Summary {
            total_feedings,
            total_ml,
            total_minutes,
            by_type,
            total_urine,
            total_poop,
            latest_weight_kg,
        }
    }

    // --- Report (per-day aggregates for a date range) ---

    pub fn report(
        &self,
        baby_name: Option<&str>,
        start: NaiveDateTime,
        end: NaiveDateTime,
    ) -> Vec<DayReport> {
        let mut reports = Vec::new();
        let mut day = start;
        while day < end {
            let next = day + chrono::Duration::days(1);
            let date_str = day.format("%Y-%m-%d").to_string();

            let name_matches = |n: &str| baby_name.map_or(true, |name| n == name);
            let in_day = |ts: NaiveDateTime| ts >= day && ts < next;

            let feedings: Vec<&Feeding> = self
                .feedings
                .iter()
                .filter(|f| in_day(f.timestamp) && name_matches(&f.baby_name))
                .collect();

            let total_feedings = feedings.len() as u64;
            let total_ml: f64 = feedings.iter().filter_map(|f| f.amount_ml).sum();
            let total_minutes: u32 = feedings.iter().filter_map(|f| f.duration_minutes).sum();
            let breast_left = feedings.iter().filter(|f| f.feeding_type == FeedingType::BreastLeft).count() as u64;
            let breast_right = feedings.iter().filter(|f| f.feeding_type == FeedingType::BreastRight).count() as u64;
            let bottle = feedings.iter().filter(|f| f.feeding_type == FeedingType::Bottle).count() as u64;
            let solid = feedings.iter().filter(|f| f.feeding_type == FeedingType::Solid).count() as u64;

            let dejections: Vec<&Dejection> = self
                .dejections
                .iter()
                .filter(|d| in_day(d.timestamp) && name_matches(&d.baby_name))
                .collect();
            let total_urine = dejections.iter().filter(|d| d.dejection_type == DejectionType::Urine).count() as u64;
            let total_poop = dejections.iter().filter(|d| d.dejection_type == DejectionType::Poop).count() as u64;

            let weight_kg = self
                .weights
                .iter()
                .filter(|w| in_day(w.timestamp) && name_matches(&w.baby_name))
                .max_by(|a, b| a.timestamp.cmp(&b.timestamp))
                .map(|w| w.weight_kg);

            reports.push(DayReport {
                date: date_str,
                total_feedings,
                total_ml,
                total_minutes,
                breast_left,
                breast_right,
                bottle,
                solid,
                total_urine,
                total_poop,
                weight_kg,
            });

            day = next;
        }
        reports
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub total_feedings: u64,
    pub total_ml: f64,
    pub total_minutes: u32,
    pub by_type: Vec<(FeedingType, u64)>,
    pub total_urine: u64,
    pub total_poop: u64,
    pub latest_weight_kg: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DayReport {
    pub date: String,
    pub total_feedings: u64,
    pub total_ml: f64,
    pub total_minutes: u32,
    pub breast_left: u64,
    pub breast_right: u64,
    pub bottle: u64,
    pub solid: u64,
    pub total_urine: u64,
    pub total_poop: u64,
    pub weight_kg: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Dejection, DejectionType, Feeding, FeedingType, Weight};
    use chrono::{NaiveDate, Timelike};

    fn ts(day: u32, h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, day)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    fn make_feeding(name: &str, ft: FeedingType, ml: Option<f64>, dur: Option<u32>, day: u32, h: u32) -> Feeding {
        Feeding::new(name.to_string(), ft, ml, dur, None, ts(day, h, 0)).unwrap()
    }

    fn make_dejection(name: &str, dt: DejectionType, day: u32, h: u32) -> Dejection {
        Dejection::new(name.to_string(), dt, None, ts(day, h, 0)).unwrap()
    }

    fn make_weight(name: &str, kg: f64, day: u32, h: u32) -> Weight {
        Weight::new(name.to_string(), kg, None, ts(day, h, 0)).unwrap()
    }

    // --- Feeding basics ---

    #[test]
    fn new_store_is_empty() {
        let store = Store::new();
        assert_eq!(store.list_feedings(None, 100).len(), 0);
    }

    #[test]
    fn add_assigns_incrementing_ids() {
        let mut store = Store::new();
        let id1 = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        let id2 = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(90.0), None, 15, 12));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn list_returns_all_in_reverse_chronological() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 14));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 11));
        let list = store.list_feedings(None, 100);
        assert_eq!(list.len(), 3);
        assert!(list[0].timestamp > list[1].timestamp);
        assert!(list[1].timestamp > list[2].timestamp);
    }

    #[test]
    fn list_filters_by_baby_name() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_feeding(make_feeding("Noah", FeedingType::Bottle, None, None, 15, 9));
        store.add_feeding(make_feeding("Emma", FeedingType::Solid, None, None, 15, 10));
        assert_eq!(store.list_feedings(Some("Emma"), 100).len(), 2);
        assert_eq!(store.list_feedings(Some("Noah"), 100).len(), 1);
    }

    #[test]
    fn list_respects_limit() {
        let mut store = Store::new();
        for h in 0..10 {
            store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, h));
        }
        assert_eq!(store.list_feedings(None, 3).len(), 3);
    }

    #[test]
    fn delete_feeding_existing() {
        let mut store = Store::new();
        let id = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        assert!(store.delete_feeding(id));
        assert_eq!(store.list_feedings(None, 100).len(), 0);
    }

    #[test]
    fn delete_feeding_nonexistent() {
        let mut store = Store::new();
        assert!(!store.delete_feeding(999));
    }

    #[test]
    fn delete_feeding_only_removes_target() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        let id2 = store.add_feeding(make_feeding("Emma", FeedingType::Solid, None, None, 15, 10));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 12));
        store.delete_feeding(id2);
        assert_eq!(store.list_feedings(None, 100).len(), 2);
    }

    // --- Update feeding ---

    #[test]
    fn update_feeding_changes_fields() {
        let mut store = Store::new();
        let id = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(100.0), None, 15, 8));
        let updated = Feeding::new("Emma".to_string(), FeedingType::Solid, Some(200.0), Some(10), Some("Edited".to_string()), ts(15, 9, 0)).unwrap();
        assert!(store.update_feeding(id, updated));
        let list = store.list_feedings(None, 100);
        assert_eq!(list[0].feeding_type, FeedingType::Solid);
        assert_eq!(list[0].amount_ml, Some(200.0));
        assert_eq!(list[0].duration_minutes, Some(10));
        assert_eq!(list[0].notes, Some("Edited".to_string()));
        assert_eq!(list[0].timestamp.hour(), 9);
    }

    #[test]
    fn update_feeding_nonexistent_returns_false() {
        let mut store = Store::new();
        let f = make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8);
        assert!(!store.update_feeding(999, f));
    }

    #[test]
    fn update_feeding_preserves_id_and_name() {
        let mut store = Store::new();
        let id = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        let updated = make_feeding("Someone", FeedingType::Solid, None, None, 15, 10);
        store.update_feeding(id, updated);
        let list = store.list_feedings(None, 100);
        assert_eq!(list[0].id, id);
        assert_eq!(list[0].baby_name, "Emma");
    }

    // --- Dejection CRUD ---

    #[test]
    fn add_dejection_assigns_id() {
        let mut store = Store::new();
        let id = store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 8));
        assert_eq!(id, 1);
    }

    #[test]
    fn feeding_and_dejection_share_id_counter() {
        let mut store = Store::new();
        let id1 = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        let id2 = store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 9));
        let id3 = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 10));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn delete_dejection() {
        let mut store = Store::new();
        let id = store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 8));
        assert!(store.delete_dejection(id));
        assert!(!store.delete_dejection(id));
    }

    #[test]
    fn update_dejection() {
        let mut store = Store::new();
        let id = store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 8));
        let updated = Dejection::new("Emma".to_string(), DejectionType::Poop, Some("Note".to_string()), ts(15, 9, 0)).unwrap();
        assert!(store.update_dejection(id, updated));
        let timeline = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(timeline[0].subtype, "poop");
        assert_eq!(timeline[0].notes, Some("Note".to_string()));
    }

    #[test]
    fn update_dejection_nonexistent() {
        let mut store = Store::new();
        let d = make_dejection("Emma", DejectionType::Urine, 15, 8);
        assert!(!store.update_dejection(999, d));
    }

    // --- Weight CRUD ---

    #[test]
    fn add_weight_assigns_id() {
        let mut store = Store::new();
        let id = store.add_weight(make_weight("Emma", 3.5, 15, 8));
        assert_eq!(id, 1);
    }

    #[test]
    fn weight_shares_id_counter() {
        let mut store = Store::new();
        let id1 = store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        let id2 = store.add_weight(make_weight("Emma", 3.5, 15, 9));
        let id3 = store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 10));
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn delete_weight() {
        let mut store = Store::new();
        let id = store.add_weight(make_weight("Emma", 3.5, 15, 8));
        assert!(store.delete_weight(id));
        assert!(!store.delete_weight(id));
    }

    #[test]
    fn update_weight() {
        let mut store = Store::new();
        let id = store.add_weight(make_weight("Emma", 3.5, 15, 8));
        let updated = Weight::new("Emma".to_string(), 4.0, Some("Gaining".to_string()), ts(15, 10, 0)).unwrap();
        assert!(store.update_weight(id, updated));
        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl[0].weight_kg, Some(4.0));
        assert_eq!(tl[0].notes, Some("Gaining".to_string()));
    }

    #[test]
    fn update_weight_preserves_name() {
        let mut store = Store::new();
        let id = store.add_weight(make_weight("Emma", 3.5, 15, 8));
        let updated = Weight::new("Someone".to_string(), 4.0, None, ts(15, 10, 0)).unwrap();
        store.update_weight(id, updated);
        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl[0].baby_name, "Emma");
    }

    #[test]
    fn update_weight_nonexistent() {
        let mut store = Store::new();
        let w = make_weight("Emma", 3.5, 15, 8);
        assert!(!store.update_weight(999, w));
    }

    // --- Unified timeline ---

    #[test]
    fn timeline_merges_feedings_and_dejections() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 9));
        store.add_feeding(make_feeding("Emma", FeedingType::BreastLeft, None, None, 15, 10));
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 11));

        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl.len(), 4);
        assert_eq!(tl[0].kind, "feeding");
        assert_eq!(tl[0].timestamp.hour(), 8);
        assert_eq!(tl[1].kind, "dejection");
        assert_eq!(tl[1].subtype, "urine");
        assert_eq!(tl[2].kind, "feeding");
        assert_eq!(tl[3].kind, "dejection");
        assert_eq!(tl[3].subtype, "poop");
    }

    #[test]
    fn timeline_includes_weights() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_weight(make_weight("Emma", 4.2, 15, 10));

        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl.len(), 2);
        assert_eq!(tl[1].kind, "weight");
        assert_eq!(tl[1].weight_kg, Some(4.2));
    }

    #[test]
    fn timeline_chronological_order() {
        let mut store = Store::new();
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 14));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 6));

        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert!(tl[0].timestamp < tl[1].timestamp);
        assert!(tl[1].timestamp < tl[2].timestamp);
    }

    #[test]
    fn timeline_filters_by_day() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 14, 20));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 8));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 16, 6));

        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl.len(), 1);
        assert_eq!(tl[0].kind, "dejection");
    }

    #[test]
    fn timeline_filters_by_name() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_dejection(make_dejection("Noah", DejectionType::Poop, 15, 9));

        let tl = store.timeline_for_day(Some("Emma"), ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl.len(), 1);
        assert_eq!(tl[0].baby_name, "Emma");
    }

    #[test]
    fn timeline_empty() {
        let store = Store::new();
        let tl = store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert!(tl.is_empty());
    }

    // --- JSON persistence ---

    #[test]
    fn json_roundtrip_preserves_data() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::BreastLeft, None, Some(15), 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 9));
        store.add_weight(make_weight("Emma", 3.5, 15, 10));

        let json = store.to_json();
        let restored = Store::from_json(&json).unwrap();
        let tl = restored.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(tl.len(), 3);
        assert_eq!(tl[0].kind, "feeding");
        assert_eq!(tl[1].kind, "dejection");
        assert_eq!(tl[2].kind, "weight");
    }

    #[test]
    fn json_roundtrip_preserves_next_id() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 9));

        let json = store.to_json();
        let mut restored = Store::from_json(&json).unwrap();
        let id3 = restored.add_feeding(make_feeding("Emma", FeedingType::Bottle, None, None, 15, 10));
        assert_eq!(id3, 3);
    }

    #[test]
    fn json_backwards_compat_no_dejections_field() {
        let json = r#"{"feedings":[],"next_id":1}"#;
        let store = Store::from_json(json).unwrap();
        assert!(store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0)).is_empty());
    }

    #[test]
    fn json_backwards_compat_no_weights_field() {
        let json = r#"{"feedings":[],"dejections":[],"next_id":1}"#;
        let store = Store::from_json(json).unwrap();
        assert!(store.timeline_for_day(None, ts(15, 0, 0), ts(16, 0, 0)).is_empty());
    }

    #[test]
    fn from_json_invalid_returns_error() {
        assert!(Store::from_json("not json").is_err());
    }

    // --- Summary (bounded) ---

    #[test]
    fn summary_includes_dejection_counts() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 9));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 15, 11));
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 13));

        let s = store.summary(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.total_feedings, 1);
        assert_eq!(s.total_urine, 2);
        assert_eq!(s.total_poop, 1);
    }

    #[test]
    fn summary_bounded_excludes_other_days() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(100.0), None, 14, 8));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(90.0), None, 16, 8));

        let s = store.summary(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.total_feedings, 1);
        assert_eq!(s.total_ml, 120.0);
    }

    #[test]
    fn summary_includes_latest_weight() {
        let mut store = Store::new();
        store.add_weight(make_weight("Emma", 3.5, 15, 8));
        store.add_weight(make_weight("Emma", 3.6, 15, 14));

        let s = store.summary(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.latest_weight_kg, Some(3.6));
    }

    #[test]
    fn summary_no_weight() {
        let store = Store::new();
        let s = store.summary(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.latest_weight_kg, None);
    }

    #[test]
    fn summary_empty_store() {
        let store = Store::new();
        let s = store.summary(None, ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.total_feedings, 0);
        assert_eq!(s.total_ml, 0.0);
        assert_eq!(s.total_minutes, 0);
        assert_eq!(s.total_urine, 0);
        assert_eq!(s.total_poop, 0);
        assert_eq!(s.latest_weight_kg, None);
    }

    #[test]
    fn summary_filters_by_name() {
        let mut store = Store::new();
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 8));
        store.add_dejection(make_dejection("Noah", DejectionType::Poop, 15, 9));

        let s = store.summary(Some("Emma"), ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(s.total_poop, 1);
    }

    // --- Report ---

    #[test]
    fn report_aggregates_per_day() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 14, 8));
        store.add_feeding(make_feeding("Emma", FeedingType::BreastLeft, None, Some(15), 14, 12));
        store.add_dejection(make_dejection("Emma", DejectionType::Urine, 14, 10));
        store.add_weight(make_weight("Emma", 3.5, 14, 9));

        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(90.0), None, 15, 8));
        store.add_dejection(make_dejection("Emma", DejectionType::Poop, 15, 10));

        let r = store.report(None, ts(14, 0, 0), ts(16, 0, 0));
        assert_eq!(r.len(), 2);

        assert_eq!(r[0].date, "2026-02-14");
        assert_eq!(r[0].total_feedings, 2);
        assert_eq!(r[0].total_ml, 120.0);
        assert_eq!(r[0].total_minutes, 15);
        assert_eq!(r[0].bottle, 1);
        assert_eq!(r[0].breast_left, 1);
        assert_eq!(r[0].total_urine, 1);
        assert_eq!(r[0].weight_kg, Some(3.5));

        assert_eq!(r[1].date, "2026-02-15");
        assert_eq!(r[1].total_feedings, 1);
        assert_eq!(r[1].total_ml, 90.0);
        assert_eq!(r[1].total_poop, 1);
        assert_eq!(r[1].weight_kg, None);
    }

    #[test]
    fn report_empty_days() {
        let store = Store::new();
        let r = store.report(None, ts(14, 0, 0), ts(16, 0, 0));
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].total_feedings, 0);
        assert_eq!(r[1].total_feedings, 0);
    }

    #[test]
    fn report_filters_by_name() {
        let mut store = Store::new();
        store.add_feeding(make_feeding("Emma", FeedingType::Bottle, Some(120.0), None, 15, 8));
        store.add_feeding(make_feeding("Noah", FeedingType::Bottle, Some(100.0), None, 15, 9));

        let r = store.report(Some("Emma"), ts(15, 0, 0), ts(16, 0, 0));
        assert_eq!(r[0].total_feedings, 1);
        assert_eq!(r[0].total_ml, 120.0);
    }
}
