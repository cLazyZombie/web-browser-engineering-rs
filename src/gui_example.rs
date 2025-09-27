use fltk::{enums, prelude::*, *};

fn main() {
    let a = app::App::default();
    let mut win = window::Window::default().with_size(800, 600);
    win.end();
    win.show();

    win.draw(|w| {
        use draw::*;
        // Fill window with white background
        draw_rect_fill(0, 0, w.w(), w.h(), enums::Color::White);
        // Set black color for text
        set_draw_color(enums::Color::Black);
        set_font(enums::Font::Courier, 16);
        // Draw "Hello, World" at the center
        draw_text("Hello, World", w.w() / 2 - 50, w.h() / 2);
    });

    a.run().unwrap();
}