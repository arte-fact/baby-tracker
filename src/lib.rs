pub mod models;
pub mod store;
pub mod tracker;

use wasm_bindgen::prelude::*;

use tracker::Tracker;

/// WASM-exported wrapper around Tracker.
/// All business logic lives in tracker.rs (testable on native).
#[wasm_bindgen]
pub struct BabyTracker {
    inner: Tracker,
}

#[wasm_bindgen]
impl BabyTracker {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        BabyTracker {
            inner: Tracker::new(),
        }
    }

    #[wasm_bindgen(js_name = loadData)]
    pub fn load_data(json: &str) -> Result<BabyTracker, JsError> {
        let inner = Tracker::from_json(json).map_err(|e| JsError::new(&e))?;
        Ok(BabyTracker { inner })
    }

    #[wasm_bindgen(js_name = exportData)]
    pub fn export_data(&self) -> String {
        self.inner.export_data()
    }

    #[wasm_bindgen(js_name = addFeeding)]
    pub fn add_feeding(
        &mut self,
        baby_name: &str,
        feeding_type: &str,
        amount_ml: Option<f64>,
        duration_minutes: Option<u32>,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, JsError> {
        self.inner
            .add_feeding(baby_name, feeding_type, amount_ml, duration_minutes, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = deleteFeeding)]
    pub fn delete_feeding(&mut self, id: u64) -> bool {
        self.inner.delete_feeding(id)
    }

    #[wasm_bindgen(js_name = listFeedings)]
    pub fn list_feedings(&self, baby_name: Option<String>, limit: usize) -> String {
        self.inner.list_feedings(baby_name.as_deref(), limit)
    }

    #[wasm_bindgen(js_name = getSummary)]
    pub fn get_summary(
        &self,
        baby_name: Option<String>,
        since: &str,
    ) -> Result<String, JsError> {
        self.inner
            .get_summary(baby_name.as_deref(), since)
            .map_err(|e| JsError::new(&e))
    }
}
