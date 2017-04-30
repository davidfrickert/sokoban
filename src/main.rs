extern crate piston_window;
extern crate opengl_graphics;
extern crate rand;
extern crate nalgebra as na;
extern crate sdl2_window;

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::mpsc;
use std::thread;
use std::collections::HashMap;


use na::core::DMatrix;
use rand::Rng;
use opengl_graphics::Texture as Tex;
use piston_window::{RenderArgs, UpdateArgs, OpenGL, PistonWindow, WindowSettings, Button, Key,
                    Transformed, image, clear, text, AdvancedWindow, RenderEvent, ReleaseEvent,
                    PressEvent, UpdateEvent};
use sdl2_window::Sdl2Window;
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
struct Player {
    sprite: Option<Arc<RwLock<Tex>>>,
    position: Position,
    canMove: bool,
}
struct Object {
    sprite: Option<(String, Arc<RwLock<Tex>>)>,
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
struct Game {
    gl: Option<GlGraphics>,
    floor: Vec<Object>,
    special: Vec<Object>,
    player: Player,
    obj_tex: Option<GameTextures>,
    player_tex: Option<PlayerTextures>,
    start_t: SystemTime,
    score: Score,
    targets_left: i32,
    size: (u32, u32),
}
struct PlayerTextures {
    player_n: Arc<RwLock<Tex>>,
    player_s: Arc<RwLock<Tex>>,
    player_e: Arc<RwLock<Tex>>,
    player_w: Arc<RwLock<Tex>>,
}
struct GameTextures {
    wall: Arc<RwLock<Tex>>,
    floor: Arc<RwLock<Tex>>,
    _crate: HashMap<String, Arc<RwLock<Tex>>>,
    b_crate: HashMap<String, Arc<RwLock<Tex>>>,
    targets: HashMap<String, Arc<RwLock<Tex>>>,
}
impl Object {
    fn new(position: Position,
           obj_type: ObjectType,
           sprite: Option<(String, Arc<RwLock<Tex>>)>)
           -> Object {
        Object {
            position: position,
            obj_type: obj_type,
            sprite: sprite,
        }
    }
}

impl CollisionMatrix {
    pub fn check_surroundings(&self, xy: (usize, usize), times: i32) -> bool {
        let mut is_invalid = false;

        for i in -times..times + 1 {
            for j in -times..times + 1 {
                is_invalid = is_invalid || self.next(xy, i, j);
                //println!("next of {:?} in {:?} is {}",
                //         xy,
                //         (i, j),
                //         self.next(xy, i, j));
            }
        }
        //std::thread::sleep(std::time::Duration::from_millis(400));
        is_invalid
    }
    pub fn next(&self, ind: (usize, usize), x: i32, y: i32) -> bool {
        if ind.0 as i32 + y < 0 || ind.1 as i32 + x < 0 || ind.0 as i32 + y > 9 ||
           ind.1 as i32 + x > 14 {
            false
        } else {
            self.coll[((ind.0 as i32 + y) as usize, (ind.1 as i32 + x) as usize)]
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

impl Game {
    fn new(size: (usize, usize)) -> Game {
        let score = Score::new();
        let start = SystemTime::now();
        let mut c_matrix =
            CollisionMatrix { coll: DMatrix::<bool>::from_element(size.0, size.1, false) };
        let player_tex = PlayerTextures {
            player_n: Arc::new(RwLock::new(Tex::from_path("./assets/player_n.png").unwrap())),
            player_s: Arc::new(RwLock::new(Tex::from_path("./assets/player_s.png").unwrap())),
            player_e: Arc::new(RwLock::new(Tex::from_path("./assets/player_e.png").unwrap())),
            player_w: Arc::new(RwLock::new(Tex::from_path("./assets/player_w.png").unwrap())),
        };

        let mut crate_tex = Vec::new();
        for path in fs::read_dir("./assets/crates").unwrap() {
            crate_tex.push(path.unwrap()
                               .file_name()
                               .into_string()
                               .unwrap());
        }

        let mut c_tex = HashMap::new();
        let mut b_tex = HashMap::new();
        let mut t_tex = HashMap::new();
        for tex in crate_tex {
            c_tex.insert(tex.to_owned(),
                         Arc::new(RwLock::new(Tex::from_path(format!("./assets/crates/{}", tex))
                                                  .unwrap())));

            b_tex.insert(tex.to_owned(),
                         Arc::new(RwLock::new(Tex::from_path(format!("./assets/blocked/{}",
                                                                     tex))
                                                      .unwrap())));
            t_tex.insert(tex.to_owned(),
                         Arc::new(RwLock::new(Tex::from_path(format!("./assets/targets/{}",
                                                                     tex))
                                                      .unwrap())));
        }
        let obj_tex = GameTextures {
            wall: Arc::new(RwLock::new(Tex::from_path("./assets/wall.png").unwrap())),
            floor: Arc::new(RwLock::new(Tex::from_path("./assets/floor.png").unwrap())),
            _crate: c_tex,
            b_crate: b_tex,
            targets: t_tex,
        };

        let player_tex = PlayerTextures {
            player_n: Arc::new(RwLock::new(Tex::from_path("./assets/player_n.png").unwrap())),
            player_s: Arc::new(RwLock::new(Tex::from_path("./assets/player_s.png").unwrap())),
            player_e: Arc::new(RwLock::new(Tex::from_path("./assets/player_e.png").unwrap())),
            player_w: Arc::new(RwLock::new(Tex::from_path("./assets/player_w.png").unwrap())),
        };
        let aux_sprite = Some(player_tex.player_n.clone());
        let player = Player {
            sprite: aux_sprite,
            position: Position::new(1, 3).unwrap(),
            canMove: false,
        };
        c_matrix.coll[(player.position.get_y() as usize, player.position.get_x() as usize)] = true;
        Game {
            gl: Some(GlGraphics::new(OpenGL::V3_2)),
            floor: Vec::new(),
            special: Vec::new(),
            player: player,
            obj_tex: Some(obj_tex),
            player_tex: Some(player_tex),
            start_t: start,
            score: score,
            targets_left: -1,
            size: (size.0 as u32, size.1 as u32),
        }
    }
    fn move_player(&mut self, key: Key) {

        match key {
            Key::Up => {
                let pos = &self.player.position.clone();
                self.check(&pos, Direction::Up);


                if let Some(ref spr) = self.player_tex {
                    self.player.sprite = Some(spr.player_n.clone());
                }
            }
            Key::Down => {
                let pos = &self.player.position.clone();
                self.check(&pos, Direction::Down);
                if let Some(ref spr) = self.player_tex {
                    self.player.sprite = Some(spr.player_s.clone());
                }

            }
            Key::Left => {
                let pos = &self.player.position.clone();
                self.check(&pos, Direction::Left);
                if let Some(ref spr) = self.player_tex {
                    self.player.sprite = Some(spr.player_w.clone());
                }


            }
            Key::Right => {
                let pos = &self.player.position.clone();

                self.check(&pos, Direction::Right);
                if let Some(ref spr) = self.player_tex {
                    self.player.sprite = Some(spr.player_e.clone());
                }

            }
            _ => (),
        }

    }
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
                    if let Some(ref spr) = ele.1.sprite {
                        crate_type = spr.0.clone();
                    }
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
                let cr = self.special
                    .iter()
                    .enumerate()
                    .find(|x| x.1.position == next);
                if let Some(ele) = cr {
                    if ele.1.obj_type == ObjectType::Target &&
                       ele.1
                           .sprite
                           .as_ref()
                           .unwrap()
                           .0 == crate_type {
                        target_found = ele.0 as i32;
                    } else {
                        success = false;
                    }
                }
            }

        }
        if success && crate_found != -1 {


            self.special
                .get_mut(crate_found as usize)
                .unwrap()
                .position = next;

            if target_found != -1 {
                {
                    let mut crt = self.special.get_mut(crate_found as usize).unwrap();
                    crt.obj_type = ObjectType::Blocking;

                    if let Some(ref spr) = self.obj_tex {
                        crt.sprite = Some((crate_type.clone(),

                                           spr.b_crate
                                               .get(&crate_type)
                                               .unwrap()
                                               .clone()));

                    }
                }
                self.score.scored += 1;
                self.targets_left -= 1;
                self.special.remove(target_found as usize);
                if self.targets_left == 0 {
                    println!("END");
                    self.gen_level();
                }
            }

        }


    }

