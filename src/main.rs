use geng::prelude::*;

#[derive(clap::Parser)]
struct Cli {
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(geng::asset::Load, Deserialize)]
#[load(serde = "toml")]
struct Config {
    background_color: Rgba<f32>,
}

struct State {
    config: Config,
    geng: Geng,
}

impl State {
    pub async fn new(geng: &Geng) -> Self {
        Self {
            config: geng
                .asset_manager()
                .load(run_dir().join("config.toml"))
                .await
                .unwrap(),
            geng: geng.clone(),
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(
            framebuffer,
            Some(self.config.background_color),
            Some(1.0),
            None,
        );
    }

    pub async fn run(mut self) {
        let mut events = self.geng.window().events();
        while let Some(event) = events.next().await {
            match event {
                geng::Event::Draw => {
                    self.geng
                        .clone()
                        .window()
                        .with_framebuffer(|framebuffer| self.draw(framebuffer));
                }
                _ => {}
            }
        }
    }
}

fn main() {
    let cli: Cli = clap::Parser::parse();
    geng::Geng::run_with(
        &{
            let mut options = geng::ContextOptions::default();
            options.window.title = "PogPaint".to_owned();
            options.with_cli(&cli.geng);
            options
        },
        |geng| async move {
            State::new(&geng).await.run().await;
        },
    );
}
