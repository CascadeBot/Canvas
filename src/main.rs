#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use image::ImageDecoder;
use rocket::http::ContentType;
use rocket::http::RawStr;
use rocket::response::Content;
use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use rusttype::{Font, Scale};

use zerocopy::AsBytes;

use image::png::PngDecoder;
use image::{ColorType, RgbaImage};
use imageproc::drawing::{draw_filled_rect_mut, Blend, draw_text_mut};
use imageproc::rect::Rect;

use image::Rgba;

fn main() {
    rocket::ignite().mount("/", routes![get_image]).launch();
}

#[get("/<file_name>")]
fn get_image(file_name: &RawStr) -> Content<Vec<u8>> {
    let decoder = PngDecoder::new(File::open(file_name.as_str()).unwrap()).unwrap();

    let width = decoder.dimensions().0;
    let height = decoder.dimensions().1;

    let data = read_8bit_image(decoder).expect("could not decode image");

    let mut base_image = Blend(RgbaImage::from_raw(width, height, data).unwrap());

    let rect = Rect::at(15, 15).of_size(20, 20);

    let red = Rgba([255u8, 0u8, 0u8, 255u8]);

    draw_filled_rect_mut(&mut base_image, rect, red);

    let font_data: &[u8] = include_bytes!("../Consolas.ttf");
    let font: Font<'static> = Font::from_bytes(font_data).expect("Couldn't parse font!");

    draw_text_mut(&mut base_image, Rgba([255u8, 255u8, 255u8, 255u8]), 30, 30, Scale {x: 20.0, y: 20.0}, &font, "Hi there!");

    let raw_data = base_image.0.into_raw();

    let mut output_data = Vec::<u8>::new();

    {
        let mut encoder = png::Encoder::new(&mut output_data, width, height);
        encoder.set_color(png::ColorType::RGBA);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&raw_data).unwrap();
    }

    Content(ContentType::PNG, output_data)
}

fn read_8bit_image<T: Read>(decoder: PngDecoder<T>) -> Result<Vec<u8>, &'static str> {
    if decoder.color_type() != ColorType::Rgba8 {
        Err("the image must be a 8-bit RGB image with an alpha channel")
    } else if let Ok(num_bytes) = decoder.total_bytes().try_into() {
        let mut buf: Vec<u8> = vec![0; num_bytes];
        decoder
            .read_image(buf.as_bytes_mut())
            .expect("could not read image into buffer");
        Ok(buf)
    } else {
        Err("could not convert total bytes to usize")
    }
}
