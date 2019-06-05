#[macro_use]
extern crate structopt;
#[macro_use] extern crate log;

use structopt::clap::arg_enum;
use structopt::StructOpt;

use stego::*;

use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::io::prelude::*;

use image::{GenericImageView, RgbaImage, DynamicImage, Rgba, Pixel};

arg_enum! {
    #[derive(Debug)]
    enum DataType {
        Image,
        Text,
        File
    }
}


#[derive(StructOpt, Debug)]
#[structopt(name = "stego", about = "Stegnography at it's finest")]
enum StegoCLI {
    #[structopt(name = "encode")]
    /// Encoding command
    Encode {
        #[structopt(short = "i", long = "input", parse(from_os_str))]
        /// Input image
        input: PathBuf,
        #[structopt(short = "o", long = "output", parse(from_os_str))]
        /// File to save modified image as
        output: PathBuf,

        #[structopt(raw(possible_values = "&DataType::variants()", case_insensitive = "true"))]
        /// Type of data to embed in image
        dtype: DataType, 

        #[structopt(short = "p", long = "payload")]
        /// Data to embed in image (either text message or path)
        payload: Option<String>,

    },
    #[structopt(name = "decode")]
    Decode {
        #[structopt(short = "i", long = "input", parse(from_os_str))]
        /// Input image to decode
        input: PathBuf,

        #[structopt(short = "o", long = "output", parse(from_os_str))]
        /// Path to save hidden image/file to
        output: Option<PathBuf>,

        #[structopt(raw(possible_values = "&DataType::variants()", case_insensitive = "true"))]
        /// Type of data to embed in image
        dtype: DataType, 
    },
}

fn main() {
    print_header();

    pretty_env_logger::init();

    let opt = StegoCLI::from_args();

    match opt {
        StegoCLI::Encode{ input, output, dtype, payload } => {
            // Use the open function to load an image from a Path.
            // ```open``` returns a dynamic image.
            let im: DynamicImage = image::open(&Path::new(&input)).unwrap();
            let mut stego = LSBStego::new(im.clone());

            let mut im2 = RgbaImage::new(0,0);

            match dtype {
                DataType::File => {
                    let mut bytes = Vec::new();
                    let mut file = File::open(&Path::new(&payload.unwrap())).unwrap();

                    file.read_to_end(&mut bytes);

                    im2 = stego.encode_binary(bytes);

                },
                DataType::Image => {

                    let pim: DynamicImage = image::open(&Path::new(&payload.unwrap())).unwrap();
                    im2 = stego.encode_image(pim);

                },
                DataType::Text => {
                    if payload != None {
                        im2 = stego.encode_text(payload.unwrap());
                    }
                    else {
                        let mut msg = String::new();
                        std::io::stdin().read_to_string(&mut msg);

                        im2 = stego.encode_text(msg);
                    }
                }
            } 

            info!("Saving file to {:?}", output);

            im2.save(&Path::new(&output));

        },
        StegoCLI::Decode { input, output, dtype} => {
            // Use the open function to load an image from a Path.
            // ```open``` returns a dynamic image.
            let im: DynamicImage = image::open(&Path::new(&input)).unwrap();
            let mut stego = LSBStego::new(im.clone());

            match dtype {
                DataType::File => {
                    let mut bytes: Vec<u8> = Vec::new();
                    info!("Saving file to {:?}", output);

                    let mut file = File::create(&Path::new(&output.unwrap())).unwrap();


                    file.write_all(&stego.decode_binary());

                },
                DataType::Image => {
                        let im2 = stego.decode_image();

                        info!("Saving file to {:?}", output);

                        im2.save(&Path::new(&output.unwrap()));


                },
                DataType::Text => {
                    // TODO Support hidden image / hiddenfile
                    print!("{}",stego.decode_text());
                }
            } 


        }

    }
}

fn print_header() {
    println!(r"
     _                   
 ___| |_ ___  __ _  ___  
/ __| __/ _ \/ _` |/ _ \ 
\__ \ ||  __/ (_| | (_) |
|___/\__\___|\__, |\___/ 
             |___/       
=========================
Created by: Avery Wagar
")
}
