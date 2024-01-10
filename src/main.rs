use std::{fs, path::PathBuf};
use clap::{Parser};
use serde_json::Value;
extern crate image;
use std::io::prelude::*;
use image::{GenericImage, GenericImageView, ImageBuffer, io, Rgba};
use std::time::{SystemTime, UNIX_EPOCH};

async fn load_palette(palette_name : &str) -> Vec<(usize, usize, usize)> {
    let default_palette = vec![
        (140, 143, 174),
        (88, 69, 99),
        (62, 33, 55),
        (154, 99, 72),
        (215, 155, 125),
        (245, 237, 186),
        (192, 199, 65),
        (100, 125, 52),
        (228, 148, 58),
        (157, 48, 59),
        (210, 100, 113),
        (112, 55, 127),
        (126, 196, 193),
        (52, 133, 157),
        (23, 67, 75),
        (31, 14, 28),
    ];

    let mut exe_dir = std::env::current_exe();
    let mut exe_dir = exe_dir.unwrap();
    exe_dir.pop();
    exe_dir.push("palettes");
    if !exe_dir.exists() {
        std::fs::create_dir(&exe_dir).unwrap();
    }

    exe_dir.push(format!("{}.hex", palette_name));

    if exe_dir.exists() {
        return load_palette_from_file(&exe_dir);
    }

    let res = reqwest::blocking::get(format!("https://Lospec.com/palette-list/{}.json", palette_name));

    if let Err(error) = res {
        println!("Unable to connect to Lospec.com and palette '{}' is not in local files, resorting to default palette.", palette_name);
        return default_palette;
    }

    let res = res.unwrap().json::<Value>();

    if let Err(error) = res {
        println!("Palette '{}' is not in the correct format. Resorting to default pallete.", palette_name);
        return default_palette;
    }

    let mut res = res.unwrap();

    if let Some(value) = res.get("error") {
        println!("Error loading palette '{}' from server: {}. Resorting to the default palette.", palette_name, value.as_str().unwrap());
        return default_palette;
    }

    let mut out = String::new();

    if let Value::Object(obj) = res {
        for color in obj["colors"].as_array().unwrap() {
            out = format!("{}#{}\n", out, color.as_str().unwrap());
        }
    }

    fs::write(exe_dir.clone(), out).expect("Unable to save palette.");

    load_palette_from_file(&exe_dir)
}

fn load_palette_from_file(path : &PathBuf) ->  Vec<(usize, usize, usize)> {
    let content = fs::read_to_string(path)
        .expect("Critical Error: palette should exist locally.");

    let mut pallete = Vec::new();

    for line in content.lines() {
        let color = csscolorparser::parse(line).unwrap().to_rgba8();
        pallete.push((color[0] as usize, color[1] as usize, color[2] as usize));
    }

    pallete
}

fn pixelate_image(input_path: &str, output_path: &str, config: &PixitConfig) -> Result<(), image::ImageError> {
    // Get the pixel size from the config
    let pixel_size = config.size;

    // Load the input image
    let img = io::Reader::open(input_path)?.decode()?;

    // Get the dimensions of the image
    let (width, height) = img.dimensions();

    // Create a new ImageBuffer with the same dimensions as the input image
    let mut output_img = ImageBuffer::new(width, height);

    // Iterate over the image in blocks of pixel_size x pixel_size
    for y in (0..height).step_by(pixel_size as usize) {
        for x in (0..width).step_by(pixel_size as usize) {
            // Calculate the average color of the block
            let mut total_color:  [u64; 4] = [0, 0, 0, 0];
            let mut pixel_count = 0;

            for j in 0..pixel_size {
                for i in 0..pixel_size {
                    if x + i < width && y + j < height {
                        let pixel = img.get_pixel(x + i, y + j);
                        for k in 0..4 {
                            total_color[k] += pixel[k] as u64;
                        }
                        pixel_count += 1;
                    }
                }
            }

            // Calculate the average color for the block
            let avg_color = Rgba([
                (total_color[0] / pixel_count) as u8,
                (total_color[1] / pixel_count) as u8,
                (total_color[2] / pixel_count) as u8,
                (total_color[3] / pixel_count) as u8,
            ]);

            // get the closest color in the palette
            let closest_color: Rgba<u8> = config.palette.iter().enumerate().map(|(i, color)| {
                let r = color.0 as u8;
                let g = color.1 as u8;
                let b = color.2 as u8;
                let distance = (r as f64 - avg_color[0] as f64).powi(2) + (g as f64 - avg_color[1] as f64).powi(2) + (b as f64 - avg_color[2] as f64).powi(2);
                (i, distance)
            }).min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).map(|(i, _)| {
                let color = config.palette[i];
                Rgba([color.0 as u8, color.1 as u8, color.2 as u8, 255u8])
            }).unwrap_or(Rgba([0, 0, 0, 0]));

            // Fill the block with the average color
            for j in 0..pixel_size {
                for i in 0..pixel_size {
                    if x + i < width && y + j < height {
                        output_img.put_pixel(x + i, y + j, closest_color);
                    }
                }
            }

        }
    }

    // shrink the output image
    // output_img.wid

    if config.large {
        output_img.save(output_path)?;
    }  else {
        let output_img = image::imageops::resize(&output_img, width / pixel_size, height / pixel_size, image::imageops::FilterType::Nearest);
        output_img.save(output_path)?;
    }

    Ok(())
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    ///The palatte which will be used to pixelate the image
    #[arg(short, long, value_name = "PALETTE")]
    palette: Option<String>,
    ///The number of pixels making up each pixel in the output image
    #[arg(short, long, value_name = "SIZE")]
    size: Option<u32>,
    ///The name of the output file
    #[arg(short, long, value_name = "NAME")]
    name: Option<String>,
    ///Exports the image at its input size
    #[arg(short, long)]
    large: bool,
    ///Image to pixelate
    #[arg(short, long)]
    image: Option<String>,
    ///File of images to pixelate
    #[arg(short, long)]
    directory: Option<String>,
}

