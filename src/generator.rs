use rand::{distributions::WeightedIndex, prelude::Distribution};

use rand::thread_rng;

use image::{GenericImage, GenericImageView, Rgba};

use std::collections::HashMap;

use image::DynamicImage;

pub struct Generator {
    map: HashMap<Rgba<u8>, String>,
    inv_map: HashMap<String, Rgba<u8>>,
    weights: HashMap<Rgba<u8>, u64>,
    triplets: Vec<Triplet>,
}

impl Generator {
    pub fn new(input_image: &DynamicImage) -> Self {
        let mut out = Generator {
            map: HashMap::new(),
            inv_map: HashMap::new(),
            weights: HashMap::new(),
            triplets: vec![],
        };
        out.calc_pairs(input_image);
        out
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Triplet {
    pub original: String,
    pub next_to: String,
    pub orientation: Direction,
}
impl Generator {
    fn calc_pairs(&mut self, input: &DynamicImage) -> Vec<Triplet> {
        let directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut out = vec![];
        let mut index = 0;

        for (_, _, col) in input.pixels() {
            self.map.entry(col).or_insert_with(|| {
                index += 1;
                index.to_string()
            });
            self.inv_map.entry(index.to_string()).or_insert_with(|| col);
            match self.weights.contains_key(&col) {
                false => {
                    self.weights.insert(col, 1);
                }
                true => {
                    let val = self.weights.get_mut(&col).unwrap();
                    *val += 1;
                }
            }
        }
        for (x, y, color) in input.pixels() {
            for dir in &directions {
                match dir {
                    Direction::Up => {
                        if y != 0 && input.in_bounds(x, y - 1) {
                            let px = input.get_pixel(x, y - 1);
                            out.push(Triplet {
                                original: self.map.get(&color).unwrap().clone(),
                                next_to: self.map.get(&px).unwrap().clone(),
                                orientation: Direction::Up,
                            });
                            out.push(Triplet {
                                original: self.map.get(&px).unwrap().clone(),
                                next_to: self.map.get(&color).unwrap().clone(),
                                orientation: Direction::Down,
                            });
                        }
                    }
                    Direction::Down => {
                        if input.in_bounds(x, y + 1) {
                            let px = input.get_pixel(x, y + 1);
                            out.push(Triplet {
                                original: self.map.get(&color).unwrap().clone(),
                                next_to: self.map.get(&px).unwrap().clone(),
                                orientation: Direction::Down,
                            });
                            out.push(Triplet {
                                original: self.map.get(&px).unwrap().clone(),
                                next_to: self.map.get(&color).unwrap().clone(),
                                orientation: Direction::Up,
                            });
                        }
                    }
                    Direction::Left => {
                        if x != 0 && input.in_bounds(x - 1, y) {
                            let px = input.get_pixel(x - 1, y);
                            out.push(Triplet {
                                original: self.map.get(&color).unwrap().clone(),
                                next_to: self.map.get(&px).unwrap().clone(),
                                orientation: Direction::Left,
                            });
                            out.push(Triplet {
                                original: self.map.get(&px).unwrap().clone(),
                                next_to: self.map.get(&color).unwrap().clone(),
                                orientation: Direction::Right,
                            });
                        }
                    }
                    Direction::Right => {
                        if input.in_bounds(x + 1, y) {
                            let px = input.get_pixel(x + 1, y);
                            out.push(Triplet {
                                original: self.map.get(&color).unwrap().clone(),
                                next_to: self.map.get(&px).unwrap().clone(),
                                orientation: Direction::Right,
                            });
                            out.push(Triplet {
                                original: self.map.get(&px).unwrap().clone(),
                                next_to: self.map.get(&color).unwrap().clone(),
                                orientation: Direction::Left,
                            });
                        }
                    }
                }
            }
        }

        out.sort();
        out.dedup();
        out
    }

    pub fn gen_smart(&mut self, width: usize, height: usize) -> DynamicImage {
        let mut generated = self.generate(width, height);
        while generated.is_err() {
            println!("Try again");
            generated = self.generate(width, height);
        }
        generated.expect("Somehow this failed")
    }

    fn generate(&mut self, width: usize, height: usize) -> Result<DynamicImage, String> {
        let mut rng = thread_rng();
        // all the waves
        let mut waves = vec![vec![self.map.keys().collect::<Vec<&Rgba<u8>>>(); width]; height];
        // seed with a random square
        let mut seed = Some((0, 0));
        let mut temp = (0, 0);
        // go until all squares are all length of 1
        while waves.iter().flatten().any(|x| x.len() != 1) {
            if waves.iter().flatten().any(|x| x.is_empty()) {
                return Err("Ran into a contradiction".to_string());
            }

            // pick the start square
            let start_square = match seed {
                Some(square) => {
                    seed = None;
                    square
                }
                None => {
                    // // find the square with the smallest entropy
                    // let mut square = (0, 0);
                    // let mut smallest_entropy = 1000.0;
                    // for (y, row) in waves.iter_mut().enumerate() {
                    //     for (x, wave) in row.iter_mut().enumerate() {
                    //         if wave.len() == 1 {
                    //             continue;
                    //         }
                    //         println!("{}", wave.len());
                    //         // calculate shannon entropy
                    //         let weight_sum: f64 =
                    //             wave.iter().map(|&x| *weights.get(x).unwrap() as f64).sum();

                    //         let shannon_entropy = weight_sum.log(10.)
                    //             - (wave
                    //                 .iter()
                    //                 .map(|&x| {
                    //                     *weights.get(x).unwrap() as f64
                    //                         * (*weights.get(x).unwrap() as f64).log(10.)
                    //                 })
                    //                 .sum::<f64>()
                    //                 / weight_sum);
                    //         if shannon_entropy < smallest_entropy {
                    //             smallest_entropy = shannon_entropy;
                    //             square = (x, y);
                    //         }
                    //         /*
                    //           shannon_entropy_for_square =
                    //           log(sum(weight)) -
                    //           (sum(weight * log(weight)) / sum(weight))
                    //         */
                    //     }
                    // }

                    temp.0 += 1;
                    if temp.0 == width {
                        temp.0 = 0;
                        temp.1 += 1;
                    }
                    temp
                }
            };

            // collapse it given the weights
            let dist = waves[start_square.1][start_square.0]
                .iter()
                .map(|&x| *self.weights.get(x).unwrap())
                .collect::<Vec<u64>>();
            let ind = WeightedIndex::new(&dist).unwrap();
            waves[start_square.1][start_square.0] =
                vec![waves[start_square.1][start_square.0][ind.sample(&mut rng)]];
            let color = waves[start_square.1][start_square.0][0];
            // propogate the rules for the adjacent square

            let mut up_possibles = vec![];
            let mut left_possibles = vec![];
            let mut right_possibles = vec![];
            let mut down_possibles = vec![];
            for rule in &self.triplets {
                if rule.original == *self.map.get(color).unwrap() {
                    match rule.orientation {
                        Direction::Left => {
                            left_possibles.push(*self.inv_map.get(&rule.next_to).unwrap())
                        }
                        Direction::Right => {
                            right_possibles.push(*self.inv_map.get(&rule.next_to).unwrap())
                        }
                        Direction::Up => {
                            up_possibles.push(*self.inv_map.get(&rule.next_to).unwrap())
                        }
                        Direction::Down => {
                            down_possibles.push(*self.inv_map.get(&rule.next_to).unwrap())
                        }
                    }
                }
            }
            let directions = vec![
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ];
            for dir in directions {
                let check_square = match dir {
                    Direction::Left => {
                        if start_square.0 != 0 {
                            &mut waves[start_square.1][start_square.0 - 1]
                        } else {
                            continue;
                        }
                    }
                    Direction::Right => {
                        if start_square.0 + 1 != width {
                            &mut waves[start_square.1][start_square.0 + 1]
                        } else {
                            continue;
                        }
                    }
                    Direction::Up => {
                        if start_square.1 != 0 {
                            &mut waves[start_square.1 - 1][start_square.0]
                        } else {
                            continue;
                        }
                    }
                    Direction::Down => {
                        if start_square.1 + 1 != height {
                            &mut waves[start_square.1 + 1][start_square.0]
                        } else {
                            continue;
                        }
                    }
                };
                if check_square.len() == 1 {
                    continue;
                }
                let possibles = match dir {
                    Direction::Left => left_possibles.clone(),
                    Direction::Right => right_possibles.clone(),
                    Direction::Up => up_possibles.clone(),
                    Direction::Down => down_possibles.clone(),
                };
                let same = check_square
                    .iter()
                    .filter(|&x| possibles.contains(x))
                    .collect::<Vec<&&Rgba<u8>>>();
                *check_square = same.iter().map(|&&x| x).collect();
            }
        }

        let mut out = DynamicImage::new_rgb8(width as u32, height as u32);
        for (y, row) in waves.iter().enumerate() {
            for (x, wave) in row.iter().enumerate() {
                out.put_pixel(x as u32, y as u32, *wave[0]);
            }
        }

        Ok(out)
    }
}
