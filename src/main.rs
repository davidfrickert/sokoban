extern crate piston_window;
extern crate opengl_graphics;
extern crate rand;
extern crate nalgebra as na;

use std::collections::HashMap;
use na::core::DMatrix;
use rand::Rng;
use opengl_graphics::Texture as Tex;
use piston_window::{RenderArgs, UpdateArgs, OpenGL, PistonWindow, WindowSettings, Button, Key,
                    Transformed, image, clear, text, AdvancedWindow, RenderEvent, PressEvent,
                    UpdateEvent};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use std::time::*;
use structs::*;
use std::fs;

pub mod structs;
#[derive(PartialEq, Copy, Clone, Debug)]
enum ObjectType {
    Blocking,
    Passing,
    Crate,
    Target,
}
#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}



struct Player<'a> {
    sprite: &'a Tex,
    position: Position,
}
struct Object<'a> {
    sprite: (String, &'a Tex),
    obj_type: ObjectType,
    position: Position,
}
struct CollisionMatrix {
    coll: DMatrix<bool>,
}
struct Score {
    moves: i32,
    scored: i32,
}
struct Game<'a> {
    gl: GlGraphics,
    floor: Vec<Object<'a>>,
    special: Vec<Object<'a>>,
    player: Player<'a>,
    obj_tex: &'a GameTextures,
    start_t: SystemTime,
    score: Score,
    targets_left: i32,
}
struct PlayerTextures {
    player_n: Tex,
    player_s: Tex,
    player_e: Tex,
    player_w: Tex,
}
struct GameTextures {
    wall: Tex,
    floor: Tex,
    _crate: HashMap<String, Tex>,
    b_crate: HashMap<String, Tex>,
    targets: HashMap<String, Tex>,
}
impl CollisionMatrix {
    pub fn check_surroundings(&self, i: (usize, usize)) -> bool {
        self.next(i, 1, 0) || self.next(i, 0, 1) || self.next(i, -1, 0) ||
        self.next(i, 0, -1) || self.next(i, 2, 0) || self.next(i, -2, 0) ||
        self.next(i, 0, -2) || self.next(i, 0, 2)
    }
    pub fn next(&self, ind: (usize, usize), x: i32, y: i32) -> bool {
        if ind.0 as i32 + x < 0 || ind.1 as i32 + y < 0 || ind.0 as i32 + x > 9 ||
           ind.1 as i32 + y > 9 {
            false
        } else {
            self.coll[((ind.0 as i32 + x) as usize, (ind.1 as i32 + y) as usize)]
        }
    }
}
impl Score {
    pub fn new() -> Score {
        Score {
            moves: 0,
            scored: 0,
        }
    }
}

impl<'a> Game<'a> {
    fn check(&mut self, position: &Position, direction: Direction) {

        use Direction::*;

        let mut next = (*position).clone();
        let mut crate_found = -1;
        let mut target_found = -1;
        let mut success = true;
        let dir = |x| {
            let mut n = x;
            match direction {
                Down => n = n + Position::new(0, 1).unwrap(),
                Up => n = n + Position::new(0, -1).unwrap(),
                Left => n = n + Position::new(-1, 0).unwrap(),
                Right => n = n + Position::new(1, 0).unwrap(),
            }
            n
        };

        next = dir(next);
        let mut crate_type = String::new();


        {
            let obj = self.special
                .iter()
                .enumerate()
                .filter(|x| !(x.1.obj_type == ObjectType::Target))
                .find(|x| x.1.position == next);

            if let Some(ele) = obj {
                success = false;
                if ele.1.obj_type == ObjectType::Crate {
                    crate_found = ele.0 as i32;
                    crate_type = ele.1.sprite.0.clone();
                }
            }

            if success {
                self.player.position = next;
                self.score.moves += 1;
            } else {
                success = true;
                next = dir(next);
            }

            if crate_found != -1 {
                let cr = self.special.iter().enumerate().find(|x| x.1.position == next);
                if let Some(ele) = cr {
                    if ele.1.obj_type == ObjectType::Target && ele.1.sprite.0 == crate_type {
                        target_found = ele.0 as i32;
                    } else {
                        success = false;
                    }
                }
            }

        }
        if success && crate_found != -1 {


            self.special.get_mut(crate_found as usize).unwrap().position = next;

            if target_found != -1 {
                {
                    let mut crt = self.special.get_mut(crate_found as usize).unwrap();
                    crt.obj_type = ObjectType::Blocking;
                    crt.sprite = (crate_type.clone(),
                                  self.obj_tex.b_crate.get(&crate_type).unwrap());
                }
                self.score.scored += 1;
                self.targets_left -= 1;
                self.special.remove(target_found as usize);
            }

        }


    }

