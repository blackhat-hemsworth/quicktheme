use palette::{IntoColor, Lab, Srgb};
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn validate_output_path(output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    if output_dir != "stdout" && !Path::new(output_dir).exists() {
        return Err(format!(
            "Error: Output directory '{}' does not exist.\nPlease create the directory or provide a valid path.",
            output_dir
        ).into());
    }
    Ok(())
}

pub fn load_image(filename: &str) -> Result<Vec<Lab>, Box<dyn std::error::Error>> {
    if !Path::new(filename).exists() {
        return Err(format!(
            "Error: Input image '{}' does not exist.\nPlease provide a valid image file path.",
            filename
        )
        .into());
    }

    let img = image::open(filename).map_err(|e| {
        format!(
            "Error: Failed to open image '{}': {}\nPlease ensure the file is a valid image format (PNG, JPEG, etc.)",
            filename, e
        )
    })?
    .resize_exact(256, 256, image::imageops::FilterType::Nearest)
    .to_rgb8();

    let pixels: Vec<Lab> = img
        .pixels()
        .map(|p| {
            let rgb = Srgb::new(
                p[0] as f32 / 255.0,
                p[1] as f32 / 255.0,
                p[2] as f32 / 255.0,
            );
            rgb.into_color()
        })
        .collect();

    Ok(pixels)
}

use crate::base16::Base16Scheme;

pub fn save_yaml(
    lines: &Vec<String>,
    theme_name: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new(output_dir).join(format!("{}.yaml", theme_name));
    let mut file = File::create(output_path)?;

    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

pub fn print_color_grid(scheme: &Base16Scheme) {
    // TODO: save to example_themes
    println!("\nColor Grid");

    // Helper function to parse hex string and print color block
    let print_color = |hex: &str, width: usize| {
        if hex.len() == 6
            && let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            )
        {
            print!("\x1b[48;2;{};{};{}m{}\x1b[0m", r, g, b, " ".repeat(width));
        }
    };

    // Print background (base00)
    print!("background: ");
    print_color(&scheme.base00, 8);
    println!();

    // Print foreground (base05)
    print!("foreground: ");
    print_color(&scheme.base05, 8);
    println!();

    println!();

    // Collect all base colors in array
    let bases = [
        &scheme.base00,
        &scheme.base01,
        &scheme.base02,
        &scheme.base03,
        &scheme.base04,
        &scheme.base05,
        &scheme.base06,
        &scheme.base07,
        &scheme.base08,
        &scheme.base09,
        &scheme.base0A,
        &scheme.base0B,
        &scheme.base0C,
        &scheme.base0D,
        &scheme.base0E,
        &scheme.base0F,
    ];

    // Print 4x4 grid
    for row in 0..4 {
        for col in 0..4 {
            let idx = col * 4 + row;
            print_color(bases[idx], 4);
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_input_path_nonexistent() {
        let result = load_image("nonexistent.png");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Input image"));
        assert!(err.to_string().contains("does not exist"));
        assert!(err.to_string().contains("nonexistent.png"));
    }

    #[test]
    fn test_validate_output_path_nonexistent() {
        let result = validate_output_path("/nonexistent/path");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Output directory"));
        assert!(err.to_string().contains("does not exist"));
        assert!(err.to_string().contains("/nonexistent/path"));
    }

    #[test]
    fn test_validate_output_path_stdout() {
        // stdout should always be valid
        let result = validate_output_path("stdout");
        assert!(result.is_ok());
    }
}
