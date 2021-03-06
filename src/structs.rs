use std::ops::Add;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Position {
    x: i32,
    y: i32,
}

impl Add for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Option<Position> {
        if x > -2 && y > -2 && x < 50 && y < 50 {
            Some(Position { x: x, y: y })
        } else {
            println!("barraca");
            None
        }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn add_x(&mut self, x: i32) {
        self.x += x;
    }

    pub fn add_y(&mut self, y: i32) {
        self.y += y;
    }
}
