use anyhow::Result;
use image::{DynamicImage, GrayImage, Luma, Rgba};
use imageproc::geometric_transformations::{Interpolation, Projection, warp_into, rotate_about_center};
use imageproc::point::Point;
use imageproc::{contours, contrast, geometry};

use crate::models::TableLocation;
struct MarkerCode {
    id: i32,
    rotations: [u16; 4],
}

const DICT_4X4_8: &[MarkerCode] = &[
    MarkerCode {
        id: 0,
        rotations: [0xb532, 0xeb48, 0x4cad, 0x12d7],
    },
    MarkerCode {
        id: 1,
        rotations: [0x0f9a, 0x6547, 0x59f0, 0xe2a6],
    },
    MarkerCode {
        id: 2,
        rotations: [0x332d, 0xde11, 0xb4cc, 0x887b],
    },
    MarkerCode {
        id: 3,
        rotations: [0x9946, 0xc13c, 0x6299, 0x3c83],
    },
    MarkerCode {
        id: 4,
        rotations: [0x549e, 0xa1d3, 0x792a, 0xcb85],
    },
    MarkerCode {
        id: 5,
        rotations: [0x79cd, 0x9ec6, 0xef42, 0xf84c],
    },
    MarkerCode {
        id: 6,
        rotations: [0x9e2e, 0x9884, 0x9742, 0x7c10],
    },
    MarkerCode {
        id: 7,
        rotations: [0xc4f2, 0xc61f, 0x7742, 0x7c31],
    },
];

const RESULTS_IDS: [i32; 4] = [0, 1, 2, 3];
const CHOICES_IDS: [i32; 4] = [4, 5, 6, 7];

pub fn detect_tables_and_align(img: &DynamicImage) -> Result<(DynamicImage, Vec<TableLocation>)> {
    let mut gray_img = img.to_luma8();
    let mut otsu_val = contrast::otsu_level(&gray_img);
    let mut thr_img = contrast::threshold(&gray_img, otsu_val, contrast::ThresholdType::Binary);
    let mut markers = detect_uruco(&thr_img)?;

    let skew_angle = get_skew_angle(&markers);
    let aligned_img = if skew_angle.abs() > 0.01 {
        let rgba_img = img.to_rgba8();
        
        let rotated_rgba = rotate_about_center(
            &rgba_img,
            -skew_angle,
            Interpolation::Bicubic,
            Rgba([255; 4])
        );
        DynamicImage::ImageRgba8(rotated_rgba)
    } else { 
        img.clone()
    };
    
    if skew_angle.abs() > 0.01 {
        gray_img = aligned_img.to_luma8();
        otsu_val = contrast::otsu_level(&gray_img);
        thr_img = contrast::threshold(&gray_img, otsu_val, contrast::ThresholdType::Binary);
        markers = detect_uruco(&thr_img)?;   
    }

    let regions = recognise_markers(markers)?;

    let mut locs: Vec<TableLocation> = Vec::new();
    for (table_type, (x1, x2, y1, y2)) in regions {
        locs.push(TableLocation {
            table_type,
            x: x1 as u32,
            y: y1 as u32,
            width: (x2 - x1) as u32,
            height: (y2 - y1) as u32,
        })
    }
    Ok((aligned_img, locs))
}

// =====================================================================
// Skew Calculation
// =====================================================================

fn get_skew_angle(markers: &[(Vec<Point<i32>>, i32)]) -> f32 {
    let mut angles = Vec::new();
    
    let horizontal_pairs = [(0,1), (2,3), (4,5), (6,7)];
    
    for (left_id, right_id) in horizontal_pairs {
        let mut left_centers: Vec<Point<i32>> = markers.iter()
            .filter(|(_, id)| *id == left_id)
            .map(|(corners, _)| get_marker_center(corners))
            .collect();
        left_centers.sort_by_key(|p| p.y);
        
        let mut right_centers: Vec<Point<i32>> = markers.iter()
            .filter(|(_, id)| *id == right_id)
            .map(|(corners, _)| get_marker_center(corners))
            .collect();
        right_centers.sort_by_key(|p| p.y);
        
        for (left_center, right_center) in left_centers.iter().zip(right_centers.iter()) {
            let dy = (right_center.y - left_center.y) as f32;
            let dx = (right_center.x - left_center.x) as f32;
            angles.push(dy.atan2(dx));
        }
    }
    
    if angles.is_empty() {
        return 0.0;   
    }
    
    let sum: f32 = angles.iter().sum();
    sum / angles.len() as f32
}

