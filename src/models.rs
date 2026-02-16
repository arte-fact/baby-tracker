use std::fmt;

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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

    // --- FeedingType serde round-trip ---

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
        assert_eq!(f.id, 0); // unassigned
    }

    #[test]
    fn feeding_new_trims_name() {
        let f = Feeding::new("  Emma  ".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0)).unwrap();
        assert_eq!(f.baby_name, "Emma");
    }

    #[test]
    fn feeding_new_empty_name_rejected() {
        let r = Feeding::new("".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0));
        assert!(r.is_err());
        let r = Feeding::new("   ".to_string(), FeedingType::Bottle, None, None, None, ts(8, 0));
        assert!(r.is_err());
    }

    #[test]
    fn feeding_new_negative_amount_rejected() {
        let r = Feeding::new("Emma".to_string(), FeedingType::Bottle, Some(-10.0), None, None, ts(8, 0));
        assert!(r.is_err());
    }

    #[test]
    fn feeding_new_blank_notes_become_none() {
        let f = Feeding::new("Emma".to_string(), FeedingType::Solid, None, None, Some("  ".to_string()), ts(8, 0)).unwrap();
        assert_eq!(f.notes, None);
    }

    // --- Feeding serde round-trip ---

    #[test]
    fn feeding_serde_roundtrip() {
        let f = Feeding::new(
            "Emma".to_string(),
            FeedingType::BreastRight,
            None,
            Some(15),
            Some("Good latch".to_string()),
            ts(14, 30),
        )
        .unwrap();
        let json = serde_json::to_string(&f).unwrap();
        let parsed: Feeding = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.baby_name, f.baby_name);
        assert_eq!(parsed.feeding_type, f.feeding_type);
        assert_eq!(parsed.duration_minutes, f.duration_minutes);
        assert_eq!(parsed.timestamp, f.timestamp);
    }
}
