use image_crate::codecs::{jpeg::JpegDecoder, png::PngDecoder};
use printpdf::*;
use printpdf::{Image, Mm, PdfDocument};
use std::{
    error::Error,
    io::Cursor,
    io::Error as IoError,
    path::{Path, PathBuf},
};

const PDF_DPI: f64 = 300.0;
const A5_WIDTH: Mm = Mm(148.0);
const A5_HEIGHT: Mm = Mm(210.0);

pub async fn create_pdf_from_images(
    name: &str,
    parent_dir: &Path,
    images: &[PathBuf],
    scale: f64,
) -> Result<PathBuf, Box<dyn Error>> {
    let (pdf, page, layer) = PdfDocument::new(name, A5_WIDTH, A5_HEIGHT, "layer");

    let mut current_page = page;
    let mut current_layer = layer;

    let page_count = images.len();
    for (index, image_path) in images.iter().enumerate() {
        let extension = image_path.extension().and_then(|x| x.to_str());
        let current_ref = pdf.get_page(current_page).get_layer(current_layer);
        let image_data = Cursor::new(tokio::fs::read(image_path).await?);
        let image = if extension == Some("png") {
            Image::try_from(PngDecoder::new(image_data)?)?
        } else if extension == Some("jpg") || extension == Some("jpeg") {
            Image::try_from(JpegDecoder::new(image_data)?)?
        } else {
            return Err(Box::new(IoError::new(
                std::io::ErrorKind::Unsupported,
                "unsupported image format",
            )));
        };

        // Calculate transform.
        let image_width = Mm::from(image.image.width.into_pt(PDF_DPI));
        let image_height = Mm::from(image.image.height.into_pt(PDF_DPI));
        let width_factor = A5_WIDTH / image_width;
        let height_factor = A5_HEIGHT / image_height;
        let scale_factor = if width_factor < height_factor {
            width_factor * scale
        } else {
            height_factor * scale
        };
        let translate_x = (A5_WIDTH - image_width * scale_factor) / 2.0;
        let translate_y = (A5_HEIGHT - image_height * scale_factor) / 2.0;
        let transform = ImageTransform {
            translate_x: Some(translate_x),
            translate_y: Some(translate_y),
            scale_x: Some(scale_factor),
            scale_y: Some(scale_factor),
            rotate: None,
            dpi: Some(PDF_DPI),
        };

        image.add_to_layer(current_ref.clone(), transform);

        // Do not add a empty page if this is the last image.
        if index + 1 < page_count {
            let (page, layer) = pdf.add_page(A5_WIDTH, A5_HEIGHT, "layer");
            current_page = page;
            current_layer = layer;
        }
    }
    let result = parent_dir.join(name);
    tokio::fs::write(result.clone(), pdf.save_to_bytes()?).await?;
    return Ok(result);
}
