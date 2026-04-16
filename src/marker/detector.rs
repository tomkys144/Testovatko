use anyhow::Result;
use aruco_rs::ImageBuffer;
use aruco_rs::core::detector::Detector;
use aruco_rs::core::dictionary::{DICTIONARY_ARUCO, Dictionary};
use aruco_rs::cv::scalar::ScalarCV;
use image::{DynamicImage, GrayImage};
use imageproc::contrast;

const RESULTS_IDS: [u32; 4] = [0, 1, 2, 3];
const CHOICES_IDS: [u32; 4] = [4, 5, 6, 7];

pub fn process_page(img: &DynamicImage) -> Result<()> {
    let gray_img = img.to_luma8();
    let otsu_val = contrast::otsu_level(&gray_img);
    let thr_img = contrast::threshold(&gray_img, otsu_val, contrast::ThresholdType::Binary);

    Ok(())
}

fn detect_uruco(img: &GrayImage) -> Result<()> {
    let dict = Dictionary::new(&DICTIONARY_ARUCO);
    let mut detector = Detector::new(&dict, ScalarCV);
    let buffer = ImageBuffer {
        data: img.as_raw(),
        width: img.width(),
        height: img.height(),
    };
    let markers = detector.detect(&buffer);

    for marker in markers {
        println!(
            "Found ArUco ID {}, Corners: {:?}",
            marker.id, marker.corners
        );
    }
    Ok(())
}
