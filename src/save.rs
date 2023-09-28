use super::*;

impl App {
    pub fn save(&self) {
        let mut data = Vec::new();
        self.state.model.save(&mut data).unwrap();
        let _ = file_dialog::save("model.pp", &data);
    }
    pub fn load(&mut self) {
        let sender = self.load_sender.clone();
        let ctx = self.ctx.clone();
        file_dialog::select(|file| {
            ctx.clone()
                .geng
                .window()
                .spawn(async move {
                    let ctx = &ctx;
                    let _ = sender.send(
                        pog_paint::Model::load(ctx.geng.asset_manager(), file.reader().unwrap())
                            .await
                            .unwrap(),
                    );
                })
                .detach();
        });
    }
}
