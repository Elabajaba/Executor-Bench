use glam::Vec2;
use polyanya::{Mesh, PolyanyaFile};
use std::io::{self, BufRead};

pub fn get_mesh() -> Mesh {
    let mesh: Mesh = PolyanyaFile::from_file("meshes/aurora-merged.mesh").into();

    mesh
}

// Scenario reading is copied from https://github.com/vleue/polyanya/blob/main/examples/scenario_runner.rs

#[derive(Clone)]
pub struct Scenario {
    pub from: Vec2,
    pub to: Vec2,
}

#[derive(Clone)]
pub struct Scenarios(pub Vec<Scenario>);

impl Scenarios {
    pub fn from_file(path: &str) -> Scenarios {
        let file = std::fs::File::open(path).unwrap();

        Scenarios(
            io::BufReader::new(file)
                .lines()
                .skip(1)
                .flatten()
                .map(|line| {
                    let mut split = line.split('\t');
                    Scenario {
                        from: Vec2::new(
                            split.nth(4).unwrap().parse::<i32>().unwrap() as f32,
                            split.next().unwrap().parse::<i32>().unwrap() as f32,
                        ),
                        to: Vec2::new(
                            split.next().unwrap().parse::<i32>().unwrap() as f32,
                            split.next().unwrap().parse::<i32>().unwrap() as f32,
                        ),
                    }
                })
                .collect(),
        )
    }
}
