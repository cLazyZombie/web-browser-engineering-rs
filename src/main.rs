//! Web Browser Engineering in Rust with SDL2 GUI

use anyhow::Result;
use browser::browser::Browser;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use std::env;

fn main() -> Result<()> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if URL argument is provided
    if args.len() != 2 {
        eprintln!("Usage: {} <URL>", args[0]);
        eprintln!("Example: {} http://example.org/", args[0]);
        std::process::exit(1);
    }

    // Create Browser instance with the URL
    let browser = Browser::load(&args[1])?;

    // Initialize SDL2
    let sdl_context = sdl2::init().map_err(|e| anyhow::anyhow!(e))?;
    let video_subsystem = sdl_context.video().map_err(|e| anyhow::anyhow!(e))?;
    let ttf_context = sdl2::ttf::init().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Create window
    let window = video_subsystem
        .window("Web Browser - SDL2", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    // Create canvas
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let texture_creator = canvas.texture_creator();

    // Track window dimensions
    let mut window_width = 800u32;
    let mut window_height = 600u32;

    // Calculate maximum scroll based on content
    let lines = browser.content().lines().count();
    let line_height = 20;
    let margin = 10;
    let total_content_height = (lines as i32 * line_height) + (margin * 2);
    let mut max_scroll_offset = (total_content_height - window_height as i32).max(0);

    // Scroll offset for handling long content
    let mut scroll_offset = 0i32;
    let scroll_speed = 20;

    // Initial render
    browser
        .draw(
            &mut canvas,
            &ttf_context,
            &texture_creator,
            scroll_offset,
            window_width,
            window_height,
        )
        .map_err(|e| anyhow::anyhow!(e))?;

    // Event loop
    let mut event_pump = sdl_context.event_pump().map_err(|e| anyhow::anyhow!(e))?;
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    window_width = width as u32;
                    window_height = height as u32;
                    // Recalculate max scroll offset
                    max_scroll_offset = (total_content_height - window_height as i32).max(0);
                    // Adjust current scroll if needed
                    scroll_offset = scroll_offset.min(max_scroll_offset);
                    browser
                        .draw(
                            &mut canvas,
                            &ttf_context,
                            &texture_creator,
                            scroll_offset,
                            window_width,
                            window_height,
                        )
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    scroll_offset = (scroll_offset + scroll_speed).min(max_scroll_offset);
                    browser
                        .draw(
                            &mut canvas,
                            &ttf_context,
                            &texture_creator,
                            scroll_offset,
                            window_width,
                            window_height,
                        )
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    scroll_offset = (scroll_offset - scroll_speed).max(0);
                    browser
                        .draw(
                            &mut canvas,
                            &ttf_context,
                            &texture_creator,
                            scroll_offset,
                            window_width,
                            window_height,
                        )
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
                Event::MouseWheel { y, .. } => {
                    scroll_offset = (scroll_offset - (y * scroll_speed))
                        .max(0)
                        .min(max_scroll_offset);
                    browser
                        .draw(
                            &mut canvas,
                            &ttf_context,
                            &texture_creator,
                            scroll_offset,
                            window_width,
                            window_height,
                        )
                        .map_err(|e| anyhow::anyhow!(e))?;
                }
                _ => {}
            }
        }

        // Small delay to prevent high CPU usage
        ::std::thread::sleep(std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}