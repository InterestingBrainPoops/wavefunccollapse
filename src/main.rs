use std::collections::HashMap;

use image::{DynamicImage, GenericImage, GenericImageView, Rgba};
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng};

fn main() {
    println!("Hello, world!");
    let img = image::open("../origin.png").expect("File not found!");

    let mut map = HashMap::new();
    let mut inv_map = HashMap::new();
    let mut weights = HashMap::new();
    let pairs = calc_pairs(img, &mut map, &mut inv_map, &mut weights);
    println!("{}", pairs.len());
    println!("{:?}", pairs);

    let mut generated = generate(&pairs, 3, 1, &map, &inv_map, &weights);
    while let Err(_) = generated {
        generated = generate(&pairs, 31, 10, &map, &inv_map, &weights);
    }
    generated.expect("e").save("./output.png").unwrap();
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
    original: String,
    next_to: String,
    orientation: Direction,
}

fn calc_pairs(
    input: DynamicImage,
    map: &mut HashMap<Rgba<u8>, String>,
    inv_map: &mut HashMap<String, Rgba<u8>>,
    weights: &mut HashMap<Rgba<u8>, u64>,
) -> Vec<Triplet> {
    let directions = vec![
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];
    let mut out = vec![];
    let mut index = 0;

    for (_, _, col) in input.pixels() {
        map.entry(col).or_insert_with(|| {
            index += 1;
            index.to_string()
        });
        inv_map.entry(index.to_string()).or_insert_with(|| col);
        match weights.contains_key(&col) {
            false => {
                weights.insert(col, 1);
            }
            true => {
                let val = weights.get_mut(&col).unwrap();
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
                            original: map.get(&color).unwrap().clone(),
                            next_to: map.get(&px).unwrap().clone(),
                            orientation: Direction::Up,
                        });
                        out.push(Triplet {
                            original: map.get(&px).unwrap().clone(),
                            next_to: map.get(&color).unwrap().clone(),
                            orientation: Direction::Down,
                        });
                    }
                }
                Direction::Down => {
                    if input.in_bounds(x, y + 1) {
                        let px = input.get_pixel(x, y + 1);
                        out.push(Triplet {
                            original: map.get(&color).unwrap().clone(),
                            next_to: map.get(&px).unwrap().clone(),
                            orientation: Direction::Down,
                        });
                        out.push(Triplet {
                            original: map.get(&px).unwrap().clone(),
                            next_to: map.get(&color).unwrap().clone(),
                            orientation: Direction::Up,
                        });
                    }
                }
                Direction::Left => {
                    if x != 0 && input.in_bounds(x - 1, y) {
                        let px = input.get_pixel(x - 1, y);
                        out.push(Triplet {
                            original: map.get(&color).unwrap().clone(),
                            next_to: map.get(&px).unwrap().clone(),
                            orientation: Direction::Left,
                        });
                        out.push(Triplet {
                            original: map.get(&px).unwrap().clone(),
                            next_to: map.get(&color).unwrap().clone(),
                            orientation: Direction::Right,
                        });
                    }
                }
                Direction::Right => {
                    if input.in_bounds(x + 1, y) {
                        let px = input.get_pixel(x + 1, y);
                        out.push(Triplet {
                            original: map.get(&color).unwrap().clone(),
                            next_to: map.get(&px).unwrap().clone(),
                            orientation: Direction::Right,
                        });
                        out.push(Triplet {
                            original: map.get(&px).unwrap().clone(),
                            next_to: map.get(&color).unwrap().clone(),
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

fn generate(
    triplets: &Vec<Triplet>,
    width: usize,
    height: usize,
    map: &HashMap<Rgba<u8>, String>,
    inv_map: &HashMap<String, Rgba<u8>>,
    weights: &HashMap<Rgba<u8>, u64>,
) -> Result<DynamicImage, String> {
    let mut rng = thread_rng();
    // all the waves
    let mut waves = vec![vec![map.keys().collect::<Vec<&Rgba<u8>>>(); width]; height];
    // seed with a random square
    let mut seed = Some((0, 0));
    // go until all squares are all length of 1
    while waves.iter().flatten().any(|x| x.len() != 1) {
        println!("{:#?}", waves);
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
                // find the square with the smallest entropy
                let mut square = (0, 0);
                let mut smallest_entropy = 1000.0;
                for (y, row) in waves.iter_mut().enumerate() {
                    for (x, wave) in row.iter_mut().enumerate() {
                        if wave.len() == 1 {
                            continue;
                        }
                        println!("{}", wave.len());
                        // calculate shannon entropy
                        let weight_sum: f64 =
                            wave.iter().map(|&x| *weights.get(x).unwrap() as f64).sum();

                        let shannon_entropy = weight_sum.log(10.)
                            - (wave
                                .iter()
                                .map(|&x| {
                                    *weights.get(x).unwrap() as f64
                                        * (*weights.get(x).unwrap() as f64).log(10.)
                                })
                                .sum::<f64>()
                                / weight_sum);
                        if shannon_entropy < smallest_entropy {
                            smallest_entropy = shannon_entropy;
                            square = (x, y);
                        }
                        /*
                          shannon_entropy_for_square =
                          log(sum(weight)) -
                          (sum(weight * log(weight)) / sum(weight))
                        */
                    }
                }
                square
            }
        };

        // collapse it given the weights
        let dist = waves[start_square.1][start_square.0]
            .iter()
            .map(|&x| *weights.get(x).unwrap())
            .collect::<Vec<u64>>();
        println!("dist : {:?}", dist);
        let ind = WeightedIndex::new(&dist).unwrap();
        waves[start_square.1][start_square.0] =
            vec![waves[start_square.1][start_square.0][ind.sample(&mut rng)]];
        let color = waves[start_square.1][start_square.0][0];
        // propogate the rules for the adjacent square

        let mut up_possibles = vec![];
        let mut left_possibles = vec![];
        let mut right_possibles = vec![];
        let mut down_possibles = vec![];
        for rule in triplets {
            if rule.original == *map.get(color).unwrap() {
                match rule.orientation {
                    Direction::Left => left_possibles.push(*inv_map.get(&rule.next_to).unwrap()),
                    Direction::Right => right_possibles.push(*inv_map.get(&rule.next_to).unwrap()),
                    Direction::Up => up_possibles.push(*inv_map.get(&rule.next_to).unwrap()),
                    Direction::Down => down_possibles.push(*inv_map.get(&rule.next_to).unwrap()),
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
            let check_square;
            let possibles;
            match dir {
                Direction::Left => {
                    check_square = {
                        if start_square.0 != 0 {
                            &mut waves[start_square.1][start_square.0 - 1]
                        } else {
                            continue;
                        }
                    };
                }
                Direction::Right => {
                    check_square = {
                        if start_square.0 + 1 != width {
                            &mut waves[start_square.1][start_square.0 + 1]
                        } else {
                            continue;
                        }
                    };
                }
                Direction::Up => {
                    check_square = {
                        if start_square.1 != 0 {
                            &mut waves[start_square.1 - 1][start_square.0]
                        } else {
                            continue;
                        }
                    };
                }
                Direction::Down => {
                    check_square = {
                        if start_square.1 + 1 != height {
                            &mut waves[start_square.1 + 1][start_square.0]
                        } else {
                            continue;
                        }
                    };
                }
            }
            if check_square.len() == 1 {
                continue;
            }
            match dir {
                Direction::Left => possibles = left_possibles.clone(),
                Direction::Right => possibles = right_possibles.clone(),
                Direction::Up => possibles = up_possibles.clone(),
                Direction::Down => possibles = down_possibles.clone(),
            }
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
