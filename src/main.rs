use std::collections::HashMap;

use crate::generator::Generator;

fn main() {
    println!("Hello, world!");
    let img = image::open("../origin.png").expect("File not found!");

    let mut generator = Generator::new(&img);
    for i in 0..100 {
        let generated = generator.gen_smart(3, 3);
        generated.save(format!("./output_{}.png", i)).unwrap();
    }
}
mod generator;
