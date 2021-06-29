use image::{imageops::FilterType, ColorType, GenericImageView};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::BufWriter;

#[derive(Debug, Deserialize)]
pub struct ThumbOptions {
    url: String,
    width: u32,
}

impl From<&str> for ThumbOptions {
    fn from(query_params: &str) -> Self {
        let qs = querify(query_params);
        ThumbOptions::new(qs)
    }
}

impl ThumbOptions {
    pub fn new(opts: HashMap<String, String>) -> ThumbOptions {
        let url: String = match opts.get("url") {
            Some(val) => String::from(val),
            None => String::from(""),
        };

        let width: u32 = match opts.get("width") {
            Some(val) => val.parse::<u32>().unwrap(),
            None => 180,
        };

        ThumbOptions {
            url: url,
            width: width,
        }
    }
}

fn querify(string: &str) -> HashMap<String, String> {
    let mut acc: HashMap<String, String> = HashMap::new();
    let pairs: Vec<&str> = string.split('&').collect();
    for kv in pairs {
        let mut it = kv.splitn(2, '=').take(2);
        match (it.next(), it.next()) {
            (Some("url"), Some(v)) => acc.insert(String::from("url"), v.to_string()),
            (Some("width"), Some(v)) => acc.insert(String::from("width"), v.to_string()),
            _ => continue,
        };
    }
    acc
}

pub async fn handle_thumbnail(
    opts: ThumbOptions,
    client: reqwest::Client,
) -> Result<Vec<u8>, Box<dyn std::error::Error + 'static>> {
    println!("123");
    let file = match client.get(&opts.url).send().await?.bytes().await {
        Ok(data) => data,
        Err(err) => return Err(err.into()),
    };

    println!("321");

    if file.len() == 0 {
        return Err(std::boxed::Box::from(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Image data malformed",
        )));
    }

    let width = opts.width;
    let format = image::guess_format(&file)?;
    let image = image::load_from_memory(&file)?;
    let original_width = image.width();
    let ratio = original_width / width;
    let original_height = image.height();
    let height = original_height / ratio;

    let resized = image::imageops::resize(&image, width, height, FilterType::Triangle).into_raw();
    let mut bytes: Vec<u8> = vec![];

    encode_image(&mut bytes, &resized, width, height, format);

    Ok(bytes)
}

pub fn encode_image(
    bytes: &mut Vec<u8>,
    file: &Vec<u8>,
    width: u32,
    height: u32,
    format: image::ImageFormat,
) -> () {
    let mut fout = BufWriter::new(bytes);

    match format {
        image::ImageFormat::Png => image::codecs::png::PngEncoder::new(fout)
            .encode(file, width, height, ColorType::Rgba8)
            .unwrap(),
        image::ImageFormat::Jpeg => image::codecs::jpeg::JpegEncoder::new(&mut fout)
            .encode(file, width, height, ColorType::Rgba8)
            .unwrap(),
        image::ImageFormat::Gif => image::codecs::gif::GifEncoder::new(fout)
            .encode(file, width, height, ColorType::Rgba8)
            .unwrap(),
        _ => (),
    }
}
