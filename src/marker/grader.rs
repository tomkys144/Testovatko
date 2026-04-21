use image::{DynamicImage, GenericImageView, GrayImage, Rgb, load};
use imageproc::contrast;
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use nalgebra::RealField;

pub fn mark_section(img: &DynamicImage) {
    let gray_img = img.to_luma8();
    let otsu_val = contrast::otsu_level(&gray_img);
    let thr_img = contrast::threshold(&gray_img, otsu_val, contrast::ThresholdType::Binary);
    let columns = detect_cells(&thr_img);
    debug_save_cells(img, &columns, "debug_cells.png");

    for question in columns.iter().skip(1) {
        let selected = detect_marked_sections(img, question);
        let selected_options = selected
            .into_iter()
            .enumerate()
            .filter(|(_, is_marked)| *is_marked)
            .map(|(i, _)| (b'A' + i as u8) as char)
            .collect::<String>();
    }
}

fn detect_marked_sections(img: &DynamicImage, cells: &[(u32, u32, u32, u32)]) -> Vec<bool> {
    let mut selected_cells = vec![false; cells.len() - 1];

    for (cell_idx, cell) in cells.iter().enumerate().skip(1) {
        let margin_width = (cell.1 - cell.0) / 10;
        let margin_height = (cell.3 - cell.2) / 10;

        let inner_width = (cell.1 - cell.0) - 2 * margin_width;
        let inner_height = (cell.3 - cell.2) - 2 * margin_height;

        let sub_step_w = (cell.1 - cell.0 - 2 * margin_width) / 3;
        let sub_step_h = (cell.3 - cell.2 - 2 * margin_height) / 3;

        let mut grid_mass = [0.0; 9];
        let mut pixels_per_region = [0.0; 9];

        for g_x in 0..3 {
            for g_y in 0..3 {
                let g_x_coord = g_x * sub_step_w + margin_width + cell.0;
                let g_y_coord = g_y * sub_step_h + margin_height + cell.2;

                for x in g_x_coord..g_x_coord + sub_step_w {
                    for y in g_y_coord..g_y_coord + sub_step_h {
                        grid_mass[(g_x * 3 + g_y) as usize] +=
                            255.0 - img.get_pixel(x, y).0[0] as f32;
                        pixels_per_region[(g_x * 3 + g_y) as usize] += 1.0;
                    }
                }
            }
        }

        let mut region_fills = [0.0; 9];
        for i in 0..9 {
            if pixels_per_region[i] > 0.0 {
                let max_mass = pixels_per_region[i] * 255.0;
                region_fills[i] = grid_mass[i] / max_mass;
            }
        }

        let mut clusters = Vec::new();

        let max_fill = region_fills.iter().copied().fold(0.0, f32::max);

        if max_fill < 0.04 {
            continue; //Even darkest region is too light
        }

        if region_fills.iter().copied().fold(0.0, f32::min) > 0.5 {
            continue;
        }

        let alpha = (max_fill * 0.2).max(0.05);

        for region in region_fills {
            if clusters.len() < 1 {
                clusters.push(region);

                continue;
            }

            let mut found = false;

            for (idx, fill) in clusters.iter().enumerate() {
                if (fill - region).abs() < alpha {
                    clusters[idx] = (fill + region) / 2.0;
                    found = true;
                    break;
                }
            }

            if !found {
                clusters.push(region);
            }
        }

        if clusters.len() > 1 {
            selected_cells[cell_idx - 1] = true;
        }
    }

    selected_cells
}

fn detect_cells(img: &GrayImage) -> Vec<Vec<(u32, u32, u32, u32)>> {
    let (col_sum, row_sum) = sum_cols_rows(&img, 3);

    let thr_width = (img.width() as f32 * 0.80) as u32;
    let thr_height = (img.height() as f32 * 0.80) as u32;

    let x_bounds = find_cell_interiors(&col_sum, thr_height);
    let y_bounds = find_cell_interiors(&row_sum, thr_width);

    let x_bounds = regularize_bounds(&x_bounds);
    let y_bounds = regularize_bounds(&y_bounds);

    let mut columns = Vec::new();

    for &(min_x, max_x) in &x_bounds {
        let mut current_column = Vec::new();

        for &(min_y, max_y) in &y_bounds {
            current_column.push((min_x, max_x, min_y, max_y));
        }

        columns.push(current_column);
    }

    columns
}

