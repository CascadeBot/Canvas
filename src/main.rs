#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use rocket::response::Content;
use rocket::http::ContentType;
use std::io::Read;
use std::convert::TryInto;
use image::ImageDecoder;
use std::fs::File;

use zerocopy::{AsBytes};

use image::{RgbaImage, ColorType};
use image::png::PngDecoder;
use imageproc::drawing::{Blend, draw_filled_rect_mut};
use imageproc::rect::Rect;

use image::Rgba;

fn main() {
    rocket::ignite().mount("/test.png", routes![get_image]).launch();
}

#[get("/")]
fn get_image() -> Content<Vec<u8>> {


    let decoder = PngDecoder::new(File::open("bg.png").unwrap()).unwrap();

    let width = decoder.dimensions().0;
    let height = decoder.dimensions().1;

    let data = read_8bit_image(decoder);

    let mut base_image = Blend(RgbaImage::from_raw(width, height, data).unwrap());

    let rect = Rect::at(15,15).of_size(20, 20);

    let red = Rgba([255u8, 0u8, 0u8, 255u8]);

    draw_filled_rect_mut(&mut base_image, rect, red);

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

fn read_8bit_image<T : Read>(decoder: PngDecoder<T>) -> Result<Vec<u8>, &'static str>  {
    if decoder.color_type() != ColorType::Rgba8 {
        Err("The image must be a 8-bit RGB image with an alpha channel")
    } else {
        if let Ok(numBytes) = decoder.total_bytes().try_into() {
            let mut buf: Vec<u8> = vec![0; numBytes];
            decoder.read_image(buf.as_bytes_mut()).expect("Could not read image into buffer");
            Ok(buf)
        } else {
            Err("Could not convert total bytes to usize")
        }
    }
}
