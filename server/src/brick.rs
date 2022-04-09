use std::fs;

#[derive(Default, Debug)]
struct Vector3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Debug)]
pub struct Brick {
    net_id: u32,

    name: String,
    colour: String,
    shape: String,

    position: Vector3,
    scale: Vector3,
    light_enabled: bool,
    light_colour: String,
    light_range: u32,
    visibility: f32,
    rotation: i32,

    collision: bool,
}

#[derive(Default)]
pub struct Environment {
    ambient: String,
    sky_colour: String,
    base_colour: String,

    weather: String,
}

fn hex(r: f32, g: f32, b: f32) -> String {
    format!(
        "{:02X}{:02X}{:02X}",
        r as f32 as u8, g as f32 as u8, b as f32 as u8
    )
}

pub fn load_from_file(file_name: String) -> bool {
    let contents = fs::read_to_string(file_name).expect("Something went wrong reading the file");

    let lines = contents.split("\n");
    let mut total_lines = 0;

    let mut environment = Environment::default();

    let mut bricks = Vec::<Brick>::new();
    bricks.push(Brick {
        collision: true,
        ..Brick::default()
    });

    for mut line in lines {
        total_lines += 1;

        line = line.trim();

        match total_lines {
            1 => {
                if line != "B R I C K  W O R K S H O P  V0.2.0.0" {
                    println!("Invalid file format");
                    return false;
                }
                continue;
            }
            3 => {
                let colours: Vec<&str> = line.split(" ").collect();
                environment.ambient = hex(
                    colours[0].parse::<f32>().unwrap(),
                    colours[1].parse::<f32>().unwrap(),
                    colours[2].parse::<f32>().unwrap(),
                );
                continue;
            }
            4 => {
                let colours: Vec<&str> = line.split(" ").collect();
                println!("{:?}", colours);
                environment.base_colour = hex(
                    colours[0].parse::<f32>().unwrap(),
                    colours[1].parse::<f32>().unwrap(),
                    colours[2].parse::<f32>().unwrap(),
                );
                continue;
            }
            5 => {
                let colours: Vec<&str> = line.split(" ").collect();
                environment.sky_colour = hex(
                    colours[0].parse::<f32>().unwrap(),
                    colours[1].parse::<f32>().unwrap(),
                    colours[2].parse::<f32>().unwrap(),
                );
                continue;
            }
            _ => {}
        }

        let mut data: Vec<&str> = line.split(" ").collect();
        let attribute = data[0].replace("+", "");
        data.remove(0);
        let value = data.join(" ");

        let last_brick = bricks.last_mut().unwrap();

        match attribute.as_str() {
            "NAME" => {
                last_brick.name = value;
                continue;
            }
            "ROT" => {
                last_brick.rotation = value.parse::<i32>().unwrap();
                continue;
            }
            "SHAPE" => {
                last_brick.shape = value;
                continue;
            }
            "NOCOLLISION" => {
                last_brick.collision = false;
                continue;
            }
            "LIGHT" => {
                let colours: Vec<&str> = value.split(" ").collect();

                last_brick.light_enabled = true;
                last_brick.light_range = colours[3].parse::<u32>().unwrap();
                last_brick.light_colour = hex(
                    colours[0].parse::<f32>().unwrap(),
                    colours[1].parse::<f32>().unwrap(),
                    colours[2].parse::<f32>().unwrap(),
                );
                continue;
            }
            _ => {}
        }

        if data.len() == 9 {
            last_brick.position = Vector3 {
                x: data[0].parse::<f32>().unwrap(),
                y: data[1].parse::<f32>().unwrap(),
                z: data[2].parse::<f32>().unwrap(),
            };
            last_brick.scale = Vector3 {
                x: data[3].parse::<f32>().unwrap(),
                y: data[4].parse::<f32>().unwrap(),
                z: data[5].parse::<f32>().unwrap(),
            };

            last_brick.colour = hex(
                data[6].parse::<f32>().unwrap(),
                data[7].parse::<f32>().unwrap(),
                data[8].parse::<f32>().unwrap(),
            );

            bricks.push(Brick {
                collision: true,
                ..Brick::default()
            });
        }
    }

    return true;
}
