// # TODO: test images & compare w wezterm implementation

mod base16;
mod cluster;
mod io;

use base16::Base16Scheme;
use clap::Parser;
use std::path::Path;

#[derive(Parser)]
#[command(name = "quicktheme")]
#[command(about = "Generate color themes from images", long_about = None)]
struct Args {
    /// Source image filepath (required)
    #[arg(short = 'f', long, value_name = "FILE")]
    source_file: String,

    /// Theme name (filename and "scheme" field in yaml -- defaults to source file name)
    #[arg(short = 't', long)]
    theme_name: Option<String>,

    /// Output directory (defaults to stdout)
    #[arg(short = 'o', long)]
    output_directory: Option<String>,

    /// Author name (defaults to "Anonymous")
    #[arg(short = 'a', long)]
    author: Option<String>,

    /// Seed to control or reproduce random behavior
    #[arg(short = 'r', long, default_value_t = 68)]
    seed: u64,

    /// Minimum color distance for clustering (defaults to 10.0)
    #[arg(short = 'm', long, default_value_t = 10.0)]
    min_distance: f32,

    /// Maximum color distance for clustering (defaults to 100.0)
    #[arg(short = 'M', long, default_value_t = 100.0)]
    max_distance: f32,

    /// Disable printing of the color grid
    #[arg(long)]
    no_grid: bool,

    /// Output mode: "swap" to invert background/foreground, "scramble" to randomize colors
    #[arg(long, value_parser = ["swap", "scramble"])]
    output_mode: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let filename = &args.source_file;
    let output_dir = args
        .output_directory
        .unwrap_or_else(|| "stdout".to_string());
    let theme_name = args.theme_name.clone().unwrap_or_else(|| {
        Path::new(filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("theme")
            .to_string()
    });
    let author = args.author.unwrap_or_else(|| "Anonymous".to_string());
    let n_clusters = 64;
    let seed = args.seed;

    io::validate_output_path(&output_dir)?;

    let pixels = io::load_image(filename)?;
    let mut colors = cluster::k_cluster(
        &pixels,
        n_clusters,
        seed,
        args.min_distance,
        args.max_distance,
    )?;

    // Apply output mode transformations
    if let Some(mode) = &args.output_mode {
        match mode.as_str() {
            "swap" => {
                colors.swap(0, 1);
            }
            "scramble" => {
                use rand::SeedableRng;
                use rand::seq::SliceRandom;
                let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
                colors.shuffle(&mut rng);
            }
            _ => {}
        }
    }

    // Build command string with all parameters used
    let command = format!(
        "quicktheme -f {} -a \"{}\" -r {} -m {} -M {}{}{}{}",
        filename,
        author,
        seed,
        args.min_distance,
        args.max_distance,
        args.theme_name
            .as_ref()
            .map(|t| format!(" -t \"{}\"", t))
            .unwrap_or_default(),
        args.output_mode
            .as_ref()
            .map(|m| format!(" --output-mode {}", m))
            .unwrap_or_default(),
        if output_dir != "stdout" {
            format!(" -o \"{}\"", output_dir)
        } else {
            String::new()
        }
    );

    let base16_scheme = Base16Scheme::new(theme_name.clone(), author, command, &colors)?;
    let yaml_lines = base16_scheme.to_yaml();

    if output_dir == "stdout" {
        for line in yaml_lines {
            println!("{}", line);
        }
    } else {
        io::save_yaml(&yaml_lines, &theme_name, &output_dir)?;
        println!("Theme saved to {}/{}.yaml", output_dir, theme_name);
    };

    if !args.no_grid {
        io::print_color_grid(&base16_scheme);
    }

    Ok(())
}