struct PixitConfig {
    size: u32,
    palette: Vec<(usize, usize, usize)>,
    large: bool
}

fn main() -> std::io::Result<()> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

    let cli = Cli::parse();

    let output_file = cli.name.unwrap_or(format!("pixit:{}:{since_epoch}", cli.palette.as_ref().unwrap_or(&String::from("default")).split(".").next().unwrap()));
    let _ = fs::create_dir(&output_file);

    let palette = match cli.palette {
        None => {
            vec![
                (140, 143, 174),
                (88, 69, 99),
                (62, 33, 55),
                (154, 99, 72),
                (215, 155, 125),
                (245, 237, 186),
                (192, 199, 65),
                (100, 125, 52),
                (228, 148, 58),
                (157, 48, 59),
                (210, 100, 113),
                (112, 55, 127),
                (126, 196, 193),
                (52, 133, 157),
                (23, 67, 75),
                (31, 14, 28),
            ]
        }
        Some(palette_name) => {
            futures::executor::block_on(load_palette(palette_name.as_str()))
        }
    };

    let config = PixitConfig {
        size: cli.size.unwrap_or(12),
        palette,
        large: cli.large
    };


    let image = cli.image;
    let directory = cli.directory;
    match (image, directory) {
        (Some(image_path), None) => {
            let image_name = image_path.split("/").last().unwrap();
            let output_path = format!("{}/{}", output_file, image_name);
            // save image as png always
            let mut output_path: Vec<&str> = output_path.split(".").collect();
            let _ = output_path.pop();
            let output_path = output_path.join(".");
            let output_path = format!("{output_path}.png");
            if let Err(err) = pixelate_image(image_path.as_str(), output_path.as_str(), &config) {
                // remove file
                let _ = fs::remove_dir_all(&output_file);
                eprintln!("Error: {:?}", err);
            } else {
                println!("Image {image_name} pixelated and saved to {output_path}");
            }
        }
        (None, Some(directory)) => {
            let dir = fs::read_dir(directory.clone())?;
            dir
                .filter(|dir_entry| dir_entry.is_ok())
                .map(|dir_entry| dir_entry.unwrap().path())
                .filter(|path| {
                    path.to_str().unwrap().ends_with(".png")
                        || path.to_str().unwrap().ends_with(".jpg")
                        || path.to_str().unwrap().ends_with(".jpeg")
                })
                .for_each(|path| {
                    let image_name = path.to_str().unwrap().split("/").last().unwrap();
                    let path = path.to_str().unwrap();
                    let output_path = format!("{}/{}", output_file, image_name);
                    // save image as png always
                    let mut output_path: Vec<&str> = output_path.split(".").collect();
                    let _ = output_path.pop();
                    let output_path = output_path.join(".");
                    let output_path = format!("{output_path}.png");
                    if let Err(err) = pixelate_image(path, output_path.as_str(), &config) {
                        // remove file
                        let _ = fs::remove_dir_all(&output_file);
                        eprintln!("Error: {:?}", err);
                    } else {
                        println!("Image {image_name} pixelated and saved to {output_path}");
                    }
                });
            println!("File: {}", directory);
        }
        (Some(image), Some(file)) => {
            println!("You can't specify both image and file")
        }
        (None, None) => {
            println!("You must specify either an image or a file")
        }
    }

    Ok(())
}
