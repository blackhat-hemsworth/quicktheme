# quicktheme

A Rust CLI for extracting Base16 color schemes from images using K-means clustering.

## Overview

quicktheme analyzes an image and extracts 16 perceptually distinct colors using two-stage K-means clustering in Lab color space. Colors are selected with configurable distance constraints (Delta E 2000) and output as a Base16 color scheme in YAML format.

## Features

- **Fast image processing** — Rust performance on images auto-resized to 256×256
- **Perceptual clustering** — Two-stage K-means in Lab color space (4 clusters for dominant color, 128 for detail)
- **Distance-based selection** — Delta E 2000 ensures selected colors are perceptually distinct
- **Base16 output** — Generates theme files compatible with Base16 color schemes
- **Reproducible** — Seeded random behavior for consistent results

## Dependencies

- `image` — Image loading and processing
- `kmeans_colors` — Hamerly-optimized K-means clustering
- `palette` — Color space conversions (RGB ↔ Lab)
- `clap` — CLI argument parsing
- `deltae` — Delta E 2000 perceptual color distance
- `rand` — Seeded random number generation

## Building

```bash
cargo build --release
```

## Usage

```bash
# Basic usage — outputs Base16 YAML to stdout
quicktheme -f path/to/image.jpg

# With options
quicktheme -f image.jpg -t my_theme -a "Author" -o ./themes -r 42 -m 15.0 -M 80.0
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-f, --source-file` | Source image path (required) | — |
| `-t, --theme-name` | Scheme name | image filename |
| `-o, --output-directory` | Save directory (omit for stdout) | stdout |
| `-a, --author` | Author attribution | "Anonymous" |
| `-r, --seed` | Random seed for reproducibility | 68 |
| `-m, --min-distance` | Minimum perceptual distance between colors | 12.0 |
| `-M, --max-distance` | Maximum perceptual distance cap | 100.0 |
| `--no-grid` | Disable ANSI color grid preview | — |
| `--output-mode` | `swap` (invert bg/fg) or `scramble` (randomize) | — |

## Output Format

Generated YAML files follow the Base16 specification:

```yaml
scheme: my_theme
author: Author
command: quicktheme -f image.jpg -a 'Author' -r 42 -m 15 -M 80 -t "my_theme"
base00: baa194
base01: 887e49
...
base0F: 9b8987
```
