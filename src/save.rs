use super::*;

impl App {
    pub fn save(&self) {
        let data = serde_json::to_string_pretty(&self.state.model).unwrap();
        let _ = file_dialog::save("model.pp", data.as_bytes());
    }
    pub fn load(&mut self) {}
}
