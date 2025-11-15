//! # Font Configuration
//!
//! Custom font loading and configuration for the trading terminal.
//! Supports multiple fonts for different contexts (monospace for code/data, proportional for UI).
//!
//! This enables VSCode-like customization with different fonts for different purposes.

use egui::{FontDefinitions, FontData, FontFamily, Context, TextStyle, FontId, Theme as EguiTheme};
use egui::epaint::text::{FontInsert, InsertFontFamily, FontPriority};

/// Font configuration for the terminal application
pub struct FontConfig {
    /// Whether custom fonts are enabled
    pub custom_fonts_enabled: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        FontConfig {
            custom_fonts_enabled: false,
        }
    }
}

impl FontConfig {
    /// Configure custom fonts for the application
    /// 
    /// This allows you to:
    /// - Use monospace fonts for code/data (like VSCode's editor)
    /// - Use proportional fonts for UI elements (like VSCode's UI)
    /// - Mix different fonts for different text styles
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// // Load fonts from files
    /// FontConfig::setup_custom_fonts(ctx, &[
    ///     ("FiraCode", include_bytes!("../../assets/fonts/FiraCode-Regular.ttf")),
    ///     ("Inter", include_bytes!("../../assets/fonts/Inter-Regular.ttf")),
    /// ]);
    /// ```
    pub fn setup_custom_fonts(ctx: &Context, fonts: &[(&str, &'static [u8])]) {
        let mut font_definitions = FontDefinitions::default();
        
        // Add each custom font
        for (name, font_data) in fonts {
            font_definitions.font_data.insert(
                name.to_string(),
                std::sync::Arc::new(FontData::from_static(font_data)),
            );
        }
        
        // Configure font families
        // Monospace: Perfect for code, terminal output, data tables (like VSCode editor)
        if let Some(font) = fonts.iter().find(|(name, _)| name.contains("Code") || name.contains("Mono")) {
            font_definitions
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, font.0.to_string());
        } else {
            // Fallback: Use system monospace
            font_definitions
                .families
                .get_mut(&FontFamily::Monospace)
                .unwrap()
                .insert(0, "Monospace".to_string());
        }
        
        // Proportional: For UI text, buttons, labels (like VSCode UI)
        if let Some(font) = fonts.iter().find(|(name, _)| !name.contains("Code") && !name.contains("Mono")) {
            font_definitions
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, font.0.to_string());
        }
        
        // Apply font definitions
        ctx.set_fonts(font_definitions);
        
        // Configure text styles (similar to VSCode's editor settings)
        // Use style_mut_of API which is safe for egui 0.33
        ctx.style_mut_of(EguiTheme::Dark, |style| {
            // Monospace styles for code/data
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            );
            
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(14.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            );
        });
        