// =====================================================================
// ArUco Detection & Decoding
// =====================================================================
fn detect_uruco(img: &GrayImage) -> Result<Vec<(Vec<Point<i32>>, i32)>> {
    let candidates = find_marker_candidates(img)?;

    let mut markers = Vec::new();

    for candidate in candidates {
        let area = calc_area(&candidate)?;
        if area > 1000.0 {
            let code = extract_bits(img, &candidate)?;
            let id = match_marker(code);

            if id.is_some() {
                let (id, rot_idx) = id.unwrap();
                markers.push((candidate, id));
            }
        }
    }

    Ok(markers)
}

fn find_marker_candidates(img: &GrayImage) -> Result<Vec<Vec<Point<i32>>>> {
    let contours = contours::find_contours::<i32>(img);
    let mut candidates = Vec::new();

    for contour in contours {
        let poly = geometry::approximate_polygon_dp(&contour.points, 10.0, true);

        if poly.len() == 4 {
            candidates.push(poly);
        }
    }

    Ok(candidates)
}

fn extract_bits(img: &GrayImage, corners: &[Point<i32>]) -> Result<u16> {
    let corners = sort_corners(corners);
    let tgt_size = 60u32;

    let target_corners = [
        (0.0, 0.0),
        (tgt_size as f32, 0.0),
        (tgt_size as f32, tgt_size as f32),
        (0.0, tgt_size as f32),
    ];

    let src_corners = [
        (corners[0].x as f32, corners[0].y as f32),
        (corners[1].x as f32, corners[1].y as f32),
        (corners[2].x as f32, corners[2].y as f32),
        (corners[3].x as f32, corners[3].y as f32),
    ];

    let projection = Projection::from_control_points(src_corners, target_corners).unwrap();
    let mut warped_img = GrayImage::new(tgt_size, tgt_size);
    warp_into(
        img,
        &projection,
        Interpolation::Bicubic,
        Luma([0u8]),
        &mut warped_img,
    );

    let cell_size = tgt_size / 6;

    let mut code: u16 = 0;

    for row in 1..5 {
        for col in 1..5 {
            let x1 = col * cell_size;
            let y1 = row * cell_size;
            let x2 = x1 + cell_size;
            let y2 = y1 + cell_size;

            let mut avg: f32 = 0.0;

            let margin = cell_size / 4;

            for x in x1 + margin..x2 - margin {
                for y in y1 + margin..y2 - margin {
                    let pxl = warped_img.get_pixel(x, y).0[0];
                    avg += pxl as f32;
                }
            }

            avg /= ((cell_size - margin) * (cell_size - margin)) as f32;

            code <<= 1;
            if avg > 128.0 {
                code |= 1;
            }
        }
    }

    Ok(code)
}

fn sort_corners(corners: &[Point<i32>]) -> [Point<i32>; 4] {
    let cx = corners.iter().map(|p| p.x).sum::<i32>() / 4;
    let cy = corners.iter().map(|p| p.y).sum::<i32>() / 4;

    let mut sorted = [corners[0], corners[1], corners[2], corners[3]];
    sorted.sort_by(|a, b| {
        let angle_a = ((a.y - cy) as f32).atan2((a.x - cx) as f32);
        let angle_b = ((b.y - cy) as f32).atan2((b.x - cx) as f32);
        angle_a.partial_cmp(&angle_b).unwrap()
    });

    sorted
}

fn match_marker(sampled_bits: u16) -> Option<(i32, usize)> {
    for marker in DICT_4X4_8 {
        for (rot_idx, &code) in marker.rotations.iter().enumerate() {
            if code == sampled_bits {
                return Some((marker.id, rot_idx));
            }
        }
    }
    None
}

// =====================================================================
// Table Grouping & Boundary Math
// =====================================================================

fn recognise_markers(
    markers: Vec<(Vec<Point<i32>>, i32)>,
) -> Result<Vec<(i32, (i32, i32, i32, i32))>> {
    let mut regions = Vec::new();

    let results_pool: Vec<_> = markers
        .iter()
        .filter(|(_, id)| RESULTS_IDS.contains(id))
        .cloned()
        .collect();
    let choices_pool: Vec<_> = markers
        .iter()
        .filter(|(_, id)| CHOICES_IDS.contains(id))
        .cloned()
        .collect();

    let results_regions = group_regions(&results_pool, &[0, 1], &[2, 3]);
    for region in results_regions {
        if let Some(region_boundaries) = extract_region_boundaries(&region) {
            regions.push((0, region_boundaries));
        }
    }

    let choices_regions = group_regions(&choices_pool, &[4, 5], &[6, 7]);
    for region in choices_regions {
        if let Some(region_boundaries) = extract_region_boundaries(&region) {
            regions.push((1, region_boundaries));
        }
    }

    Ok(regions)
}

