use super::*;

use std::path::PathBuf;

pub mod versions {
    use super::*;

    pub mod v0 {
        use super::*;

        pub const HEADER: &str = "PogPaint";

        #[derive(Serialize, Deserialize)]
        pub enum Image {
            Load(PathBuf),
            Embed { size: vec2<usize>, data: Vec<u8> },
        }

        #[derive(Serialize, Deserialize)]
        pub struct Plane {
            pub image: Option<Image>,
            pub offset: vec2<i32>,
            pub transform: mat4<f32>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct Pp {
            pub planes: Vec<Plane>,
        }
    }

    pub mod v1 {
        use super::*;
        pub const VERSION: u8 = 1;

        #[derive(Serialize, Deserialize)]
        pub enum ImageData {
            Load(PathBuf),
            Embed { size: vec2<usize>, data: Vec<u8> },
        }

        #[derive(Serialize, Deserialize)]
        pub struct Image {
            pub data: ImageData,
            pub offset: vec2<i32>,
        }

        #[derive(Serialize, Deserialize)]
        pub enum HeightmapData {
            Embed { data: Array2D<f32> },
        }

        #[derive(Serialize, Deserialize)]
        pub struct Heightmap {
            pub data: HeightmapData,
            pub offset: vec2<i32>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct Plane {
            pub image: Option<Image>,
            pub heightmap: Option<Heightmap>,
            pub transform: mat4<f32>,
        }

        #[derive(Serialize, Deserialize)]
        pub struct Pp {
            pub planes: Vec<Plane>,
        }

        impl From<v0::Image> for ImageData {
            fn from(old: v0::Image) -> Self {
                match old {
                    v0::Image::Load(path) => Self::Load(path),
                    v0::Image::Embed { size, data } => Self::Embed { size, data },
                }
            }
        }

        impl From<v0::Plane> for Plane {
            fn from(old: v0::Plane) -> Self {
                Self {
                    image: match old.image {
                        Some(image) => Some(Image {
                            data: image.into(),
                            offset: old.offset,
                        }),
                        None => None,
                    },
                    heightmap: None,
                    transform: old.transform,
                }
            }
        }

        impl From<v0::Pp> for Pp {
            fn from(old: v0::Pp) -> Self {
                Self {
                    planes: old.planes.into_iter().map(Into::into).collect(),
                }
            }
        }
    }
}

use versions::v1 as current_version;

use current_version::{Heightmap, HeightmapData, Image, ImageData, Plane, Pp};

const HEADER: &str = "VersionedPogPaint";

impl Model {
    pub fn save(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        let pp = Pp {
            planes: self
                .planes
                .iter()
                .map(|plane| Plane {
                    image: plane.texture.texture.as_ref().map(|texture| Image {
                        data: ImageData::Embed {
                            size: texture.size(),
                            data: {
                                let framebuffer = ugli::FramebufferRead::new_color(
                                    &self.ugli,
                                    ugli::ColorAttachmentRead::Texture(texture),
                                );
                                framebuffer.read_color().data().to_vec()
                            },
                        },
                        offset: plane.texture.offset,
                    }),
                    heightmap: plane.heightmap.as_ref().map(|heightmap| Heightmap {
                        data: HeightmapData::Embed {
                            data: heightmap.data.clone(),
                        },
                        offset: heightmap.offset,
                    }),
                    transform: plane.transform,
                })
                .collect(),
        };
        writer.write_all(HEADER.as_bytes())?;
        writer.write_all(&[current_version::VERSION])?;
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
        let version = {
            let mut first_byte = vec![0];
            reader.read_exact(&mut first_byte).await?;
            let first_byte = first_byte[0];
            if first_byte == versions::v0::HEADER.as_bytes()[0] {
                let mut header = vec![0; versions::v0::HEADER.len()];
                header[0] = first_byte;
                reader.read_exact(&mut header[1..]).await?;
                assert_eq!(header, versions::v0::HEADER.as_bytes());
                0
            } else {
                let mut header = vec![0; HEADER.len()];
                header[0] = first_byte;
                reader.read_exact(&mut header[1..]).await?;
                assert_eq!(header, HEADER.as_bytes());
                let mut version_byte = vec![0];
                reader.read_exact(&mut version_byte).await?;
                version_byte[0]
            }
        };
        println!("{version}");
        reader.read_to_end(&mut buf).await.unwrap();
        let mut decoder = flate2::read::GzDecoder::new(buf.as_slice());
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        let pp: current_version::Pp = match version {
            0 => {
                let v0: versions::v0::Pp = bincode::deserialize(&buf).unwrap();
                v0.into()
            }
            versions::v1::VERSION => bincode::deserialize(&buf).unwrap(),
            _ => todo!("unknown version {version}"),
        };

        Ok(Self {
            ugli: asset_manager.ugli().clone(),
            planes: stream::iter(pp.planes.into_iter())
                .then(|plane| async move {
                    let texture = |image: ImageData| async move {
                        let mut texture = match image {
                            ImageData::Load(path) => asset_manager.load(path).await.unwrap(),
                            ImageData::Embed { size, data } => {
                                let mut texture =
                                    ugli::Texture::new_uninitialized(asset_manager.ugli(), size);
                                texture.sub_image(vec2::ZERO, size, &data);
                                texture
                            }
                        };
                        texture.set_filter(ugli::Filter::Nearest);
                        texture
                    };
                    crate::Plane {
                        texture: {
                            let (texture, offset) = match plane.image {
                                Some(image) => (Some(texture(image.data).await), image.offset),
                                None => (None, vec2::ZERO),
                            };
                            crate::Texture::from(asset_manager.ugli(), texture, offset)
                        },
                        heightmap: match plane.heightmap {
                            Some(heightmap) => Some(crate::Heightmap {
                                data: match heightmap.data {
                                    HeightmapData::Embed { data } => data,
                                },
                                offset: heightmap.offset,
                            }),
                            None => None,
                        },
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