fn regularize_bounds(bounds: &[(u32, u32)]) -> Vec<(u32, u32)> {
    if bounds.len() < 3 {
        return bounds.to_vec();
    }

    let mut widths: Vec<u32> = bounds.iter().map(|&(s, e)| e.saturating_sub(s)).collect();
    widths.sort_unstable();
    let median_width = widths[widths.len() / 2];

    let mut gaps: Vec<u32> = bounds
        .windows(2)
        .map(|w| w[1].0.saturating_sub(w[0].1))
        .collect();
    gaps.sort_unstable();
    let median_gap = gaps[gaps.len() / 2];

    let mut fixed_bounds = Vec::new();

    for &(start, end) in bounds {
        let width = end.saturating_sub(start);

        if width < (median_width / 4 * 3) {
            println!("Skipping small column: {} - {}", start, end);
            continue;
        }

        let total_space = (width + median_gap) as f32;
        let perfect_cell_space = (median_width + median_gap) as f32;
        let expected_cells = (total_space / perfect_cell_space).round() as u32;

        if expected_cells > 1 {
            let mut current_start = start;
            let mut calculated_width =
                (width - (median_gap * (expected_cells - 1))) / expected_cells;

            for _ in 0..expected_cells {
                fixed_bounds.push((current_start, current_start + calculated_width));
                current_start += calculated_width + median_gap;
            }
        } else {
            fixed_bounds.push((start, end));
        }
    }

    fixed_bounds
}
fn find_cell_interiors(profile: &[u32], grid_line_thr: u32) -> Vec<(u32, u32)> {
    let mut bounds = Vec::new();
    let mut insde_cell = false;
    let mut start = 0;

    for (i, &blck_count) in profile.iter().enumerate() {
        if blck_count < grid_line_thr && !insde_cell {
            insde_cell = true;
            start = i as u32;
        } else if blck_count >= grid_line_thr && insde_cell {
            insde_cell = false;
            let end = i as u32;

            if end.saturating_sub(start) > 20 {
                bounds.push((start, end));
            }
        }
    }

    if insde_cell {
        let end = profile.len() as u32;
        if end.saturating_sub(start) > 20 {
            bounds.push((start, end));
        }
    }

    bounds
}
fn sum_cols_rows(img: &GrayImage, window_sz: usize) -> (Vec<u32>, Vec<u32>) {
    let width = img.width() as usize;
    let height = img.height() as usize;

    let mut col_sums = vec![0u32; width];
    let mut row_sums = vec![0u32; height];

    let max_gap = 2;

    let mut col_current = vec![0u32; width];
    let mut col_gap = vec![0u32; width];

    for y in 0..height {
        let mut row_current = 0u32;
        let mut row_gap = 0u32;

        for x in 0..width {
            let is_dark = img.get_pixel(x as u32, y as u32).0[0] < 128;

            if is_dark {
                row_current += 1;
                row_gap = 0;
                if row_current > row_sums[y] {
                    row_sums[y] = row_current;
                }
            } else {
                if row_gap < max_gap && row_current > 0 {
                    row_current += 1;
                    row_gap += 1;
                } else {
                    row_current = 0;
                }
            }

            if is_dark {
                col_current[x] += 1;
                col_gap[x] = 0;
                if col_current[x] > col_sums[x] {
                    col_sums[x] = col_current[x];
                }
            } else {
                if col_gap[x] < max_gap && col_current[x] > 0 {
                    col_current[x] += 1;
                    col_gap[x] += 1;
                } else {
                    col_current[x] = 0;
                }
            }
        }
    }

    if window_sz > 0 {
        col_sums = window_avg(&col_sums, window_sz);
        row_sums = window_avg(&row_sums, window_sz);
    }

    (col_sums, row_sums)
}

fn get_answers(img: &mut DynamicImage, cells: &[Vec<(u32, u32, u32, u32)>]) {
    for column in cells.iter().skip(1) {
        for (idx, (min_x, max_x, min_y, max_y)) in column.iter().enumerate().skip(1) {
            let width = max_x - min_x;
            let height = max_y - min_y;

            let cell = img.crop(*min_x, *min_y, width, height);
        }
    }
}

fn window_avg(profile: &[u32], window_sz: usize) -> Vec<u32> {
    let mut result = vec![0u32; profile.len()];
    let half_window = window_sz / 2;

    for i in 0..profile.len() {
        let start = i.saturating_sub(half_window);
        let end = (i + half_window).min(profile.len() - 1);

        result[i] = profile[start..=end].iter().sum();
    }

    result
}

fn debug_save_cells(img: &DynamicImage, columns: &[Vec<(u32, u32, u32, u32)>], filename: &str) {
    let mut debug_img = img.to_rgb8();

    let outline_color = Rgb([255u8, 0u8, 0u8]);

    for cells in columns {
        for &(min_x, max_x, min_y, max_y) in cells {
            let width = max_x - min_x;
            let height = max_y - min_y;

            let rect = Rect::at(min_x as i32, min_y as i32).of_size(width, height);

            draw_hollow_rect_mut(&mut debug_img, rect, outline_color);
        }
    }

    match debug_img.save(format!("/tmp/{}", filename)) {
        Ok(_) => println!("Saved debug cells to: {}", filename),
        Err(e) => println!("Failed to save debug image: {}", e),
    }
}