        // Also configure for light theme
        ctx.style_mut_of(EguiTheme::Light, |style| {
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Monospace),
            );
            
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(14.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            );
            
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(20.0, FontFamily::Proportional),
            );
        });
    }
    
    /// Setup fonts with JetBrains Mono as the primary font (replaces system fonts)
    /// 
    /// JetBrains Mono font will be used for all UI elements as the default monospace font
    pub fn setup_terminal_fonts(ctx: &Context) {
        // Start with default font definitions
        let mut font_definitions = FontDefinitions::default();
        
        // Load JetBrains Mono font and make it the primary font (replaces all system fonts)
        // JetBrains Mono is a sharp, technical monospace font perfect for terminals
        let jetbrains_font_bytes = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf");
        font_definitions.font_data.insert(
            "JetBrains Mono".to_string(),
            std::sync::Arc::new(FontData::from_static(jetbrains_font_bytes)),
        );
        
        // Completely replace Monospace font family with only JetBrains Mono
        // Clear all existing fonts and set JetBrains Mono as the only monospace font
        font_definitions
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .clear();
        font_definitions
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("JetBrains Mono".to_string());
        
        // Also use JetBrains Mono for Proportional (terminal text)
        font_definitions
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .clear();
        font_definitions
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .push("JetBrains Mono".to_string());
        
        tracing::info!("[FONT] JetBrains Mono font loaded and set as primary font, replacing all system fonts");
        
        // Apply font definitions
        ctx.set_fonts(font_definitions);
        
        // Configure text styles for terminal using JetBrains Mono font
        // Use style_mut_of API which is safe for egui 0.33
        ctx.style_mut_of(EguiTheme::Dark, |style| {
            // Terminal-optimized font sizes, all using JetBrains Mono
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            );
            
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(13.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(13.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(18.0, FontFamily::Proportional), // JetBrains Mono
            );
        });
        
        // Also configure for light theme
        ctx.style_mut_of(EguiTheme::Light, |style| {
            style.text_styles.insert(
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            );
            
            style.text_styles.insert(
                TextStyle::Small,
                FontId::new(11.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Body,
                FontId::new(13.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Button,
                FontId::new(13.0, FontFamily::Proportional), // JetBrains Mono
            );
            
            style.text_styles.insert(
                TextStyle::Heading,
                FontId::new(18.0, FontFamily::Proportional), // JetBrains Mono
            );
        });
        
        tracing::info!("[FONT] JetBrains Mono font set as primary font, replacing system fonts");
    }
    
    /// Setup fonts from TTF files in the assets directory
    /// 
    /// # Font Recommendations for VSCode-like Experience:
    /// 
    /// **Monospace (Editor/Code):**
    /// - Fira Code (with ligatures)
    /// - JetBrains Mono
    /// - Source Code Pro
    /// - Cascadia Code
    /// - Consolas (Windows)
    /// - Menlo (macOS)
    /// 
    /// **Proportional (UI):**
    /// - Inter
    /// - Roboto
    /// - Segoe UI (Windows)
    /// - San Francisco (macOS)
    /// - Open Sans
    pub fn setup_fonts_from_files(ctx: &Context) {
        // Example: Load fonts from embedded resources
        // In a real application, you would:
        // 1. Place font files in a assets/fonts/ directory
        // 2. Use include_bytes! to embed them
        // 3. Or load them at runtime from the file system
        
        // Example structure:
        // let fonts = vec![
        //     ("FiraCode", include_bytes!("../../assets/fonts/FiraCode-Regular.ttf")),
        //     ("FiraCodeBold", include_bytes!("../../assets/fonts/FiraCode-Bold.ttf")),
        //     ("Inter", include_bytes!("../../assets/fonts/Inter-Regular.ttf")),
        //     ("InterBold", include_bytes!("../../assets/fonts/Inter-Bold.ttf")),
        // ];
        // 
        // Self::setup_custom_fonts(ctx, &fonts);
        
        // For now, use terminal-optimized defaults
        Self::setup_terminal_fonts(ctx);
    }
    
    /// Load custom fonts from embedded assets
    /// JetBrains Mono font is already loaded in setup_terminal_fonts() as the primary font
    /// This function verifies JetBrains Mono is properly loaded
    pub fn load_custom_fonts(ctx: &Context) {
        tracing::info!("[FONT] Verifying JetBrains Mono font is loaded");
        
        // JetBrains Mono font is already loaded in setup_terminal_fonts() as the primary font
        // No need to load it again here
        tracing::info!("[FONT] JetBrains Mono font already loaded as primary font in setup_terminal_fonts()");
        
        // Verify JetBrains Mono font was registered
        let fonts_verified = ctx.fonts(|fonts| {
            let defs = fonts.definitions();
            let has_jetbrains = defs.font_data.contains_key("JetBrains Mono");
            let jetbrains_in_proportional = defs.families
                .get(&FontFamily::Proportional)
                .map(|f| f.contains(&"JetBrains Mono".to_string()))
                .unwrap_or(false);
            
            tracing::info!("[FONT] Verification - JetBrains Mono: {}, JetBrains Mono in Proportional: {}", 
                has_jetbrains, jetbrains_in_proportional);
            
            has_jetbrains && jetbrains_in_proportional
        });
        
        if fonts_verified {
            tracing::info!("[FONT] JetBrains Mono font verified successfully!");
        } else {
            tracing::warn!("[FONT] Font verification failed - JetBrains Mono may not be properly registered");
        }
    }
    
    /// Load Momo Trust Display font from embedded assets (deprecated - Momo font no longer used)
    /// This function is kept for compatibility but no longer loads Momo font
    #[deprecated(note = "Momo Trust Display font is no longer used. All fonts now use JetBrains Mono.")]
    pub fn load_momo_trust_display(_ctx: &Context) {
        // Momo font has been removed, JetBrains Mono is now used for all text
        tracing::info!("[FONT] Momo font loading skipped - JetBrains Mono is used instead");
    }
    
    /// Apply Momo Trust Display font bytes to egui context (deprecated - Momo font no longer used)
    /// This should be called from the main thread with downloaded font bytes
    #[deprecated(note = "Momo Trust Display font is no longer used. All fonts now use JetBrains Mono.")]
    pub fn apply_momo_trust_display_font(ctx: &Context, font_bytes: Vec<u8>) {
        let font_name = "Momo Trust Display";
        
        tracing::info!("[FONT] Applying font '{}' using add_font API", font_name);
        
        ctx.add_font(FontInsert::new(
            font_name,
            FontData::from_owned(font_bytes),
            vec![
                // Add to custom FontFamily::Name for explicit use
                InsertFontFamily {
                    family: FontFamily::Name("Momo Trust Display".into()),
                    priority: FontPriority::Highest,
                },
                // Also add to Proportional as highest priority for fallback
                InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Highest,
                },
            ],
        ));
        
        tracing::info!("[FONT] Font '{}' applied successfully", font_name);
        
        // Verify font was registered
        let font_verified = ctx.fonts(|fonts| {
            let defs = fonts.definitions();
            let has_font_data = defs.font_data.contains_key(font_name);
            let has_custom_family = defs.families.contains_key(&FontFamily::Name("Momo Trust Display".into()));
            let in_proportional = defs.families
                .get(&FontFamily::Proportional)
                .map(|f| f.contains(&font_name.to_string()))
                .unwrap_or(false);
            
            tracing::info!("[FONT] Verification - font_data: {}, custom_family: {}, in_proportional: {}", 
                has_font_data, has_custom_family, in_proportional);
            
            has_font_data && (has_custom_family || in_proportional)
        });
        
        if font_verified {
            tracing::info!("[FONT] Font verification successful!");
        } else {
            tracing::warn!("[FONT] Font verification failed - font may not be properly registered");
        }
    }
    
    /// Load Avenir 85 Heavy font from embedded assets (deprecated - now uses JetBrains Mono)
    /// This function is kept for compatibility but no longer loads Avenir
    #[deprecated(note = "Avenir font no longer used. All fonts now use JetBrains Mono.")]
    pub fn load_avenir_85_heavy(_ctx: &Context) {
        // Avenir font has been removed, JetBrains Mono is now used for all text
        tracing::info!("[FONT] Avenir font loading skipped - JetBrains Mono is used instead");
    }
    
    /// Get terminal font (JetBrains Mono) - previously Avenir, now uses JetBrains Mono
    /// This returns JetBrains Mono which is used for all terminal text
    pub fn get_avenir_font(_ctx: &Context, size: f32) -> FontId {
        // Use Proportional which now has JetBrains Mono loaded
        FontId::new(size, FontFamily::Proportional)
    }
    
    /// Get available font families
    pub fn get_available_fonts(ctx: &Context) -> Vec<String> {
        // In egui 0.27.2, we access font data through the style
        // Since we can't directly access font definitions, we return the font families from style
        let style = ctx.style();
        let mut fonts = Vec::new();
        
        // Collect font names from text styles
        for (_, font_id) in &style.text_styles {
            if !fonts.contains(&font_id.family.to_string()) {
                fonts.push(font_id.family.to_string());
            }
        }
        
        fonts
    }
}

