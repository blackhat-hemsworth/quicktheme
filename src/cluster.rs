use deltae::LabValue;
use kmeans_colors::{CentroidData, Sort, get_kmeans_hamerly};
use palette::{IntoColor, Lab, Srgb};

// stolen from wezterm :P
fn color_diff_lab(a: &LabValue, b: &LabValue) -> f32 {
    *deltae::DeltaE::new(a, b, deltae::DEMethod::DE2000).value()
}

pub(crate) fn select_from_clusters(
    dominant: Lab,
    clusters: Vec<CentroidData<Lab>>,
    min_distance: f32,
    max_distance: f32,
    min_lightness_contrast: f32,
) -> Result<Vec<Srgb<u8>>, Box<dyn std::error::Error>> {
    let dominant_rgb: Srgb = dominant.into_color();
    let dominant_srgb = Srgb::new(
        (dominant_rgb.red * 255.0).round() as u8,
        (dominant_rgb.green * 255.0).round() as u8,
        (dominant_rgb.blue * 255.0).round() as u8,
    );

    let mut colors = vec![dominant_srgb];
    let mut color_lab_values = vec![LabValue {
        l: dominant.l,
        a: dominant.a,
        b: dominant.b,
    }];

    let mut foreground_picked = false;
    let mut current_min_distance = min_distance;
    loop {
        if current_min_distance < 0.0 {
            return Err(format!(
                "Insufficient distinct colors found: expected at least 16, but only found {}.",
                colors.len()
            )
            .into());
        } else {
            for prospect in clusters.iter() {
                if colors.len() >= 16 {
                    return Ok(colors);
                }
                let prospect_lab_value = LabValue {
                    l: prospect.centroid.l,
                    a: prospect.centroid.a,
                    b: prospect.centroid.b,
                };

                let distances: Vec<f32> = color_lab_values
                    .iter()
                    .map(|existing_lab_value| {
                        color_diff_lab(&prospect_lab_value, existing_lab_value)
                    })
                    .collect();

                let mut sorted_distances = distances.clone();
                sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let median = sorted_distances[sorted_distances.len() / 2];
                let minimum = sorted_distances[0];

                let lightness_contrast =
                    (prospect_lab_value.l - color_lab_values[0].l).abs();
                let proper_distance = median > current_min_distance
                    && minimum > current_min_distance / 3.0
                    && distances[0] > current_min_distance * 2.0
                    && distances.iter().all(|&d| d < max_distance)
                    && lightness_contrast >= min_lightness_contrast;

                if proper_distance {
                    // background distance needs to be a liiiittle extra
                    let background_dist = color_diff_lab(&prospect_lab_value, &color_lab_values[0]);
                    if background_dist < current_min_distance * 2.0 {
                        break;
                    }

                    let rgb: Srgb = prospect.centroid.into_color();
                    let color = Srgb::new(
                        (rgb.red * 255.0).round() as u8,
                        (rgb.green * 255.0).round() as u8,
                        (rgb.blue * 255.0).round() as u8,
                    );
                    if !foreground_picked
                        && color_diff_lab(&prospect_lab_value, &color_lab_values[0])
                            >= current_min_distance * 3.0
                    {
                        colors.insert(1, color);
                        color_lab_values.insert(1, prospect_lab_value);
                        foreground_picked = true;
                    } else {
                        colors.push(color);
                        color_lab_values.push(prospect_lab_value);
                    }
                }
            }
        }
        current_min_distance -= 2.0;
    }
}

pub fn k_cluster(
    pixels: &[Lab],
    n_clusters: usize,
    seed: u64,
    min_distance: f32,
    max_distance: f32,
    min_lightness_contrast: f32,
) -> Result<Vec<Srgb<u8>>, Box<dyn std::error::Error>> {
    let max_runs = 1;
    let converge = 5.0;

    let white = LabValue {
        l: 100.0,
        a: 0.0,
        b: 0.0,
    };

    let dominant_result = get_kmeans_hamerly(4, max_runs, converge, false, pixels, seed);
    let mut dominant_sorted =
        Lab::sort_indexed_colors(&dominant_result.centroids, &dominant_result.indices);
    dominant_sorted.sort_unstable_by(|a, b| b.percentage.total_cmp(&a.percentage));

    // Find the first color that is visually distinct from both black and white
    let mut dominant_color = dominant_sorted[0].centroid; // Fallback to most frequent
    for option in &dominant_sorted {
        let option_lab = LabValue {
            l: option.centroid.l,
            a: option.centroid.a,
            b: option.centroid.b,
        };

        // sorry, sickos
        if color_diff_lab(&option_lab, &white) > 20.0 {
            dominant_color = option.centroid;
            break;
        }
    }

    let clusters = get_kmeans_hamerly(n_clusters, max_runs, converge, false, pixels, seed);
    let mut sorted_clusters = Lab::sort_indexed_colors(&clusters.centroids, &clusters.indices);
    sorted_clusters.sort_unstable_by(|a, b| b.percentage.total_cmp(&a.percentage));

    select_from_clusters(
        dominant_color,
        sorted_clusters,
        min_distance,
        max_distance,
        min_lightness_contrast,
    )
}

