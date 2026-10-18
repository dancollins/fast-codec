use std::{env, fs, path::Path, path::PathBuf};

use resvg::{tiny_skia, usvg};

const ICON_SVG: &str = "assets/icon.svg";

// Windows picks the closest frame, so include the 125% and 150% sizes as well.
const ICO_SIZES: [u32; 10] = [16, 20, 24, 30, 32, 40, 48, 60, 64, 256];

const WINDOW_ICON_PX: u32 = 64;

fn main() {
    slint_build::compile("ui/main.slint").unwrap();

    println!("cargo:rerun-if-changed={ICON_SVG}");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let tree =
        usvg::Tree::from_data(&fs::read(ICON_SVG).unwrap(), &usvg::Options::default()).unwrap();

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let ico = out_dir.join("fast-codec.ico");
        write_ico(&tree, &ico);
        // set_icon embeds it as icon resource 1, which main.rs loads for the window.
        winresource::WindowsResource::new()
            .set_icon(ico.to_str().unwrap())
            .compile()
            .unwrap();
    } else {
        fs::write(
            out_dir.join("window-icon.rgba"),
            render_rgba(&tree, WINDOW_ICON_PX),
        )
        .unwrap();
        println!("cargo:rustc-env=WINDOW_ICON_PX={WINDOW_ICON_PX}");
    }
}

fn write_ico(tree: &usvg::Tree, out: &Path) {
    let frames: Vec<Vec<u8>> = ICO_SIZES
        .iter()
        .map(|&px| render(tree, px).encode_png().unwrap())
        .collect();

    // ICO with PNG-encoded frames: a 6-byte header, a 16-byte entry per frame, then the PNGs.
    let mut ico = Vec::new();
    ico.extend_from_slice(&[0, 0, 1, 0]);
    ico.extend_from_slice(&(frames.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * frames.len() as u32;
    for (&px, png) in ICO_SIZES.iter().zip(&frames) {
        let side = if px >= 256 { 0 } else { px as u8 };
        ico.extend_from_slice(&[side, side, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += png.len() as u32;
    }
    for png in &frames {
        ico.extend_from_slice(png);
    }
    fs::write(out, ico).unwrap();
}

/// Straight (not premultiplied) RGBA.
fn render_rgba(tree: &usvg::Tree, px: u32) -> Vec<u8> {
    render(tree, px)
        .pixels()
        .iter()
        .flat_map(|p| {
            let c = p.demultiply();
            [c.red(), c.green(), c.blue(), c.alpha()]
        })
        .collect()
}

fn render(tree: &usvg::Tree, px: u32) -> tiny_skia::Pixmap {
    let mut pixmap = tiny_skia::Pixmap::new(px, px).unwrap();
    let scale = px as f32 / tree.size().width();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap
}
