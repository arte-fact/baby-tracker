use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// --- FeedingType ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeedingType {
    BreastLeft,
    BreastRight,
    Bottle,
    Solid,
}

impl fmt::Display for FeedingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeedingType::BreastLeft => write!(f, "Breast (Left)"),
            FeedingType::BreastRight => write!(f, "Breast (Right)"),
            FeedingType::Bottle => write!(f, "Bottle"),
            FeedingType::Solid => write!(f, "Solid"),
        }
    }
}

impl FeedingType {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "breast-left" | "bl" => Ok(FeedingType::BreastLeft),
            "breast-right" | "br" => Ok(FeedingType::BreastRight),
            "bottle" | "b" => Ok(FeedingType::Bottle),
            "solid" | "s" => Ok(FeedingType::Solid),
            _ => Err(format!(
                "Unknown feeding type: '{}'. Use: breast-left (bl), breast-right (br), bottle (b), solid (s)",
                s
            )),
        }
    }
}

// --- Feeding ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feeding {
    pub id: u64,
    pub baby_name: String,
    pub feeding_type: FeedingType,
    pub amount_ml: Option<f64>,
    pub duration_minutes: Option<u32>,
    pub notes: Option<String>,
    pub timestamp: NaiveDateTime,
}

impl Feeding {
    pub fn new(
        baby_name: String,
        feeding_type: FeedingType,
        amount_ml: Option<f64>,
        duration_minutes: Option<u32>,
        notes: Option<String>,
        timestamp: NaiveDateTime,
    ) -> Result<Self, String> {
        if baby_name.trim().is_empty() {
            return Err("Baby name cannot be empty".to_string());
        }
        if let Some(ml) = amount_ml {
            if ml < 0.0 {
                return Err("Amount cannot be negative".to_string());
            }
        }
        Ok(Feeding {
            id: 0,
            baby_name: baby_name.trim().to_string(),
            feeding_type,
            amount_ml,
            duration_minutes,
            notes: notes.filter(|n| !n.trim().is_empty()),
            timestamp,
        })
    }
}

// --- DejectionType ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DejectionType {
    Urine,
    Poop,
}

impl fmt::Display for DejectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DejectionType::Urine => write!(f, "Urine"),
            DejectionType::Poop => write!(f, "Poop"),
        }
    }
}

impl DejectionType {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "urine" | "pee" | "u" => Ok(DejectionType::Urine),
            "poop" | "p" => Ok(DejectionType::Poop),
            _ => Err(format!(
                "Unknown dejection type: '{}'. Use: urine (pee/u), poop (p)",
                s
            )),
        }
    }
}

// --- Dejection ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dejection {
    pub id: u64,
    pub baby_name: String,
    pub dejection_type: DejectionType,
    pub notes: Option<String>,
    pub timestamp: NaiveDateTime,
}

impl Dejection {
    pub fn new(
        baby_name: String,
        dejection_type: DejectionType,
        notes: Option<String>,
        timestamp: NaiveDateTime,
    ) -> Result<Self, String> {
        if baby_name.trim().is_empty() {
            return Err("Baby name cannot be empty".to_string());
        }
        Ok(Dejection {
            id: 0,
            baby_name: baby_name.trim().to_string(),
            dejection_type,
            notes: notes.filter(|n| !n.trim().is_empty()),
            timestamp,
        })
    }
}

// --- Weight ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weight {
    pub id: u64,
    pub baby_name: String,
    pub weight_kg: f64,
    pub notes: Option<String>,
    pub timestamp: NaiveDateTime,
}

impl Weight {
    pub fn new(
        baby_name: String,
        weight_kg: f64,
        notes: Option<String>,
        timestamp: NaiveDateTime,
    ) -> Result<Self, String> {
        if baby_name.trim().is_empty() {
            return Err("Baby name cannot be empty".to_string());
        }
        if weight_kg <= 0.0 {
            return Err("Weight must be positive".to_string());
        }
        Ok(Weight {
            id: 0,
            baby_name: baby_name.trim().to_string(),
            weight_kg,
            notes: notes.filter(|n| !n.trim().is_empty()),
            timestamp,
        })
    }
}

// --- Unified timeline entry for day view ---

#[derive(Debug, Serialize)]
pub struct TimelineEntry {
    pub id: u64,
    pub kind: &'static str,
    pub baby_name: String,
    pub subtype: String,
    pub amount_ml: Option<f64>,
    pub duration_minutes: Option<u32>,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
    pub timestamp: NaiveDateTime,
}

