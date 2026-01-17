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

                let proper_distance = median > current_min_distance
                    && minimum > current_min_distance / 3.0
                    && distances[0] > current_min_distance * 2.0
                    && distances.iter().all(|&d| d < max_distance);

                if proper_distance {
                    // background distance needs to be a liiiittle extra
                    let background_dist = color_diff_lab(&prospect_lab_value, &color_lab_values[0]);
                    if background_dist < current_min_distance * 1.5 {
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

    select_from_clusters(dominant_color, sorted_clusters, min_distance, max_distance)
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

        let result = select_from_clusters(dominant, clusters, 5.0, 200.0);
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

        let result = k_cluster(&pixels_from_all_white_image, 60, 1, 10.0, 200.0);
        println!("{:?}", result);
        assert!(result.is_ok());
        let colors = result.unwrap();
        assert_eq!(colors.len(), 16);
    }
}
