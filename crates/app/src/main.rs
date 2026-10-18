#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod convert;

use fast_codec_codec::{Base64, Transform};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;
    window.set_transform(Base64.name().into());
    window.set_window_icon(window_icon());

    // `edited` fires on user edits only, so these setters don't call back into each other.
    let weak = window.as_weak();
    window.on_decoded_edited(move |decoded| {
        let window = weak.unwrap();
        let out = convert::encode(&Base64, &decoded);
        window.set_encoded(out.text.into());
        window.set_status(out.status.into());
    });

    let weak = window.as_weak();
    window.on_encoded_edited(move |encoded| {
        let window = weak.unwrap();
        let out = convert::decode(&Base64, &encoded);
        window.set_decoded(out.text.into());
        window.set_status(out.status.into());
    });

    window.run()
}

// Pre-rendered rather than the SVG: Slint 1.18 multiplies an SVG icon's colours by alpha
// twice on the way to winit, darkening the translucent bits (`icon_to_winit` in
// i-slint-backend-winit). Straight RGBA reaches winit unchanged.
fn window_icon() -> slint::Image {
    let px = env!("WINDOW_ICON_PX").parse().unwrap();
    let rgba = include_bytes!(concat!(env!("OUT_DIR"), "/window-icon.rgba"));
    slint::Image::from_rgba8(slint::SharedPixelBuffer::clone_from_slice(rgba, px, px))
}
