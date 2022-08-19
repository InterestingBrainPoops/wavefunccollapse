use clap::Parser;
use std::{collections::HashMap, path::PathBuf, time::Instant};

use crate::generator::Generator;
#[derive(Parser, Debug)]
struct Args {
    #[clap(long, value_parser)]
    origin_image: PathBuf,
    #[clap(long, value_parser)]
    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    println!("Hello, world!");
    let img = image::open(args.origin_image).expect("File not found!");

    let mut generator = Generator::new(&img, 2, 2);
    for i in 0..1 {
        let t0 = Instant::now();
        let generated = generator.gen_smart(10, 10);
        println!("{:?}", Instant::now() - t0);

        generated
            .save(format!(
                "{}output_{}.png",
                args.output_path.to_str().unwrap(),
                i
            ))
            .unwrap();
    }
}
mod generator;
