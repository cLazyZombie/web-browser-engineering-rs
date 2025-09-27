use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

fn main() -> Result<(), String> {
    // Initialize SDL2
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    // Create window
    let window = video_subsystem
        .window("SDL2 Example", 800, 600)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    // Create canvas
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Load font (using system font on macOS)
    let font_path = if cfg!(target_os = "macos") {
        "/System/Library/Fonts/Helvetica.ttc"
    } else if cfg!(target_os = "linux") {
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"
    } else {
        // Windows
        "C:\\Windows\\Fonts\\arial.ttf"
    };

    let font = ttf_context.load_font(font_path, 24)?;

    // Render text to surface
    let surface = font
        .render("Hello, World")
        .blended(Color::RGB(0, 0, 0)) // Black text
        .map_err(|e| e.to_string())?;

    // Convert surface to texture
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;

    // Query texture to get width and height
    let TextureQuery { width, height, .. } = texture.query();

    // Calculate position to center text
    let target = Rect::new(
        (800 - width as i32) / 2,
        (600 - height as i32) / 2,
        width,
        height,
    );

    // Event loop
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Clear canvas with white background
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        canvas.clear();

        // Draw text
        canvas.copy(&texture, None, Some(target))?;

        // Present canvas
        canvas.present();

        // Small delay to prevent high CPU usage
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}