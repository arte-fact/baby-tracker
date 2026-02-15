use std::fmt;

use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
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
    pub fn from_str(s: &str) -> Result<Self, String> {
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

    pub fn to_db_str(&self) -> &'static str {
        match self {
            FeedingType::BreastLeft => "breast-left",
            FeedingType::BreastRight => "breast-right",
            FeedingType::Bottle => "bottle",
            FeedingType::Solid => "solid",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "breast-left" => FeedingType::BreastLeft,
            "breast-right" => FeedingType::BreastRight,
            "bottle" => FeedingType::Bottle,
            "solid" => FeedingType::Solid,
            _ => FeedingType::Bottle, // fallback
        }
    }
}

#[derive(Debug, Clone)]
pub struct Feeding {
    pub id: i64,
    pub baby_name: String,
    pub feeding_type: FeedingType,
    pub amount_ml: Option<f64>,
    pub duration_minutes: Option<i32>,
    pub notes: Option<String>,
    pub timestamp: NaiveDateTime,
}