fn group_regions(
    markers: &[(Vec<Point<i32>>, i32)],
    top_ids: &[i32],
    bottom_ids: &[i32],
) -> Vec<Vec<(Vec<Point<i32>>, i32)>> {
    let top_markers: Vec<_> = markers
        .iter()
        .filter(|(_, id)| top_ids.contains(id))
        .cloned()
        .collect();
    let bottom_markers: Vec<_> = markers
        .iter()
        .filter(|(_, id)| bottom_ids.contains(id))
        .cloned()
        .collect();

    let top_rows = cluster_y(&top_markers);
    let bottom_rows = cluster_y(&bottom_markers);

    let mut regions = Vec::new();

    for top_row in top_rows {
        let t_y = average_y(&top_row);
        let mut best_bottom_row = None;
        let mut min_dist = i32::MAX;

        for bottom_row in &bottom_rows {
            let b_y = average_y(bottom_row);
            if b_y > t_y && (b_y - t_y) < min_dist {
                min_dist = b_y - t_y;
                best_bottom_row = Some(bottom_row.clone());
            }
        }

        if let Some(bottom_row) = best_bottom_row {
            let mut region = top_row.clone();
            region.extend(bottom_row);
            regions.push(region);
        }
    }

    regions
}

fn cluster_y(markers: &[(Vec<Point<i32>>, i32)]) -> Vec<Vec<(Vec<Point<i32>>, i32)>> {
    if markers.is_empty() {
        return vec![];
    }

    let mut sorted = markers.to_vec();
    sorted.sort_by_key(|(corners, _)| get_marker_center(corners).y);

    let mut clusters = Vec::new();
    let mut current_cluster = vec![sorted[0].clone()];
    let mut current_y = average_y(&current_cluster);

    let y_threshold = 150;

    for marker in sorted.iter().skip(1) {
        let y = get_marker_center(&marker.0).y;

        if (y - current_y).abs() < y_threshold {
            current_cluster.push(marker.clone());
            current_y = average_y(&current_cluster);
        } else {
            clusters.push(current_cluster);
            current_cluster = vec![marker.clone()];
            current_y = average_y(&current_cluster);
        }
    }
    clusters.push(current_cluster);
    clusters
}

fn extract_region_boundaries(markers: &[(Vec<Point<i32>>, i32)]) -> Option<(i32, i32, i32, i32)> {
    let mut left = Vec::new();
    let mut right = Vec::new();
    let mut top = Vec::new();
    let mut bottom = Vec::new();

    for (corners, id) in markers {
        if matches!(id, 0 | 2 | 4 | 6) {
            left.push(corners.iter().map(|p| p.x).max().unwrap_or(0));
        }
        if matches!(id, 1 | 3 | 5 | 7) {
            right.push(corners.iter().map(|p| p.x).min().unwrap_or(0));
        }
        if matches!(id, 0 | 1 | 4 | 5) {
            top.push(corners.iter().map(|p| p.y).max().unwrap_or(0));
        }
        if matches!(id, 2 | 3 | 6 | 7) {
            bottom.push(corners.iter().map(|p| p.y).min().unwrap_or(0));
        }
    }

    let inner_min_x = left.into_iter().max()?;
    let inner_max_x = right.into_iter().min()?;
    let inner_min_y = top.into_iter().max()?;
    let inner_max_y = bottom.into_iter().min()?;

    if inner_min_x >= 0
        && inner_min_y >= 0
        && inner_max_x > inner_min_x
        && inner_max_y > inner_min_y
    {
        Some((inner_min_x, inner_max_x, inner_min_y, inner_max_y))
    } else {
        None
    }
}

// =====================================================================
// Math helpers
// =====================================================================

fn calc_area(p: &[Point<i32>]) -> Result<f32> {
    let s = (p[0].x * p[1].y - p[1].x * p[0].y + p[1].x * p[2].y - p[2].x * p[1].y
        + p[2].x * p[3].y
        - p[3].x * p[2].y
        + p[3].x * p[0].y
        - p[0].x * p[3].y)
        .abs() as f32
        / 2.0;

    Ok(s)
}

fn get_marker_center(corners: &[Point<i32>]) -> Point<i32> {
    let x = corners.iter().map(|p| p.x).sum::<i32>() / corners.len() as i32;
    let y = corners.iter().map(|p| p.y).sum::<i32>() / corners.len() as i32;
    Point::new(x, y)
}

fn average_y(cluster: &[(Vec<Point<i32>>, i32)]) -> i32 {
    if cluster.is_empty() {
        return 0;
    }

    let sum: i32 = cluster.iter().map(|(c, _)| get_marker_center(c).y).sum();
    sum / cluster.len() as i32
}