/// Helper to create a font configuration UI (for settings)
pub fn font_settings_ui(ui: &mut egui::Ui, ctx: &Context) {
    ui.heading("Font Settings");
    
    ui.separator();
    
    ui.label("Available Fonts:");
    let available_fonts = FontConfig::get_available_fonts(ctx);
    for font in available_fonts {
        ui.label(format!("  • {}", font));
    }
    
    ui.separator();
    
    ui.label("Font Sizes:");
    
    // Font size controls - collect values first, then apply using style_mut_of
    // Get current theme to modify the correct style
    let current_theme = ctx.style().visuals.dark_mode;
    let theme = if current_theme { EguiTheme::Dark } else { EguiTheme::Light };
    
    // Get current font sizes
    let mut monospace_size = ctx.style().text_styles.get(&TextStyle::Monospace)
        .map(|f| f.size)
        .unwrap_or(13.0);
    let mut body_size = ctx.style().text_styles.get(&TextStyle::Body)
        .map(|f| f.size)
        .unwrap_or(13.0);
    let mut heading_size = ctx.style().text_styles.get(&TextStyle::Heading)
        .map(|f| f.size)
        .unwrap_or(18.0);
    
    // Font size controls
    ui.horizontal(|ui| {
        ui.label("Monospace:");
        ui.add(egui::Slider::new(&mut monospace_size, 8.0..=24.0));
    });
    
    ui.horizontal(|ui| {
        ui.label("Body:");
        ui.add(egui::Slider::new(&mut body_size, 8.0..=24.0));
    });
    
    ui.horizontal(|ui| {
        ui.label("Heading:");
        ui.add(egui::Slider::new(&mut heading_size, 12.0..=32.0));
    });
    
    // Apply changes using style_mut_of API
    ctx.style_mut_of(theme, |style| {
        if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Monospace) {
            font_id.size = monospace_size;
        }
        if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Body) {
            font_id.size = body_size;
        }
        if let Some(font_id) = style.text_styles.get_mut(&TextStyle::Heading) {
            font_id.size = heading_size;
        }
    });
}



