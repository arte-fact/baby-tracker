pub mod models;
pub mod store;
pub mod tracker;

use wasm_bindgen::prelude::*;

use tracker::Tracker;

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

    // --- Feeding ---

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

    #[wasm_bindgen(js_name = updateFeeding)]
    pub fn update_feeding(
        &mut self,
        id: u64,
        feeding_type: &str,
        amount_ml: Option<f64>,
        duration_minutes: Option<u32>,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, JsError> {
        self.inner
            .update_feeding(id, feeding_type, amount_ml, duration_minutes, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = deleteFeeding)]
    pub fn delete_feeding(&mut self, id: u64) -> bool {
        self.inner.delete_feeding(id)
    }

    // --- Dejection ---

    #[wasm_bindgen(js_name = addDejection)]
    pub fn add_dejection(
        &mut self,
        baby_name: &str,
        dejection_type: &str,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, JsError> {
        self.inner
            .add_dejection(baby_name, dejection_type, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = updateDejection)]
    pub fn update_dejection(
        &mut self,
        id: u64,
        dejection_type: &str,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, JsError> {
        self.inner
            .update_dejection(id, dejection_type, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = deleteDejection)]
    pub fn delete_dejection(&mut self, id: u64) -> bool {
        self.inner.delete_dejection(id)
    }

    // --- Weight ---

    #[wasm_bindgen(js_name = addWeight)]
    pub fn add_weight(
        &mut self,
        baby_name: &str,
        weight_kg: f64,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<u64, JsError> {
        self.inner
            .add_weight(baby_name, weight_kg, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = updateWeight)]
    pub fn update_weight(
        &mut self,
        id: u64,
        weight_kg: f64,
        notes: Option<String>,
        timestamp: &str,
    ) -> Result<bool, JsError> {
        self.inner
            .update_weight(id, weight_kg, notes, timestamp)
            .map_err(|e| JsError::new(&e))
    }

    #[wasm_bindgen(js_name = deleteWeight)]
    pub fn delete_weight(&mut self, id: u64) -> bool {
        self.inner.delete_weight(id)
    }

    // --- Timeline ---

    #[wasm_bindgen(js_name = timelineForDay)]
    pub fn timeline_for_day(
        &self,
        baby_name: Option<String>,
        date: &str,
    ) -> Result<String, JsError> {
        self.inner
            .timeline_for_day(baby_name.as_deref(), date)
            .map_err(|e| JsError::new(&e))
    }

    // --- Summary (day-bounded) ---

    #[wasm_bindgen(js_name = getSummary)]
    pub fn get_summary(
        &self,
        baby_name: Option<String>,
        date: &str,
    ) -> Result<String, JsError> {
        self.inner
            .get_summary(baby_name.as_deref(), date)
            .map_err(|e| JsError::new(&e))
    }

    // --- Report ---

    #[wasm_bindgen(js_name = getReport)]
    pub fn get_report(
        &self,
        baby_name: Option<String>,
        start_date: &str,
        end_date: &str,
    ) -> Result<String, JsError> {
        self.inner
            .report(baby_name.as_deref(), start_date, end_date)
            .map_err(|e| JsError::new(&e))
    }
}