    fn render(&mut self, args: &RenderArgs) {
        let iter = self.floor.iter().chain(self.special.iter());
        let player = &self.player;
        let mut glyphs = GlyphCache::new("assets/FiraSans-Regular.ttf").unwrap();
        let time = SystemTime::now().duration_since(self.start_t).unwrap().as_secs();
        let score;
        let aux = 100 * self.score.scored - (self.score.moves) - time as i32;

        if aux < 0 {
            score = 0;
        } else {
            score = aux;
        }
        let t = self.targets_left;
        self.gl.draw(args.viewport(), |c, g| {

            clear([1.0, 1.0, 1.0, 1.0], g);
            for img in iter {
                let pos = &img.position;
                let transform = c.transform
                    .trans(((pos.get_x() * 64)) as f64, ((pos.get_y() * 64)) as f64);
                image(img.sprite.1, transform, g);
            }
            image(player.sprite,
                  c.transform.trans((player.position.get_x() * 64) as f64,
                                    (player.position.get_y() * 64) as f64),
                  g);

            text::Text::new_color([0., 1., 0., 1.], 64)
                .draw(&format!("Score: {:?} Time: {:?} T: {}", score, time, t),
                      &mut glyphs,
                      &c.draw_state,
                      c.transform.trans(0., 64. * 16. - 11.),
                      g);


        });
    }
    fn update(&mut self, args: &UpdateArgs) {}
}


