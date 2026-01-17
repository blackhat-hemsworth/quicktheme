# QuickThemes

A Rust implementation for extracting color palettes from images using K-means clustering.

## Overview

QuickThemes analyzes an image and extracts the 16 most dominant colors using K-means clustering. The colors are sorted by frequency and output as a Base16 color scheme in YAML format.

## Features

- **Fast image processing** - Uses Rust's performance to quickly process images
- **K-means clustering** - Extracts dominant colors based on Lab color space for perceptually accurate results
- **Base16 output** - Generates theme files compatible with Base16 color schemes
- **Frequency sorting** - Colors are ordered by their frequency in the image

## Dependencies

- `image` - Image loading and processing
- `kmeans_colors` - K-means clustering implementation
- `palette` - Color space conversions (RGB to Lab)

## Building

```bash
cargo build --release
```

## Usage

Currently hardcoded to process `data/WLA_metmuseum_Water_Lilies_by_Claude_Monet.jpg` and output to `themes/test_theme.yaml`.

```bash
cargo run
```

## Output Format

Generated YAML files follow the Base16 specification:

```yaml
scheme: test_theme
author: Anonymous, using quickthemes
base00: baa194
base01: 887e49
...
base0F: 9b8987
```

## Comparison to Python Implementation

This Rust implementation provides the same functionality as the Python version from `image-to-colorscheme-pydemo`:
- Loads images using the `image` crate (equivalent to OpenCV)
- Performs K-means clustering in Lab color space
- Sorts colors by frequency
- Outputs Base16 YAML theme files

Key differences:
- Uses Lab color space for clustering (more perceptually accurate)
- Much faster execution due to Rust's performance
- Native binary with no Python dependencies

## Future Enhancements

- Command-line argument parsing
- Custom theme names and authors
- Adjustable cluster count
- Multiple output formats
- Optional color contrast adjustment
- Visualization output