    fn render(&mut self, args: &RenderArgs) {
        if let Some(ref mut gl) = self.gl {
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

            gl.draw(args.viewport(), |c, g| {

                clear([1.0, 1.0, 1.0, 1.0], g);
                for img in iter {
                    let pos = &img.position;
                    let transform = c.transform.trans(((pos.get_x() * 64)) as f64,
                                                      ((pos.get_y() * 64)) as f64);
                    if let Some(ref spr) = img.sprite {
                        image(&(*spr.1.read().unwrap()), transform, g);
                    }
                }
                if let Some(ref spr) = player.sprite {
                    image(&(*(spr.read().unwrap())),
                          c.transform.trans((player.position.get_x() * 64) as f64,
                                            (player.position.get_y() * 64) as f64),
                          g);
                }
                text::Text::new_color([0., 1., 0., 1.], 64)
                .draw(&format!("Score: {:?} Time: {:?} T: {}", score, time, t),
                      &mut glyphs,
                      &c.draw_state,
                      c.transform.trans(0., 64. * 11. - 11.),
                      g);


            });
        }
    }
    fn update(&mut self, args: &UpdateArgs) {}
    fn gen_level(&mut self) {
        self.special.clear();
        self.floor.clear();
        let mut c_matrix = CollisionMatrix {
            coll: DMatrix::<bool>::from_element(self.size.1 as usize, self.size.0 as usize, false),
        };
        let mut rand = rand::thread_rng();
        let mut crates = Vec::new();
        let mut obj;
        //walls / floors
        for i in 0..self.size.0 as i32 {
            for j in 0..self.size.1 as i32 {
                if i == 0 || i == self.size.0 as i32 - 1 || j == 0 || j == self.size.1 as i32 - 1 {
                    if let Some(ref g) = self.gl {
                        obj = Object::new(Position::new(i, j).unwrap(),
                                          ObjectType::Blocking,
                                          Some(("wall".to_string(),
                                                self.obj_tex
                                                    .as_ref()
                                                    .unwrap()
                                                    .wall
                                                    .clone())));
                    } else {
                        obj = Object::new(Position::new(i, j).unwrap(), ObjectType::Blocking, None);
                    }
                    self.special.push(obj);

                } else {
                    if let Some(ref g) = self.gl {
                        obj = Object::new(Position::new(i, j).unwrap(),
                                          ObjectType::Passing,
                                          Some(("floor".to_string(),
                                                self.obj_tex
                                                    .as_ref()
                                                    .unwrap()
                                                    .floor
                                                    .clone())));
                    } else {
                        obj = Object::new(Position::new(i, j).unwrap(), ObjectType::Passing, None);
                    }

                    self.floor.push(obj);
                }
            }
        }




        let n_crates: usize = rand.gen_range(3, 9);

        //vec of crate colors (eg: red, blue, green...)
        let mut elems = Vec::new();
        if let Some(ref obj) = self.obj_tex {
            let key_iter = obj._crate.keys();
            for ele in key_iter {
                elems.push(ele.to_owned());
            }
        }
        //hash map of crate numbers
        let mut crate_numbers = HashMap::new();
        let mut crate_n = crate_numbers.clone();
        'l: loop {
            for elem in elems.iter() {
                {
                    let entry = crate_numbers.entry(elem.clone()).or_insert(0);
                    *entry += rand.gen_range(0, 2);
                }
                if crate_numbers.values().sum::<usize>() >= n_crates {
                    break 'l;
                }
            }
        }
        crate_numbers = crate_numbers.into_iter().filter(|&(_, v)| v != 0).collect();

