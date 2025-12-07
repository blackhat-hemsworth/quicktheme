use palette::Srgb;

// copied from wezterm :P
#[derive(Debug)]
#[allow(non_snake_case, dead_code)]
pub struct Base16Scheme {
    pub scheme: String,
    pub author: String,
    pub command: String,
    pub base00: String,
    pub base01: String,
    pub base02: String,
    pub base03: String,
    pub base04: String,
    pub base05: String,
    pub base06: String,
    pub base07: String,
    pub base08: String,
    pub base09: String,
    pub base0A: String,
    pub base0B: String,
    pub base0C: String,
    pub base0D: String,
    pub base0E: String,
    pub base0F: String,
}

impl Base16Scheme {
    pub fn new(
        scheme: String,
        author: String,
        command: String,
        colors: &[Srgb<u8>],
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if colors.len() != 16 {
            return Err(format!("Expected 16 colors, got {}", colors.len()).into());
        }

        let to_hex = |c: &Srgb<u8>| format!("{:02x}{:02x}{:02x}", c.red, c.green, c.blue);

        Ok(Base16Scheme {
            scheme,
            author: format!(
                "{}, using quickthemes @ https://github.com/blackhat-hemsworth/quickthemes",
                author
            ),
            command,
            base00: to_hex(&colors[0]), // background in wez / mini.nvim
            base01: to_hex(&colors[5]),
            base02: to_hex(&colors[2]),
            base03: to_hex(&colors[3]),
            base04: to_hex(&colors[4]),
            base05: to_hex(&colors[1]), // foreground in wez / mini.nvim
            base06: to_hex(&colors[6]),
            base07: to_hex(&colors[7]),
            base08: to_hex(&colors[8]),
            base09: to_hex(&colors[9]),
            base0A: to_hex(&colors[10]),
            base0B: to_hex(&colors[11]),
            base0C: to_hex(&colors[12]),
            base0D: to_hex(&colors[13]),
            base0E: to_hex(&colors[14]),
            base0F: to_hex(&colors[15]),
        })
    }

    pub fn to_yaml(&self) -> Vec<String> {
        // makes per schema def here: https://github.com/chriskempson/base16/blob/main/file.md
        // mini.base16 expects hex, wezterm expects no hex, so defaulted to spec
        vec![
            format!("scheme: \"{}\"", self.scheme),
            format!("author: \"{}\"", self.author),
            format!("command: \"{}\"", self.command),
            format!("base00: \"{}\"", self.base00),
            format!("base01: \"{}\"", self.base01),
            format!("base02: \"{}\"", self.base02),
            format!("base03: \"{}\"", self.base03),
            format!("base04: \"{}\"", self.base04),
            format!("base05: \"{}\"", self.base05),
            format!("base06: \"{}\"", self.base06),
            format!("base07: \"{}\"", self.base07),
            format!("base08: \"{}\"", self.base08),
            format!("base09: \"{}\"", self.base09),
            format!("base0A: \"{}\"", self.base0A),
            format!("base0B: \"{}\"", self.base0B),
            format!("base0C: \"{}\"", self.base0C),
            format!("base0D: \"{}\"", self.base0D),
            format!("base0E: \"{}\"", self.base0E),
            format!("base0F: \"{}\"", self.base0F),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_colors() -> Vec<Srgb<u8>> {
        vec![
            Srgb::new(0, 0, 0),       // base00 - black
            Srgb::new(255, 255, 255), // base01 - white
            Srgb::new(255, 0, 0),     // base02 - red
            Srgb::new(0, 255, 0),     // base03 - green
            Srgb::new(0, 0, 255),     // base04 - blue
            Srgb::new(255, 255, 0),   // base05 - yellow
            Srgb::new(255, 0, 255),   // base06 - magenta
            Srgb::new(0, 255, 255),   // base07 - cyan
            Srgb::new(128, 128, 128), // base08 - gray
            Srgb::new(192, 192, 192), // base09 - light gray
            Srgb::new(64, 64, 64),    // base0A - dark gray
            Srgb::new(128, 0, 0),     // base0B - dark red
            Srgb::new(0, 128, 0),     // base0C - dark green
            Srgb::new(0, 0, 128),     // base0D - dark blue
            Srgb::new(128, 128, 0),   // base0E - dark yellow
            Srgb::new(128, 0, 128),   // base0F - dark magenta
        ]
    }

    #[test]
    fn test_base16_scheme_new() {
        let colors = create_test_colors();
        let scheme = Base16Scheme::new(
            "test-scheme".to_string(),
            "Test Author".to_string(),
            "quicktheme -f test.png -a \"Test Author\" -r 42 -m 10 -M 100".to_string(),
            &colors,
        );

        assert!(scheme.is_ok());
        let scheme = scheme.unwrap();
        assert_eq!(scheme.scheme, "test-scheme");
        assert!(scheme.author.contains("Test Author"));
        assert!(scheme.author.contains("quickthemes"));
    }

    #[test]
    fn test_base16_scheme_invalid_color_count() {
        let colors = vec![Srgb::new(0, 0, 0); 10]; // Only 10 colors
        let scheme = Base16Scheme::new(
            "test-scheme".to_string(),
            "Test Author".to_string(),
            "test command".to_string(),
            &colors,
        );

        assert!(scheme.is_err());
        assert!(scheme.unwrap_err().to_string().contains("Expected 16 colors"));
    }

    #[test]
    fn test_to_yaml_format() {
        let colors = create_test_colors();
        let scheme = Base16Scheme::new(
            "test-theme".to_string(),
            "Jane Doe".to_string(),
            "quicktheme -f image.png -a \"Jane Doe\" -r 123 -m 5 -M 100".to_string(),
            &colors,
        )
        .unwrap();

        let yaml_lines = scheme.to_yaml();

        // Check that we have the right number of lines (3 metadata + 16 colors)
        assert_eq!(yaml_lines.len(), 19);

        // Check first three lines are scheme, author, and command
        assert!(yaml_lines[0].starts_with("scheme: "));
        assert!(yaml_lines[0].contains("test-theme"));
        
        assert!(yaml_lines[1].starts_with("author: "));
        assert!(yaml_lines[1].contains("Jane Doe"));
        
        assert!(yaml_lines[2].starts_with("command: "));
        assert!(yaml_lines[2].contains("quicktheme"));

        // Check base00 through base0F are present
        assert!(yaml_lines[3].starts_with("base00: "));
        assert!(yaml_lines[18].starts_with("base0F: "));
    }

    #[test]
    fn test_to_yaml_hex_format() {
        let colors = create_test_colors();
        let scheme = Base16Scheme::new(
            "hex-test".to_string(),
            "Test".to_string(),
            "test".to_string(),
            &colors,
        )
        .unwrap();

        let yaml_lines = scheme.to_yaml();

        // Check that base00 (black) is formatted correctly
        assert!(yaml_lines[3].contains("\"000000\""));
        
        // Check that base01 (white - maps to colors[5] which is yellow)
        assert!(yaml_lines[4].contains("\"ffff00\""));
    }

    #[test]
    fn test_command_field_preserved() {
        let colors = create_test_colors();
        let test_command = "quicktheme -f test.png -a \"Author\" -r 999 -m 1 -M 200 --output-mode swap";
        let scheme = Base16Scheme::new(
            "cmd-test".to_string(),
            "Author".to_string(),
            test_command.to_string(),
            &colors,
        )
        .unwrap();

        assert_eq!(scheme.command, test_command);
        
        let yaml_lines = scheme.to_yaml();
        assert!(yaml_lines[2].contains(test_command));
    }
}
