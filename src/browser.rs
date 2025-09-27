//! Browser implementation for rendering web content.

use crate::url::Url;
use anyhow::Result;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};

/// A web browser that loads and displays content.
#[derive(Debug)]
pub struct Browser {
    url: Url,
    content: String,
}

impl Browser {
    /// Loads content from a URL and creates a Browser instance.
    /// Acts as a constructor that fetches the content immediately.
    pub fn load(url_str: &str) -> Result<Self> {
        let url = Url::new(url_str)?;
        let response_body = url.request()?;
        Ok(Self {
            url,
            content: response_body.into_string(),
        })
    }

    /// Returns a reference to the loaded URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns a reference to the loaded content.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Draws the browser content to the SDL2 canvas.
    /// Renders text content character-by-character with scrolling support.
    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        ttf_context: &Sdl2TtfContext,
        texture_creator: &TextureCreator<WindowContext>,
        scroll_offset: i32,
        window_width: u32,
        window_height: u32,
    ) -> Result<(), String> {
        // Load font (using system font on macOS)
        let font_path = if cfg!(target_os = "macos") {
            "/System/Library/Fonts/Helvetica.ttc"
        } else if cfg!(target_os = "linux") {
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
        } else {
            // Windows
            "C:\\Windows\\Fonts\\arial.ttf"
        };

        let font = ttf_context.load_font(font_path, 14)?;

        // Clear canvas with white background
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        // Calculate texture dimensions based on content
        let lines: Vec<&str> = self.content.lines().collect();
        let line_height = 20;
        let margin = 10;
        let scrollbar_space = 20; // Reserve space for scrollbar

        // Fixed texture width to prevent stretching
        let texture_width = 2000u32; // Large enough for most content
        let total_height = (lines.len() as u32 * line_height as u32) + (margin as u32 * 2);

        // Create a render target texture for the entire content
        let mut content_texture = texture_creator
            .create_texture_target(None, texture_width, total_height)
            .map_err(|e| e.to_string())?;

        // Render all characters to the texture
        canvas
            .with_texture_canvas(&mut content_texture, |texture_canvas| {
                // Clear the texture with white background
                texture_canvas.set_draw_color(Color::RGB(255, 255, 255));
                texture_canvas.clear();

                // Render each line character by character
                for (line_idx, line) in lines.iter().enumerate() {
                    let mut x_position = margin;
                    let y_position = margin + (line_idx as i32 * line_height);

                    // Render each character in the line
                    for ch in line.chars() {
                        if ch == ' ' {
                            // Handle space character - just advance position
                            x_position += 6; // Approximate space width
                            continue;
                        }

                        // Render the character
                        if let Ok(surface) = font
                            .render_char(ch)
                            .blended(Color::RGB(0, 0, 0))
                            .map_err(|e| e.to_string())
                        {
                            // Convert surface to texture
                            if let Ok(char_texture) = texture_creator
                                .create_texture_from_surface(&surface)
                                .map_err(|e| e.to_string())
                            {
                                // Get character dimensions
                                let sdl2::render::TextureQuery { width, height, .. } =
                                    char_texture.query();

                                // Calculate character position
                                let target = Rect::new(x_position, y_position, width, height);

                                // Draw character to the content texture
                                let _ = texture_canvas.copy(&char_texture, None, Some(target));

                                // Advance x position for next character
                                x_position += width as i32;
                            }
                        }
                    }
                }
            })
            .map_err(|e| format!("Failed to render to texture: {:?}", e))?;

        // Copy the visible portion of the content texture to the canvas
        let visible_height = window_height;
        let visible_width = window_width - scrollbar_space;

        // Calculate how much content remains below the current scroll position
        let remaining_height = (total_height as i32 - scroll_offset).max(0) as u32;
        let copy_height = remaining_height.min(visible_height);

        // Use the smaller of window width or texture width to avoid stretching
        let copy_width = visible_width.min(texture_width);

        let source_rect = Rect::new(
            0,
            scroll_offset.max(0),
            copy_width,
            copy_height,
        );
        let dest_rect = Rect::new(0, 0, copy_width, copy_height);

        canvas.copy(&content_texture, Some(source_rect), Some(dest_rect))?;

        // Draw scrollbar if content is scrollable
        if total_height > visible_height {
            // Scrollbar dimensions
            let scrollbar_width = 12;
            let scrollbar_x = (window_width - scrollbar_width - 5) as i32; // 5px margin from right edge
            let scrollbar_track_height = visible_height as i32 - 20; // 10px margin top and bottom
            let scrollbar_track_y = 10;

            // Draw scrollbar track (background)
            canvas.set_draw_color(Color::RGB(220, 220, 220));
            canvas.fill_rect(Rect::new(
                scrollbar_x,
                scrollbar_track_y,
                scrollbar_width,
                scrollbar_track_height as u32,
            ))?;

            // Calculate thumb size and position
            let view_ratio = visible_height as f32 / total_height as f32;
            let thumb_height = (scrollbar_track_height as f32 * view_ratio).max(20.0) as u32;

            let max_scroll = (total_height as i32 - visible_height as i32).max(0);
            let scroll_ratio = if max_scroll > 0 {
                scroll_offset as f32 / max_scroll as f32
            } else {
                0.0
            };

            let thumb_travel = scrollbar_track_height as u32 - thumb_height;
            let thumb_y = scrollbar_track_y + (thumb_travel as f32 * scroll_ratio) as i32;

            // Draw scrollbar thumb
            canvas.set_draw_color(Color::RGB(128, 128, 128));
            canvas.fill_rect(Rect::new(
                scrollbar_x,
                thumb_y,
                scrollbar_width,
                thumb_height,
            ))?;

            // Draw thumb border for better visibility
            canvas.set_draw_color(Color::RGB(100, 100, 100));
            canvas.draw_rect(Rect::new(
                scrollbar_x,
                thumb_y,
                scrollbar_width,
                thumb_height,
            ))?;
        }

        canvas.present();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheme::Scheme;

    #[test]
    fn test_browser_load_creates_instance_with_url_and_content() {
        // This test will need a way to mock the network request
        // For now, we'll test the structure exists and compiles
        // Real network tests would use FakeConnection
    }

    #[test]
    fn test_browser_url_getter_returns_url() {
        // We'll need to create a test helper that constructs a Browser
        // with mocked data to test the getters
    }

    #[test]
    fn test_browser_content_getter_returns_content() {
        // Similar to above, test the content getter
    }

    fn create_test_browser_with_mock_response() -> Browser {
        // Helper to create a Browser with mock data
        // This would use FakeConnection to simulate network response
        let url = Url::new("http://example.com").unwrap();
        let content = "Test content".to_string();
        Browser { url, content }
    }

    #[test]
    fn test_browser_struct_has_correct_fields() {
        let browser = create_test_browser_with_mock_response();
        assert_eq!(browser.url().scheme, Scheme::Http);
        assert_eq!(browser.url().host.as_str(), "example.com");
        assert_eq!(browser.content(), "Test content");
    }

    #[test]
    fn test_browser_load_with_invalid_url_returns_error() {
        let result = Browser::load("not_a_valid_url");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing '://' separator"));
    }

    // Integration test using FakeConnection
    #[test]
    fn test_browser_load_with_fake_connection() {
        // This would require modifying Browser::load to accept a connection
        // or using a different approach for dependency injection
        // For now, we'll leave this as a placeholder
    }
}