        let mut loop_fails = 0;
        //crate loop
        'l: loop {
            'out: for i in 2..self.size.0 as i32 - 2 {
                'ins: for j in 2..self.size.1 as i32 - 2 {
                    let r: f32 = rand.gen_range(0., 1.);
                    if r > 0.60 && c_matrix.coll[(j as usize, i as usize)] == false {

                        let mut dist = 3 - loop_fails / 30;
                        if dist < 0 {
                            dist = 0;
                        }

                        if c_matrix.check_surroundings((j as usize, i as usize), dist) {
                            loop_fails += 1;

                            continue;
                        }
                        loop_fails = 0;
                        let tex = *rand.choose(&crate_numbers.keys().collect::<Vec<_>>()).unwrap();

                        crate_n.entry(tex.to_owned()).or_insert(0);

                        if crate_n[tex] == crate_numbers[tex] {
                            continue;
                        }
                        *crate_n.get_mut(tex).unwrap() += 1;

                        if let Some(ref g) = self.gl {
                            crates.push(Object::new(Position::new(i, j).unwrap(),
                                                    ObjectType::Crate,
                                                    Some((tex.to_string(),
                                                          self.obj_tex
                                                              .as_ref()
                                                              .unwrap()
                                                              ._crate
                                                              .get(tex)
                                                              .unwrap()
                                                              .clone()))));
                        } else {
                            crates.push(Object::new(Position::new(i, j).unwrap(),
                                                    ObjectType::Crate,
                                                    None));
                        }
                        c_matrix.coll[(j as usize, i as usize)] = true;
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

            for i in 1..self.size.0 as i32 - 2 {
                for j in 1..self.size.1 as i32 - 2 {
                    let r: f32 = rand.gen_range(0., 1.);
                    if r > 0.90 && c_matrix.coll[(j as usize, i as usize)] == false {

                        let mut dist = 3 - loop_fails / 30;
                        if dist < 0 {
                            dist = 0;
                        }
                        if c_matrix.check_surroundings((j as usize, i as usize), dist) {
                            loop_fails += 1;

                            continue;
                        }

                        loop_fails = 0;
                        let tex = *rand.choose(&crate_numbers.keys().collect::<Vec<_>>()).unwrap();

                        _targets.entry(tex.to_owned()).or_insert(0);
                        if _targets[tex] == crate_numbers[tex] {
                            continue;
                        }


                        *_targets.get_mut(tex).unwrap() += 1;



                        if let Some(ref g) = self.gl {
                            crates.push(Object::new(Position::new(i, j).unwrap(),
                                                    ObjectType::Target,
                                                    Some((tex.to_string(),
                                                          self.obj_tex
                                                              .as_ref()
                                                              .unwrap()
                                                              .targets
                                                              .get(tex)
                                                              .unwrap()
                                                              .clone()))));
                        } else {
                            crates.push(Object::new(Position::new(i, j).unwrap(),
                                                    ObjectType::Target,
                                                    None));
                        }

                        targets += 1;
                        c_matrix.coll[(j as usize, i as usize)] = true;
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
        self.special.append(&mut target);
        self.targets_left = targets as i32;
        for i in 0..c_matrix.coll.nrows() {
            println!("{}: ", i);
            for j in 0..c_matrix.coll.ncols() {
                print!("[{}]", c_matrix.coll[(i as usize, j as usize)]);
            }
            println!("\n");
        }

    }
}

fn main() {
    let size = (15, 10);
    let mut window: PistonWindow<Sdl2Window> = WindowSettings::new("sokoban", (15*64, 10*74))
        .exit_on_esc(true)
        //.opengl(OpenGL::V3_2)
        .resizable(true)
        .build()
        .unwrap();
    window.hide();
    let mut game = Game::new((size.0 as usize, size.1 as usize));



    let bef_gen = SystemTime::now();
    game.gen_level();
    let time = SystemTime::now().duration_since(bef_gen).unwrap().subsec_nanos() as f64 /
               1_000_000_000.;
    println!("time generating map: {:?}", time);
    window.show();
    let mut arc_game = Arc::new(RwLock::new(game));
    let mut game_ = arc_game.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
                      let key = rx.recv().unwrap();
                      game_.write().unwrap().move_player(key);
                  });
    let mut pressed = false;
    let mut k = piston_window::Key::Unknown;
    let mut threads = Arc::new(RwLock::new(Vec::new()));

    while let Some(e) = window.next() {
        if let Some(Button::Keyboard(key)) = e.press_args() {
            k = key;
            pressed = true;
        }
        if let Some(Button::Keyboard(key)) = e.release_args() {
            pressed = false;
        }


        let thrd = tx.clone();
        let pr = Arc::new(pressed);
        let prs = pr.clone();
        println!("{:?}", prs);
        let tc = threads.clone();
        let t = thread::spawn(move || if tc.write().unwrap().len() == 0 {
                                  let c = thread::current();

                                  tc.write().unwrap().push(c);

                                  while *prs && tc.write().unwrap().len() == 1 {
                                      println!("{:?}", prs);
                                      thread::sleep_ms(50);
                                      thrd.send(k);
                                  }
                                  tc.write().unwrap().pop();
                              });
        if let Some(r) = e.render_args() {
            arc_game.write().unwrap().render(&r);
        }
        if let Some(u) = e.update_args() {
            arc_game.write().unwrap().update(&u);
        }
    }
    drop(window);

}