pub fn nudge_lightness(colors: &mut [Srgb<u8>], min_lightness_contrast: f32) {
    if colors.len() < 3 || min_lightness_contrast <= 0.0 {
        return;
    }

    let bg_srgb: Srgb<f32> = Srgb::new(
        colors[0].red as f32 / 255.0,
        colors[0].green as f32 / 255.0,
        colors[0].blue as f32 / 255.0,
    );
    let bg_lab: Lab = bg_srgb.into_color();
    let bg_l = bg_lab.l;

    // Nudge accent colors (indices 2..end), skip foreground (index 1)
    for color in colors.iter_mut().skip(2) {
        let srgb_f: Srgb<f32> = Srgb::new(
            color.red as f32 / 255.0,
            color.green as f32 / 255.0,
            color.blue as f32 / 255.0,
        );
        let mut lab: Lab = srgb_f.into_color();
        let diff = (lab.l - bg_l).abs();

        if diff < min_lightness_contrast {
            let needed = min_lightness_contrast - diff;
            if bg_l < 50.0 {
                lab.l = (lab.l + needed).min(100.0);
            } else {
                lab.l = (lab.l - needed).max(0.0);
            }

            let rgb: Srgb = lab.into_color();
            *color = Srgb::new(
                (rgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (rgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_centroid(
        l: f32,
        a: f32,
        b: f32,
        percentage: f32,
        index: u8,
    ) -> CentroidData<Lab> {
        CentroidData {
            centroid: Lab::new(l, a, b),
            percentage,
            index,
        }
    }

    #[test]
    fn test_select_from_clusters() {
        let dominant = Lab::new(50.0, 0.0, 0.0);
        let dominant_rgb: Srgb = dominant.into_color();
        let dominant_srgb = Srgb::new(
            (dominant_rgb.red * 255.0).round() as u8,
            (dominant_rgb.green * 255.0).round() as u8,
            (dominant_rgb.blue * 255.0).round() as u8,
        );
        let mut clusters = vec![];
        for i in 0..25 {
            let l = 20.0 + (i as f32 * 3.0);
            let a = (i as f32 * 40.0).sin() * 35.0;
            let b = (i as f32 * 40.0).cos() * 35.0;
            clusters.push(create_test_centroid(
                l,
                a,
                b,
                1.0 / (i as f32 + 1.0),
                i as u8,
            ));
        }

        let result = select_from_clusters(dominant, clusters, 5.0, 200.0, 0.0);
        assert!(result.is_ok());
        let colors = result.unwrap();
        assert_eq!(colors.len(), 16);
        assert_eq!(colors[0], dominant_srgb);
    }

    #[test]
    fn test_color_diff_lab() {
        // Test the color difference function
        let black = LabValue {
            l: 0.0,
            a: 0.0,
            b: 0.0,
        };
        let white = LabValue {
            l: 100.0,
            a: 0.0,
            b: 0.0,
        };
        let gray = LabValue {
            l: 50.0,
            a: 0.0,
            b: 0.0,
        };

        let dist_bw = color_diff_lab(&black, &white);
        assert!(dist_bw > 50.0);

        let dist_bg = color_diff_lab(&black, &gray);
        assert!(dist_bg < dist_bw);

        let dist_same = color_diff_lab(&black, &black);
        assert_eq!(dist_same, 0.0);
    }

    #[test]
    fn test_all_white_works_but_complains() {
        let mut pixels_from_all_white_image = vec![Srgb::new(1.0, 1.0, 1.0).into_color()];
        for n in 1..20 {
            pixels_from_all_white_image
                .push(Srgb::new(1.0 - n as f32 * 0.00001, 1.0, 1.0).into_color());
        }
        println!("{:?}", pixels_from_all_white_image);

        let result = k_cluster(&pixels_from_all_white_image, 60, 1, 10.0, 200.0, 0.0);
        println!("{:?}", result);
        assert!(result.is_ok());
        let colors = result.unwrap();
        assert_eq!(colors.len(), 16);
    }

    #[test]
    fn test_lightness_contrast_rejects_similar_l() {
        // Background at L=30, candidates with different hues but similar L values
        let dominant = Lab::new(30.0, 0.0, 0.0);
        let mut clusters = vec![];

        // First add a high-contrast foreground candidate
        clusters.push(create_test_centroid(80.0, 10.0, 10.0, 0.5, 0));

        // Add candidates with similar L to background but different hues
        for i in 1..20 {
            let a = (i as f32 * 2.0).sin() * 50.0;
            let b = (i as f32 * 2.0).cos() * 50.0;
            clusters.push(create_test_centroid(32.0, a, b, 0.3 / i as f32, i as u8));
        }

        // With high lightness contrast requirement, these should be rejected
        let result = select_from_clusters(dominant, clusters, 5.0, 200.0, 25.0);
        assert!(result.is_err() || result.as_ref().unwrap().len() < 16);
    }

    #[test]
    fn test_nudge_lightness_dark_background() {
        // Dark background (L ~20), accent close to background should be pushed up
        let bg: Srgb = Lab::new(20.0, 0.0, 0.0).into_color();
        let fg: Srgb = Lab::new(80.0, 0.0, 0.0).into_color();
        let accent: Srgb = Lab::new(25.0, 30.0, 10.0).into_color();

        let mut colors = vec![
            Srgb::new(
                (bg.red * 255.0).round() as u8,
                (bg.green * 255.0).round() as u8,
                (bg.blue * 255.0).round() as u8,
            ),
            Srgb::new(
                (fg.red * 255.0).round() as u8,
                (fg.green * 255.0).round() as u8,
                (fg.blue * 255.0).round() as u8,
            ),
            Srgb::new(
                (accent.red * 255.0).round() as u8,
                (accent.green * 255.0).round() as u8,
                (accent.blue * 255.0).round() as u8,
            ),
        ];

        nudge_lightness(&mut colors, 25.0);

        // Convert nudged accent back to Lab and check L* increased
        let nudged_srgb: Srgb<f32> = Srgb::new(
            colors[2].red as f32 / 255.0,
            colors[2].green as f32 / 255.0,
            colors[2].blue as f32 / 255.0,
        );
        let nudged_lab: Lab = nudged_srgb.into_color();
        let bg_lab: Lab = Srgb::new(
            colors[0].red as f32 / 255.0,
            colors[0].green as f32 / 255.0,
            colors[0].blue as f32 / 255.0,
        )
        .into_color();
        assert!(
            (nudged_lab.l - bg_lab.l).abs() >= 24.0,
            "Expected L* diff >= 24, got {}",
            (nudged_lab.l - bg_lab.l).abs()
        );
    }

    #[test]
    fn test_nudge_lightness_light_background() {
        // Light background (L ~80), accent close to background should be pushed down
        let bg: Srgb = Lab::new(80.0, 0.0, 0.0).into_color();
        let fg: Srgb = Lab::new(20.0, 0.0, 0.0).into_color();
        let accent: Srgb = Lab::new(75.0, -20.0, 15.0).into_color();

        let mut colors = vec![
            Srgb::new(
                (bg.red * 255.0).round() as u8,
                (bg.green * 255.0).round() as u8,
                (bg.blue * 255.0).round() as u8,
            ),
            Srgb::new(
                (fg.red * 255.0).round() as u8,
                (fg.green * 255.0).round() as u8,
                (fg.blue * 255.0).round() as u8,
            ),
            Srgb::new(
                (accent.red * 255.0).round() as u8,
                (accent.green * 255.0).round() as u8,
                (accent.blue * 255.0).round() as u8,
            ),
        ];

        nudge_lightness(&mut colors, 25.0);

        let nudged_srgb: Srgb<f32> = Srgb::new(
            colors[2].red as f32 / 255.0,
            colors[2].green as f32 / 255.0,
            colors[2].blue as f32 / 255.0,
        );
        let nudged_lab: Lab = nudged_srgb.into_color();
        let bg_lab: Lab = Srgb::new(
            colors[0].red as f32 / 255.0,
            colors[0].green as f32 / 255.0,
            colors[0].blue as f32 / 255.0,
        )
        .into_color();
        // Accent should have been pushed down (lower L*)
        assert!(
            nudged_lab.l < bg_lab.l,
            "Expected accent L* ({}) < bg L* ({})",
            nudged_lab.l,
            bg_lab.l
        );
        assert!(
            (nudged_lab.l - bg_lab.l).abs() >= 24.0,
            "Expected L* diff >= 24, got {}",
            (nudged_lab.l - bg_lab.l).abs()
        );
    }
}
