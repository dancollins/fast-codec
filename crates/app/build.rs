use std::{env, fs, path::Path, path::PathBuf};

use resvg::{tiny_skia, usvg};

const ICON_SVG: &str = "assets/icon.svg";

// Windows picks the closest frame, so include the 125% and 150% sizes as well.
const ICO_SIZES: [u32; 10] = [16, 20, 24, 30, 32, 40, 48, 60, 64, 256];

fn main() {
    slint_build::compile("ui/main.slint").unwrap();

    println!("cargo:rerun-if-changed={ICON_SVG}");
    let ico = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("fast-codec.ico");
    write_ico(Path::new(ICON_SVG), &ico);

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon(ico.to_str().unwrap())
            .compile()
            .unwrap();
    }
}

fn write_ico(svg: &Path, out: &Path) {
    let tree = usvg::Tree::from_data(&fs::read(svg).unwrap(), &usvg::Options::default()).unwrap();
    let frames: Vec<Vec<u8>> = ICO_SIZES.iter().map(|&px| render_png(&tree, px)).collect();

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

fn render_png(tree: &usvg::Tree, px: u32) -> Vec<u8> {
    let mut pixmap = tiny_skia::Pixmap::new(px, px).unwrap();
    let scale = px as f32 / tree.size().width();
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().unwrap()
}
