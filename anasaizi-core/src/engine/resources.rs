mod texture_loader;
mod texture_storage;

pub use texture_loader::TextureLoader;
pub use texture_storage::{StoredTexture, TextureCollection, TextureId, TextureStorage};

use image::{GenericImageView, ImageFormat};
use std::sync::{Arc, Mutex};

pub async fn load_image_path(
    path: &'static str,
    id: &'static str,
    image_results: Arc<Mutex<TextureStorage>>,
    debug: bool,
) {
    let texture_id = TextureId::new(id);

    let contents = if !debug {
        tokio::fs::read(path).await.unwrap()
    } else {
        tokio::fs::read("assets/textures/debug/albedo.jpg")
            .await
            .unwrap()
    };

    let mut image_object = image::load_from_memory(&contents).unwrap();
    image_object = image_object.flipv();

    let (image_width, image_height) = (image_object.width(), image_object.height());

    let image_data = match &image_object {
        image::DynamicImage::ImageLuma8(_)
        | image::DynamicImage::ImageBgr8(_)
        | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba8().into_raw(),
        image::DynamicImage::ImageLumaA8(_)
        | image::DynamicImage::ImageBgra8(_)
        | image::DynamicImage::ImageRgba8(_) => image_object.to_bytes(),
        _ => {
            panic!("Invalid image format, image should be rgba compatible");
        }
    };

    let stored_texture = StoredTexture::new(
        image_data,
        image_width,
        image_height,
        path.to_string(),
        texture_id.texture(),
        None,
    );

    let mut lock = image_results.lock().unwrap();
    lock.insert(texture_id.clone(), stored_texture);
}

//
// pub async fn load_image_raw(
//     data: &[u8],
//     id: &str,
//     image_results: Arc<Mutex<TextureStorage>>,
//     debug: bool,
// ) {
//     let parent_buffer_data = &data;
//     let begin = view.offset();
//     let end = begin + view.length();
//     let data = &parent_buffer_data[begin..end];
//
//     let image_object = match mime_type {
//         "image/jpeg" => image::load_from_memory_with_format(data, ImageFormat::Jpeg),
//         "image/png" => image::load_from_memory_with_format(data, ImageFormat::Png),
//         _ => panic!(format!("unsupported image type (image: {}, mime_type: {})",
//                             g_img.index(), mime_type)),
//     }.unwrap();
//
//     let image_data = match &image_object {
//         image::DynamicImage::ImageLuma8(_)
//         | image::DynamicImage::ImageBgr8(_)
//         | image::DynamicImage::ImageRgb8(_) => image_object.to_rgba8().into_raw(),
//         image::DynamicImage::ImageLumaA8(_)
//         | image::DynamicImage::ImageBgra8(_)
//         | image::DynamicImage::ImageRgba8(_) => image_object.to_bytes(),
//         _ => {
//             panic!("Invalid image format, image should be rgba compatible");
//         }
//     };
//
//     let stored_texture = StoredTexture::new(
//         image_data,
//         image_width,
//         image_height,
//         path,
//         id.to_string(),
//         None,
//     );
//
//     let mut lock = image_results.lock().unwrap();
//     lock.insert(id.to_string(), stored_texture);
// }
