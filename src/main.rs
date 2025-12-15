use image::codecs::avif::AvifEncoder;
use image::codecs::png::PngEncoder;
use image::{EncodableLayout, ExtendedColorType, ImageEncoder, Rgba};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode, Version};
use serde::Deserialize;
use serde_hex::{SerHex, SerHexOpt, StrictCap};
use serde_querystring::ParseMode;
use std::env;
use std::io::{Write, stdout};

#[derive(Copy, Clone, Deserialize, Debug, Default)]
enum Format {
    #[default]
    Png,
    Avif,
    Svg,
}

#[derive(Copy, Clone, Deserialize, Debug, Default)]
enum Ec {
    L,
    #[default]
    M,
    Q,
    H,
}

impl Into<EcLevel> for Ec {
    fn into(self) -> EcLevel {
        match self {
            Self::L => EcLevel::L,
            Self::M => EcLevel::M,
            Self::Q => EcLevel::Q,
            Self::H => EcLevel::H,
        }
    }
}

#[derive(Copy, Clone, Deserialize, Debug, Default)]
enum Mode {
    #[default]
    Auto,
    Standard,
    Micro,
}

#[derive(Deserialize, Debug)]
struct Options {
    #[serde(default)]
    width: Option<u32>,
    #[serde(default, with = "SerHexOpt::<StrictCap>")]
    fg: Option<[u8; 4]>,
    #[serde(default, with = "SerHexOpt::<StrictCap>")]
    bg: Option<[u8; 4]>,
    #[serde(default)]
    ec: Ec,
    #[serde(default)]
    version: Option<i16>,
    #[serde(default)]
    format: Format,
    #[serde(default)]
    mode: Mode,
}

fn generate_qr(data: &str) -> Result<(), String> {
    let query = env::var("QUERY_STRING")
        .expect("QUERY_STRING environment variable is expected; is this being called by CGI?");
    let options: Options = serde_querystring::from_str(&query, ParseMode::UrlEncoded)
        .map_err(|error| error.to_string())?;
    let qrcode = match (
        options.mode,
        options.version.unwrap_or(9),
        options.ec.into(),
    ) {
        (Mode::Auto, _, ec) => QrCode::with_error_correction_level(data.as_bytes(), ec)
            .map_err(|error| error.to_string())?,
        (Mode::Standard, ver, ec) => {
            QrCode::with_version(data.as_bytes(), Version::Normal(ver), ec)
                .map_err(|error| error.to_string())?
        }
        (Mode::Micro, ver, ec) => QrCode::with_version(data.as_bytes(), Version::Micro(ver), ec)
            .map_err(|error| error.to_string())?,
    };

    let mut stdout = stdout().lock();
    match options.format {
        Format::Png => {
            let mut renderer = qrcode.render::<Rgba<u8>>();
            let width = u32::min(options.width.unwrap_or(256), 1024);
            renderer.min_dimensions(width, width);
            if let Some(fg) = options.fg {
                renderer.dark_color(Rgba::<u8>(fg));
            }
            if let Some(bg) = options.bg {
                renderer.light_color(Rgba::<u8>(bg));
            }
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
        }
        Format::Avif => {
            let mut renderer = qrcode.render::<Rgba<u8>>();
            let width = u32::min(options.width.unwrap_or(256), 1024);
            renderer.min_dimensions(width, width);
            if let Some(fg) = options.fg {
                renderer.dark_color(Rgba::<u8>(fg));
            }
            if let Some(bg) = options.bg {
                renderer.light_color(Rgba::<u8>(bg));
            }
            writeln!(stdout, "Content-type: image/avif").map_err(|error| error.to_string())?;
            writeln!(stdout, "Content-disposition: attachment; filename=qr.avif")
                .map_err(|error| error.to_string())?;
            writeln!(stdout).map_err(|error| error.to_string())?;
            let encoder = AvifEncoder::new(stdout);
            let image = renderer.build();
            encoder
                .write_image(
                    image.as_bytes(),
                    image.width(),
                    image.height(),
                    ExtendedColorType::Rgba8,
                )
                .map_err(|error| error.to_string())?;
        }
        Format::Svg => {
            let mut renderer = qrcode.render::<svg::Color>();
            let width = u32::min(options.width.unwrap_or(256), 1024);
            renderer.min_dimensions(width, width);
            let fg = options.fg.map(|fg| {
                let mut s = Vec::with_capacity(8);
                SerHex::<StrictCap>::into_hex_raw(&fg, &mut s).unwrap();
                format!("#{}", String::from_utf8(s).unwrap())
            });
            if let Some(fg) = &fg {
                renderer.dark_color(svg::Color(fg));
            }
            let bg = options.bg.map(|bg| {
                let mut s = Vec::with_capacity(8);
                SerHex::<StrictCap>::into_hex_raw(&bg, &mut s).unwrap();
                format!("#{}", String::from_utf8(s).unwrap())
            });
            if let Some(bg) = &bg {
                renderer.light_color(svg::Color(bg));
            }
            writeln!(stdout, "Content-type: image/svg+xml").map_err(|error| error.to_string())?;
            writeln!(stdout, "Content-disposition: attachment; filename=qr.svg")
                .map_err(|error| error.to_string())?;
            writeln!(stdout).map_err(|error| error.to_string())?;
            write!(stdout, "{}", renderer.build()).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

fn main() {
    let path = env::var("PATH_INFO")
        .expect("PATH_INFO environment variable is expected; is this being called by CGI?");
    if let Err(error) = generate_qr(&path[1..]) {
        eprintln!("{error}");
        println!("Content-type: text/plain\n");
        println!("{error}");
    }
}
