#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod convert;

use fast_codec_codec::Base64;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let window = MainWindow::new()?;

    // `edited` fires on user edits only, so these setters don't call back into each other.
    let weak = window.as_weak();
    window.on_plain_edited(move |plain| {
        let window = weak.unwrap();
        let out = convert::encode(&Base64, &plain);
        window.set_encoded(out.text.into());
        window.set_status(out.status.into());
    });

    let weak = window.as_weak();
    window.on_encoded_edited(move |encoded| {
        let window = weak.unwrap();
        let out = convert::decode(&Base64, &encoded);
        window.set_plain(out.text.into());
        window.set_status(out.status.into());
    });

    window.run()
}
