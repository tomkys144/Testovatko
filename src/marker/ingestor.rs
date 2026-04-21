use anyhow::Result;
use image::{DynamicImage, Rgba, RgbaImage};
use poppler::cairo::{Context, Format, ImageSurface};
use poppler::{PopplerDocument, PopplerPage};
use std::collections::BTreeMap;

pub fn load_quiz(pdf_path: &str) -> Result<(PopplerDocument, BTreeMap<String, BTreeMap<u32, usize>>)> {
    let doc = PopplerDocument::new_from_file(pdf_path, None)?;

    let mut tests: BTreeMap<String, BTreeMap<u32, usize>> = BTreeMap::new();

    for (pdf_index, page) in doc.pages().enumerate() {
        let img = pdf_to_image(&page, 2.0)?;

        let barcode_res =
            rxing::helpers::detect_in_image(img.clone(), Some(rxing::BarcodeFormat::PDF_417));

        if let Ok(barcode) = barcode_res {
            let raw_text = barcode.getText();

            if let Some((username, page_num_str)) = raw_text.split_once('@') {
                let page_num = page_num_str.parse::<u32>().unwrap();
                tests
                    .entry(username.to_string())
                    .or_default()
                    .insert(page_num, pdf_index);
            }
        }
    }

    Ok((doc, tests))
}

pub fn load_page(doc: &PopplerDocument, page_num: usize, scale: Option<f64>) -> Result<DynamicImage> {
    let page = doc.get_page(page_num).expect("Failed to get page");

    let img = pdf_to_image(&page, scale.unwrap_or(4.0)).expect("Failed to convert to image");

    Ok(img)
}

fn pdf_to_image(pdf: &PopplerPage, scale: f64) -> Result<DynamicImage> {
    let (width, height) = pdf.get_size();

    let pix_width = (width * scale) as i32;
    let pix_height = (height * scale) as i32;

    let mut surface = ImageSurface::create(Format::ARgb32, pix_width, pix_height)
        .expect("Failed to create surface");

    let ctx = Context::new(&surface).expect("Failed to create context");

    ctx.set_source_rgb(1.0, 1.0, 1.0);
    ctx.paint().expect("Failed to paint background");

    ctx.scale(scale, scale);
    pdf.render(&ctx);

    drop(ctx);

    let stride = surface.stride() as usize;
    let data = surface.data().expect("Failed to get data");

    let mut img = RgbaImage::new(pix_width as u32, pix_height as u32);

    for y in 0..(pix_height as usize) {
        for x in 0..(pix_width as usize) {
            let offset = y * stride + x * 4;

            let b = data[offset];
            let g = data[offset + 1];
            let r = data[offset + 2];
            let a = data[offset + 3];

            img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
        }
    }

    Ok(DynamicImage::ImageRgba8(img))
}