fn main() {

    //init
    let start = SystemTime::now();
    let n_sq_w = 20;
    let n_sq_h = 15;
    let (width, height) = (64 * n_sq_w, 64 * (n_sq_h + 1));
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("piston", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();
    window.hide();
    let score = Score::new();

    let mut c_matrix = CollisionMatrix {
        coll: DMatrix::<bool>::from_element(n_sq_w as usize, n_sq_h as usize, false),
    };
    let player_tex = PlayerTextures {
        player_n: Tex::from_path("./assets/player_n.png").unwrap(),
        player_s: Tex::from_path("./assets/player_s.png").unwrap(),
        player_e: Tex::from_path("./assets/player_e.png").unwrap(),
        player_w: Tex::from_path("./assets/player_w.png").unwrap(),
    };

    let mut crate_tex = Vec::new();
    for path in fs::read_dir("./assets/crates").unwrap() {
        crate_tex.push(path.unwrap().file_name().into_string().unwrap());
    }

    let mut c_tex = HashMap::new();
    let mut b_tex = HashMap::new();
    let mut t_tex = HashMap::new();
    for tex in crate_tex {
        c_tex.insert(tex.to_owned(),
                     Tex::from_path(format!("./assets/crates/{}", tex)).unwrap());

        b_tex.insert(tex.to_owned(),
                     Tex::from_path(format!("./assets/blocked/{}", tex)).unwrap());
        t_tex.insert(tex.to_owned(),
                     Tex::from_path(format!("./assets/targets/{}", tex)).unwrap());
    }
    let obj_tex = GameTextures {
        wall: Tex::from_path("./assets/wall.png").unwrap(),
        floor: Tex::from_path("./assets/floor.png").unwrap(),
        _crate: c_tex,
        b_crate: b_tex,
        targets: t_tex,
    };

    let player = Player {
        sprite: &player_tex.player_n,
        position: Position::new(3, 3).unwrap(),
    };
    c_matrix.coll[(player.position.get_x() as usize, player.position.get_y() as usize)] = true;
    let mut game = Game {
        gl: GlGraphics::new(opengl),
        floor: Vec::new(),
        special: Vec::new(),
        player: player,
        obj_tex: &obj_tex,
        start_t: start,
        score: score,
        targets_left: -1,
    };
    let mut rand = rand::thread_rng();
    let mut crates = Vec::new();


    //walls / floors
    for i in 0..n_sq_w as i32 {
        for j in 0..n_sq_h as i32 {
            if i == 0 || i == n_sq_w as i32 - 1 || j == 0 || j == n_sq_h as i32 - 1 {
                let obj = Object {
                    sprite: ("wall".to_owned(), &obj_tex.wall),
                    obj_type: ObjectType::Blocking,
                    position: Position::new(i, j).unwrap(),
                };
                game.special.push(obj);

            } else {
                let obj = Object {
                    sprite: ("floor".to_owned(), &obj_tex.floor),
                    obj_type: ObjectType::Passing,
                    position: Position::new(i, j).unwrap(),
                };
                game.floor.push(obj);
            }
        }
    }

    let n_crates: usize = rand.gen_range(3, 16);

    //vec of crate colors (eg: red, blue, green...)
    let mut elems = Vec::new();
    for ele in obj_tex._crate.keys() {
        elems.push(ele.to_owned());
    }
    //hash map of crate numbers
    let mut crate_numbers = HashMap::new();
    let mut crate_n = crate_numbers.clone();
    'l: loop {
        for elem in &elems {
            {
                let entry = crate_numbers.entry(elem.clone()).or_insert(0);
                *entry += rand.gen_range(0, 4);
            }
            if crate_numbers.values().sum::<usize>() >= n_crates {
                break 'l;
            }
        }
    }

    //crate loop
    'l: loop {
        'out: for i in 2..n_sq_w as i32 - 2 {
            'ins: for j in 2..n_sq_h as i32 - 2 {
                let r: f32 = rand.gen_range(0., 1.);
                if r > 0.60 && c_matrix.coll[(i as usize, j as usize)] == false {
                    if c_matrix.check_surroundings((i as usize, j as usize)) {
                        continue;
                    }

                    let tex = *rand.choose(&crate_numbers.keys().collect::<Vec<_>>()).unwrap();

                    crate_n.entry(tex.to_owned()).or_insert(0);

                    if crate_n[tex] == crate_numbers[tex] {
                        continue;
                    }

                    *crate_n.get_mut(tex).unwrap() += 1;
                    crates.push(Object {
                        sprite: (tex.to_owned(), obj_tex._crate.get(tex).unwrap()),
                        obj_type: ObjectType::Crate,
                        position: Position::new(i, j).unwrap(),
                    });


                    c_matrix.coll[(i as usize, j as usize)] = true;
                    if crate_n == crate_numbers {
                        break 'l;
                    }



                    if rand.gen() {
                        continue 'out;
                    } else {
                        continue 'ins;
                    }

                }
            }
        }
    }


    let mut _targets = HashMap::new();
    let mut targets = 0;
    let crates_ = crates.len();
    let mut target = Vec::with_capacity(crates_);
    'l: loop {
        for i in 1..n_sq_w as i32 - 2 {
            for j in 1..n_sq_h as i32 - 2 {
                let r: f32 = rand.gen_range(0., 1.);
                if r > 0.90 && c_matrix.coll[(i as usize, j as usize)] == false {
                    if c_matrix.check_surroundings((i as usize, j as usize)) {
                        continue;
                    }

                    let tex = *rand.choose(&crate_numbers.keys().collect::<Vec<_>>()).unwrap();

                    _targets.entry(tex.to_owned()).or_insert(0);

                    if _targets[tex] == crate_numbers[tex] {
                        continue;
                    }

                    *_targets.get_mut(tex).unwrap() += 1;



                    target.push(Object {
                        sprite: (tex.to_owned(), obj_tex.targets.get(tex).unwrap()),
                        obj_type: ObjectType::Target,
                        position: Position::new(i, j).unwrap(),
                    });
                    targets += 1;
                    c_matrix.coll[(i as usize, j as usize)] = true;
                    if _targets == crate_numbers {
                        break 'l;
                    }
                }
            }
        }
    }
    println!("final result: crates: {:?}\n targets: {:?}\n",
             crate_n,
             _targets);
    target.append(&mut crates);
    game.special.append(&mut target);
    game.targets_left = targets as i32;
    window.show();
    while let Some(e) = window.next() {

        if let Some(Button::Keyboard(key)) = e.press_args() {
            let pos = game.player.position.clone();
            let mut spr = game.player.sprite;

            match key {
                Key::Up => {
                    game.check(&pos, Direction::Up);
                    spr = &player_tex.player_n;
                }
                Key::Down => {
                    game.check(&pos, Direction::Down);
                    spr = &player_tex.player_s;
                }
                Key::Left => {
                    game.check(&pos, Direction::Left);
                    spr = &player_tex.player_w;
                }
                Key::Right => {
                    game.check(&pos, Direction::Right);
                    spr = &player_tex.player_e;
                }
                _ => (),
            }
            game.player.sprite = spr;
        }

        if let Some(r) = e.render_args() {
            game.render(&r);
        }
        if let Some(u) = e.update_args() {
            game.update(&u);
        }
    }


}
