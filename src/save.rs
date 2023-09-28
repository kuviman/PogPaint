use std::path::PathBuf;

use super::*;

#[derive(Serialize, Deserialize)]
pub enum Image {
    Load(PathBuf),
    Embed { size: vec2<usize>, data: Vec<u8> },
}

#[derive(Serialize, Deserialize)]
pub struct Plane {
    image: Option<Image>,
    offset: vec2<i32>,
    transform: mat4<f32>,
}

#[derive(Serialize, Deserialize)]
pub struct Pp {
    pub planes: Vec<Plane>,
}

impl App {
    pub fn save(&self) {
        let pp = Pp {
            planes: self
                .state
                .model
                .planes
                .iter()
                .map(|plane| Plane {
                    image: plane.texture.texture.as_ref().map(|texture| Image::Embed {
                        size: texture.size(),
                        data: {
                            let framebuffer = ugli::FramebufferRead::new_color(
                                self.ctx.geng.ugli(),
                                ugli::ColorAttachmentRead::Texture(texture),
                            );
                            framebuffer.read_color().data().to_vec()
                        },
                    }),
                    offset: plane.texture.bb.bottom_left(),
                    transform: plane.transform,
                })
                .collect(),
        };
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        encoder
            .write_all(&bincode::serialize(&pp).unwrap())
            .unwrap();
        let _ = file_dialog::save("model.pp", &encoder.finish().unwrap());
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
                    let mut reader = file.reader().unwrap();
                    let mut buf = Vec::new();
                    reader.read_to_end(&mut buf).await.unwrap();
                    let mut decoder = flate2::read::GzDecoder::new(buf.as_slice());
                    let mut buf = Vec::new();
                    decoder.read_to_end(&mut buf).unwrap();
                    let pp: Pp = bincode::deserialize(&buf).unwrap();
                    let model = Model {
                        planes: stream::iter(pp.planes.into_iter())
                            .then(|plane| async move {
                                let texture = match plane.image {
                                    Some(image) => Some(match image {
                                        Image::Load(path) => {
                                            ctx.geng.asset_manager().load(path).await.unwrap()
                                        }
                                        Image::Embed { size, data } => {
                                            let mut texture = ugli::Texture::new_uninitialized(
                                                ctx.geng.ugli(),
                                                size,
                                            );
                                            texture.sub_image(vec2::ZERO, size, &data);
                                            texture
                                        }
                                    }),
                                    None => None,
                                };
                                crate::Plane {
                                    texture: crate::Texture::from(&ctx, texture, plane.offset),
                                    transform: plane.transform,
                                }
                            })
                            .collect()
                            .await,
                    };
                    let _ = sender.send(model);
                })
                .detach();
        });
    }
}
