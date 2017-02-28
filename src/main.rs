extern crate piston_window;
extern crate opengl_graphics;
extern crate rand;
extern crate nalgebra as na;

use na::core::DMatrix;
use rand::Rng;
use std::ops::Deref;
use opengl_graphics::Texture as Tex;
use piston_window::{RenderArgs, UpdateArgs, OpenGL, PistonWindow, WindowSettings, Button, Key,
                    Transformed, image, clear, text, AdvancedWindow, RenderEvent, PressEvent,
                    UpdateEvent};
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use std::time::*;
use structs::*;

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
    sprite: &'a Tex,
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
    _crate: Tex,
    b_crate: Tex,
    target: Tex,
}
impl Score {
    pub fn new() -> Score {
        Score {
            moves: 0,
            scored: 0,
        }
    }
}
impl<'a> Drop for Game<'a> {
    fn drop(&mut self) {
        for e in self.floor.iter() {
            drop(e.sprite);
            drop(e);
        }
        for e in self.special.iter() {
            drop(e.sprite);
            drop(e);
        }
        drop(self.player.sprite);
        drop(self.obj_tex);
    }
}
impl<'a> Game<'a> {
    fn check(&mut self, position: &Position, direction: Direction) {
        
        use Direction::*;

        let mut next = (*position).clone();
        let mut crate_found = -1;
        let mut target_found = -1;
        let mut success = true;
        let mut dir = |x| {
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
                }
            }
        }
        
        {
            if success {
                self.player.position = next;
                self.score.moves += 1;
            } else {
                success = true;
                next = dir(next); 

                if crate_found != -1 {
                    let cr = self.special.iter().enumerate().find(|x| x.1.position == next);
                    if let Some(ele) = cr {
                        if ele.1.obj_type == ObjectType::Target {
                            target_found = ele.0 as i32;
                        } else {
                            success = false;
                        }
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
                    crt.sprite = &self.obj_tex.b_crate;
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
                image(img.sprite, transform, g);
            }
            image(player.sprite,
                  c.transform.trans((player.position.get_x() * 64) as f64,
                                    (player.position.get_y() * 64) as f64),
                  g);

            text::Text::new_color([0., 1., 0., 1.], 64)
                .draw(&format!("Score: {:?} Time: {:?} T: {}", score, time, t),
                      &mut glyphs,
                      &c.draw_state,
                      c.transform.trans(0., 704. - 11.),
                      g);


        });
    }
    fn update(&mut self, args: &UpdateArgs) {}
}


fn main() {

    let (width, height) = (64 * 10, 64 * 11);
    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new("piston", (width, height))
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();
    window.hide();
    println!("Loading...");
    let score = Score::new();
    let start = SystemTime::now();
    let mut c_matrix = CollisionMatrix { coll: DMatrix::<bool>::from_element(10, 10, false) };
    let player_tex = PlayerTextures {
        player_n: Tex::from_path("./assets/player_n.png").unwrap(),
        player_s: Tex::from_path("./assets/player_s.png").unwrap(),
        player_e: Tex::from_path("./assets/player_e.png").unwrap(),
        player_w: Tex::from_path("./assets/player_w.png").unwrap(),
    };
    let obj_tex = GameTextures {
        wall: Tex::from_path("./assets/wall.png").unwrap(),
        floor: Tex::from_path("./assets/floor.png").unwrap(),
        _crate: Tex::from_path("./assets/crates/crate_19.png").unwrap(),
        b_crate: Tex::from_path("./assets/crates/crate_11.png").unwrap(),
        target: Tex::from_path("./assets/crates/crate_29.png").unwrap(),
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

    for i in 0..10 {
        for j in 0..10 {
            if i == 0 || i == 9 || j == 0 || j == 9 {
                let obj = Object {
                    sprite: &obj_tex.wall,
                    obj_type: ObjectType::Blocking,
                    position: Position::new(i, j).unwrap(),
                };
                game.special.push(obj);

            } else {
                let r: f32 = rand.gen_range(0., 1.);
                let obj = Object {
                    sprite: &obj_tex.floor,
                    obj_type: ObjectType::Passing,
                    position: Position::new(i, j).unwrap(),
                };
                game.floor.push(obj);

                if i > 1 && i < 8 && j > 1 && j < 8 && r <= 0.12 &&
                   c_matrix.coll[(i as usize, j as usize)] == false {
                    crates.push(Object {
                        sprite: &obj_tex._crate,
                        obj_type: ObjectType::Crate,
                        position: Position::new(i, j).unwrap(),
                    });
                    c_matrix.coll[(i as usize, j as usize)] = true;
                }
            }
        }
    }

    let mut targets = 0;
    let crates_ = crates.len();
    let mut target = Vec::with_capacity(crates_);
    for i in 1..9 {
        for j in 1..9 {
            if !(i == 0 || i == 9 || j == 0 || j == 9) {
                let r: f32 = rand.gen_range(0., 1.);
                if targets < crates_ && r > 0.66 &&
                   c_matrix.coll[(i as usize, j as usize)] == false {
                    target.push(Object {
                        sprite: &obj_tex.target,
                        obj_type: ObjectType::Target,
                        position: Position::new(i, j).unwrap(),
                    });
                    targets += 1;
                    c_matrix.coll[(i as usize, j as usize)] = true;
                }


            }
        }
    }
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
