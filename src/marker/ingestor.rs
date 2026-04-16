use anyhow::Result;
use image::DynamicImage;
use pdfium::{PdfiumDocument, PdfiumRenderConfig};
use std::collections::BTreeMap;

pub fn load_quiz(pdf_path: &str) -> Result<BTreeMap<String, BTreeMap<u32, DynamicImage>>> {
    let doc = PdfiumDocument::new_from_path(pdf_path, None).unwrap();
    let conf = PdfiumRenderConfig::new().with_height(1080);

    let mut tests: BTreeMap<String, BTreeMap<u32, DynamicImage>> = BTreeMap::new();

    for page in doc.pages() {
        let img = page
            .unwrap()
            .render(&conf)
            .unwrap()
            .as_rgba8_image()
            .unwrap();

        let barcode_res =
            rxing::helpers::detect_in_image(img.clone(), Some(rxing::BarcodeFormat::PDF_417));

        if let Ok(barcode) = barcode_res {
            let raw_text = barcode.getText();

            if let Some((username, page_num_str)) = raw_text.split_once('@') {
                let page_num = page_num_str.parse::<u32>().unwrap();
                tests
                    .entry(username.to_string())
                    .or_default()
                    .insert(page_num, img);
            }
        }
    }

    Ok(tests)
}