impl TimelineEntry {
    pub fn from_feeding(f: &Feeding) -> Self {
        TimelineEntry {
            id: f.id,
            kind: "feeding",
            baby_name: f.baby_name.clone(),
            subtype: serde_json::to_string(&f.feeding_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            amount_ml: f.amount_ml,
            duration_minutes: f.duration_minutes,
            weight_kg: None,
            notes: f.notes.clone(),
            timestamp: f.timestamp,
        }
    }

    pub fn from_dejection(d: &Dejection) -> Self {
        TimelineEntry {
            id: d.id,
            kind: "dejection",
            baby_name: d.baby_name.clone(),
            subtype: serde_json::to_string(&d.dejection_type)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            amount_ml: None,
            duration_minutes: None,
            weight_kg: None,
            notes: d.notes.clone(),
            timestamp: d.timestamp,
        }
    }

    pub fn from_weight(w: &Weight) -> Self {
        TimelineEntry {
            id: w.id,
            kind: "weight",
            baby_name: w.baby_name.clone(),
            subtype: "weight".to_string(),
            amount_ml: None,
            duration_minutes: None,
            weight_kg: Some(w.weight_kg),
            notes: w.notes.clone(),
            timestamp: w.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn ts(h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 2, 15)
            .unwrap()
            .and_hms_opt(h, m, 0)
            .unwrap()
    }

    // --- FeedingType parsing ---

    #[test]
    fn parse_feeding_type_full_names() {
        assert_eq!(FeedingType::parse("breast-left").unwrap(), FeedingType::BreastLeft);
        assert_eq!(FeedingType::parse("breast-right").unwrap(), FeedingType::BreastRight);
        assert_eq!(FeedingType::parse("bottle").unwrap(), FeedingType::Bottle);
        assert_eq!(FeedingType::parse("solid").unwrap(), FeedingType::Solid);
    }

    #[test]
    fn parse_feeding_type_shortcuts() {
        assert_eq!(FeedingType::parse("bl").unwrap(), FeedingType::BreastLeft);
        assert_eq!(FeedingType::parse("br").unwrap(), FeedingType::BreastRight);
        assert_eq!(FeedingType::parse("b").unwrap(), FeedingType::Bottle);
        assert_eq!(FeedingType::parse("s").unwrap(), FeedingType::Solid);
    }

    #[test]
    fn parse_feeding_type_case_insensitive() {
        assert_eq!(FeedingType::parse("BOTTLE").unwrap(), FeedingType::Bottle);
        assert_eq!(FeedingType::parse("Breast-Left").unwrap(), FeedingType::BreastLeft);
    }

    #[test]
    fn parse_feeding_type_invalid() {
        assert!(FeedingType::parse("juice").is_err());
        assert!(FeedingType::parse("").is_err());
    }

    #[test]
    fn feeding_type_display() {
        assert_eq!(FeedingType::BreastLeft.to_string(), "Breast (Left)");
        assert_eq!(FeedingType::Bottle.to_string(), "Bottle");
    }

    #[test]
    fn feeding_type_serde_roundtrip() {
        let ft = FeedingType::BreastLeft;
        let json = serde_json::to_string(&ft).unwrap();
        assert_eq!(json, "\"breast-left\"");
        let parsed: FeedingType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ft);
    }

    // --- Feeding construction & validation ---

    #[test]
    fn feeding_new_valid() {
        let f = Feeding::new(
            "Emma".to_string(),
            FeedingType::Bottle,
            Some(120.0),
            None,
            Some("Morning".to_string()),
            ts(8, 0),
        )
        .unwrap();
        assert_eq!(f.baby_name, "Emma");
        assert_eq!(f.amount_ml, Some(120.0));
        assert_eq!(f.id, 0);
    }

    #[test]
    fn feeding_new_trims_name() {
        let f = Feeding::new("  Emma  ".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0)).unwrap();
        assert_eq!(f.baby_name, "Emma");
    }

