use super::*;

use std::path::PathBuf;

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

impl Model {
    pub fn save(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        let pp = Pp {
            planes: self
                .planes
                .iter()
                .map(|plane| Plane {
                    image: plane.texture.texture.as_ref().map(|texture| Image::Embed {
                        size: texture.size(),
                        data: {
                            let framebuffer = ugli::FramebufferRead::new_color(
                                &self.ugli,
                                ugli::ColorAttachmentRead::Texture(texture),
                            );
                            framebuffer.read_color().data().to_vec()
                        },
                    }),
                    offset: plane.texture.offset,
                    transform: plane.transform,
                })
                .collect(),
        };
        let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::best());
        encoder.write_all(&bincode::serialize(&pp).unwrap())?;
        encoder.finish()?;
        Ok(())
    }

    pub async fn load(
        asset_manager: &geng::asset::Manager,
        reader: impl AsyncBufRead,
    ) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        let mut reader = std::pin::pin!(reader);
        reader.read_to_end(&mut buf).await.unwrap();
        let mut decoder = flate2::read::GzDecoder::new(buf.as_slice());
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        let pp: Pp = bincode::deserialize(&buf).unwrap();
        Ok(Self {
            ugli: asset_manager.ugli().clone(),
            planes: stream::iter(pp.planes.into_iter())
                .then(|plane| async move {
                    let mut texture = match plane.image {
                        Some(image) => Some(match image {
                            Image::Load(path) => asset_manager.load(path).await.unwrap(),
                            Image::Embed { size, data } => {
                                let mut texture =
                                    ugli::Texture::new_uninitialized(asset_manager.ugli(), size);
                                texture.sub_image(vec2::ZERO, size, &data);
                                texture
                            }
                        }),
                        None => None,
                    };
                    if let Some(texture) = &mut texture {
                        texture.set_filter(ugli::Filter::Nearest);
                    }
                    crate::Plane {
                        texture: crate::Texture::from(asset_manager.ugli(), texture, plane.offset),
                        transform: plane.transform,
                    }
                })
                .collect()
                .await,
        })
    }
}

impl geng::asset::Load for Model {
    type Options = ();

    fn load(
        manager: &geng::asset::Manager,
        path: &std::path::Path,
        _options: &Self::Options,
    ) -> geng::asset::Future<Self> {
        let manager = manager.clone();
        let path = path.to_owned();
        async move {
            let reader = file::load(path).await?;
            Ok(Self::load(&manager, reader).await?)
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = Some("pp");
}
