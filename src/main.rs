use image::codecs::png::PngEncoder;
use image::{EncodableLayout, ExtendedColorType, ImageEncoder, Rgba};
use qrcode::QrCode;
use serde::Deserialize;
use serde_hex::{SerHexOpt, StrictCap};
use serde_querystring::ParseMode;
use std::env;
use std::io::{Write, stdout};

#[derive(Deserialize, Debug)]
struct Options {
    #[serde(default)]
    width: Option<u32>,
    #[serde(default, with = "SerHexOpt::<StrictCap>")]
    fg: Option<[u8; 4]>,
    #[serde(default, with = "SerHexOpt::<StrictCap>")]
    bg: Option<[u8; 4]>,
}

fn handle() -> Result<(), String> {
    let path = env::var("PATH_INFO")
        .expect("PATH_INFO environment variable is expected; is this being called by CGI?");
    let query = env::var("QUERY_STRING")
        .expect("QUERY_STRING environment variable is expected; is this being called by CGI?");
    let options: Options = serde_querystring::from_str(&query, ParseMode::UrlEncoded)
        .map_err(|error| error.to_string())?;
    let qrcode = QrCode::new(path.as_bytes()).map_err(|error| error.to_string())?;
    let mut renderer = qrcode.render::<Rgba<u8>>();
    let width = u32::min(options.width.unwrap_or(256), 1024);
    renderer.min_dimensions(width, width);
    if let Some(fg) = options.fg {
        renderer.dark_color(Rgba::<u8>(fg));
    }
    if let Some(bg) = options.bg {
        renderer.light_color(Rgba::<u8>(bg));
    }
    let mut stdout = stdout().lock();
    writeln!(stdout, "Content-type: image/png").map_err(|error| error.to_string())?;
    writeln!(stdout, "Content-disposition: attachment; filename=qr.png")
        .map_err(|error| error.to_string())?;
    writeln!(stdout).map_err(|error| error.to_string())?;
    let encoder = PngEncoder::new(stdout);
    let image = renderer.build();
    encoder
        .write_image(
            image.as_bytes(),
            image.width(),
            image.height(),
            ExtendedColorType::Rgba8,
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn main() {
    if let Err(error) = handle() {
        println!("Content-type: text/plain");
        println!("{error}");
    }
}