    #[test]
    fn feeding_new_empty_name_rejected() {
        assert!(Feeding::new("".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0)).is_err());
        assert!(Feeding::new("   ".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0)).is_err());
    }

    #[test]
    fn feeding_new_negative_amount_rejected() {
        assert!(Feeding::new("Emma".to_string(), FeedingType::Bottle, Some(-10.0), None, None, ts(8, 0)).is_err());
    }

    #[test]
    fn feeding_new_blank_notes_become_none() {
        let f = Feeding::new("Emma".to_string(), FeedingType::Solid, None, None, Some("  ".to_string()), ts(8, 0)).unwrap();
        assert_eq!(f.notes, None);
    }

    #[test]
    fn feeding_serde_roundtrip() {
        let f = Feeding::new("Emma".to_string(), FeedingType::BreastRight, None, Some(15), Some("Good latch".to_string()), ts(14, 30)).unwrap();
        let json = serde_json::to_string(&f).unwrap();
        let parsed: Feeding = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.baby_name, f.baby_name);
        assert_eq!(parsed.feeding_type, f.feeding_type);
        assert_eq!(parsed.duration_minutes, f.duration_minutes);
        assert_eq!(parsed.timestamp, f.timestamp);
    }

    // --- DejectionType parsing ---

    #[test]
    fn parse_dejection_type_full_names() {
        assert_eq!(DejectionType::parse("urine").unwrap(), DejectionType::Urine);
        assert_eq!(DejectionType::parse("poop").unwrap(), DejectionType::Poop);
    }

    #[test]
    fn parse_dejection_type_shortcuts() {
        assert_eq!(DejectionType::parse("pee").unwrap(), DejectionType::Urine);
        assert_eq!(DejectionType::parse("u").unwrap(), DejectionType::Urine);
        assert_eq!(DejectionType::parse("p").unwrap(), DejectionType::Poop);
    }

    #[test]
    fn parse_dejection_type_case_insensitive() {
        assert_eq!(DejectionType::parse("URINE").unwrap(), DejectionType::Urine);
        assert_eq!(DejectionType::parse("Poop").unwrap(), DejectionType::Poop);
    }

    #[test]
    fn parse_dejection_type_invalid() {
        assert!(DejectionType::parse("vomit").is_err());
        assert!(DejectionType::parse("").is_err());
    }

    #[test]
    fn dejection_type_serde_roundtrip() {
        let dt = DejectionType::Poop;
        let json = serde_json::to_string(&dt).unwrap();
        assert_eq!(json, "\"poop\"");
        let parsed: DejectionType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, dt);
    }

    // --- Dejection construction ---

    #[test]
    fn dejection_new_valid() {
        let d = Dejection::new("Emma".to_string(), DejectionType::Poop, Some("Soft".to_string()), ts(10, 0)).unwrap();
        assert_eq!(d.baby_name, "Emma");
        assert_eq!(d.dejection_type, DejectionType::Poop);
        assert_eq!(d.notes, Some("Soft".to_string()));
    }

    #[test]
    fn dejection_new_empty_name_rejected() {
        assert!(Dejection::new("".to_string(), DejectionType::Urine, None, ts(10, 0)).is_err());
    }

    #[test]
    fn dejection_new_blank_notes_become_none() {
        let d = Dejection::new("Emma".to_string(), DejectionType::Urine, Some("  ".to_string()), ts(10, 0)).unwrap();
        assert_eq!(d.notes, None);
    }

    // --- Weight ---

    #[test]
    fn weight_new_valid() {
        let w = Weight::new("Emma".to_string(), 3.5, Some("Birth".to_string()), ts(8, 0)).unwrap();
        assert_eq!(w.baby_name, "Emma");
        assert_eq!(w.weight_kg, 3.5);
        assert_eq!(w.notes, Some("Birth".to_string()));
    }

    #[test]
    fn weight_new_empty_name_rejected() {
        assert!(Weight::new("".to_string(), 3.5, None, ts(8, 0)).is_err());
    }

    #[test]
    fn weight_new_zero_rejected() {
        assert!(Weight::new("Emma".to_string(), 0.0, None, ts(8, 0)).is_err());
    }

    #[test]
    fn weight_new_negative_rejected() {
        assert!(Weight::new("Emma".to_string(), -1.0, None, ts(8, 0)).is_err());
    }

    #[test]
    fn weight_new_blank_notes_become_none() {
        let w = Weight::new("Emma".to_string(), 3.5, Some("  ".to_string()), ts(8, 0)).unwrap();
        assert_eq!(w.notes, None);
    }

    // --- TimelineEntry ---

    #[test]
    fn timeline_entry_from_weight() {
        let mut w = Weight::new("Emma".to_string(), 4.2, None, ts(10, 0)).unwrap();
        w.id = 5;
        let e = TimelineEntry::from_weight(&w);
        assert_eq!(e.kind, "weight");
        assert_eq!(e.subtype, "weight");
        assert_eq!(e.weight_kg, Some(4.2));
        assert_eq!(e.amount_ml, None);
    }

    #[test]
    fn timeline_entry_from_feeding() {
        let mut f = Feeding::new("Emma".to_string(), FeedingType::Bottle, Some(120.0), None, None, ts(8, 0)).unwrap();
        f.id = 1;
        let e = TimelineEntry::from_feeding(&f);
        assert_eq!(e.kind, "feeding");
        assert_eq!(e.subtype, "bottle");
        assert_eq!(e.amount_ml, Some(120.0));
    }

    #[test]
    fn timeline_entry_from_dejection() {
        let mut d = Dejection::new("Emma".to_string(), DejectionType::Poop, None, ts(9, 0)).unwrap();
        d.id = 2;
        let e = TimelineEntry::from_dejection(&d);
        assert_eq!(e.kind, "dejection");
        assert_eq!(e.subtype, "poop");
        assert_eq!(e.amount_ml, None);
    }
}
