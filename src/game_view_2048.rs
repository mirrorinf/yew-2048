use yew::{events::KeyboardEvent, html, Component, Context, Html};
use std::ops::{Index, IndexMut};

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js="export function set_focus() {document.getElementById(\"gameplay\").focus();}")]
extern "C" {
    fn set_focus();
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up, Down, Left, Right
}

#[derive(Debug, Clone, Copy)]
struct Position {
    row: u8,
    column: u8,
}

pub struct GameState {
    state: [u64; 36],
    is_dead: bool,
    won: bool,
}

struct LineIteration {
    head: Position,
    direction: Direction,
    ended: bool,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        } 
    }

    fn perpendicular_positive(&self) -> Self {
        match self {
            Direction::Up => Direction::Left,
            Direction::Left => Direction::Down,
            Direction::Down => Direction::Right,
            Direction::Right => Direction::Up,
        }
    }
}

impl Position {
    fn position(self) -> usize {
        (6 * self.row + self.column) as usize
    }
}

impl Index<Position> for GameState {
    type Output = u64;

    fn index(&self, i: Position) -> &u64 {
        if i.row > 5 || i.column > 5 {
            panic!("Index out of bound!");
        }
        &self.state[i.position()]
    }
}

impl IndexMut<Position> for GameState {
    fn index_mut(&mut self, i: Position) -> &mut u64 {
        if i.row > 5 || i.column > 5 {
            panic!("Index out of bound!");
        }

        &mut self.state[i.position()]
    }
}

impl Position {
    fn neibouring_cell(self, pointing: Direction) -> Option<Position> {
        match pointing {
            Direction::Up => if self.row == 0 { None } else { Some(Position{row: self.row - 1, column: self.column}) },
            Direction::Down => if self.row == 5 { None } else { Some(Position{row: self.row + 1, column: self.column}) },
            Direction::Left => if self.column == 0 { None } else { Some(Position{row: self.row, column: self.column - 1}) },
            Direction::Right => if self.column == 5 { None } else { Some(Position{row: self.row, column: self.column + 1}) },
        }
    }

    fn from_index(index: u64) -> Self {
        Self {
            row: (index / 6) as u8,
            column: (index % 6) as u8,
        }
    }
}

impl Iterator for LineIteration {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        if self.ended {
            return None;
        }

        let temp = self.head;
        if let Some(next) = self.head.neibouring_cell(self.direction) {
            self.head = next;
        } else {
            self.ended = true;
        }

        Some(temp)
    }
}

impl LineIteration {
    fn heads(direction: Direction) -> Self {
        let start = match direction {
            Direction::Up => Position{row: 0, column: 5},
            Direction::Down => Position{row: 5, column: 0},
            Direction::Left => Position{row: 0, column: 0},
            Direction::Right => Position{row: 5, column: 5},
        };

        Self {head: start, direction: direction.perpendicular_positive(), ended: false}
    }
}

impl GameState {
    fn cell(&self, x: Position) -> String {
        let order = self[x];

        if order == 0 {
            "".to_string()
        } else {
            format!("{}", order)
        }
    }

    fn dead(&self) -> bool {
        for i in 0..36 {
            let p = Position::from_index(i);
            if self[p] == 0 {
                return false;
            }
            if let Some(j) = p.neibouring_cell(Direction::Up) {
                if self.mergeable(p, j) {
                    return false
                }
            }
            if let Some(j) = p.neibouring_cell(Direction::Down) {
                if self.mergeable(p, j) {
                    return false
                }
            }
            if let Some(j) = p.neibouring_cell(Direction::Left) {
                if self.mergeable(p, j) {
                    return false
                }
            }
            if let Some(j) = p.neibouring_cell(Direction::Right) {
                if self.mergeable(p, j) {
                    return false
                }
            }
        }

        true
    }

    fn wins(&self) -> bool {
        for i in 0..36 {
            if self.state[i] >= 2048 {
                return true
            }
        }

        false
    }

    fn shitword(&self) -> &'static str {
        if self.won {
            return "你nb。然鹅想重新开始？并没有实现呢，刷新吧。"
        }
        if self.is_dead {
            return "你寄了。想重新开始？然鹅并没有实现呢，刷新吧。";
        }

        "按E/S/D/F操作晓得的不咯？"
    }

    fn add_at_random_position(&mut self) {
        let empties: Vec<usize> = self.state.iter().enumerate().map(|s| {if *s.1 == 0u64 {Some(s.0)} else {None}}).filter(|s| {s.is_some()}).map(|s| {s.unwrap()}).collect();

        if empties.len() == 0 {
            return;
        }

        let mut buffer = [0u8; 1];
        getrandom::getrandom(&mut buffer).unwrap();
        let number = buffer[0] as usize % empties.len();
        self.state[empties[number]] = 1;
    }

    fn mergeable(&self, x: Position, y: Position) -> bool {
        (self[x] != 0) && (self[y] != 0) && (self[x] == self[y])
    }

    fn aggregate(&mut self, head: Position, direction: Direction) {
        let mut write = head;
        let mut count = 0;

        let elements = LineIteration {head, direction: direction.opposite(), ended: false};
        for p in elements {
            if self[p] == 0 {
                continue;
            }
            if count == 0 {
                self[write] = self[p];
                count = 1;
                continue;
            }
            if count == 1 {
                if self.mergeable(write, p) {
                    self[write] += self[p];
                    write = write.neibouring_cell(direction.opposite()).unwrap();
                    count = 0;
                } else {
                    write = write.neibouring_cell(direction.opposite()).unwrap();
                    self[write] = self[p];
                    count = 1;
                }
            }
        }

        let remaining = if count == 0 {
            LineIteration {head: write, direction: direction.opposite(), ended: false}
        } else {
            if let Some(next) = write.neibouring_cell(direction.opposite()) {
                LineIteration {head: next, direction: direction.opposite(), ended: false}
            } else {
                LineIteration {head: write, direction: direction.opposite(), ended: true}
            }
        };
        for p in remaining {
            self[p] = 0;
        }
    }

    fn update_state(&mut self, direction: Direction) {
        let heads = LineIteration::heads(direction);
        for head in heads {
            self.aggregate(head, direction);
        }

        if self.wins() {
            self.won = true;
            return;
        }

        self.add_at_random_position();

        if self.dead() {
            self.is_dead = true;
            return;
        }
    }
}

impl Component for GameState {
    type Message = Direction;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let mut obj = Self {
            state: [0; 36],
            is_dead: false,
            won: false,
        };
        obj.add_at_random_position();
        log::info!("Created obj");
        obj
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let onkeypress = link.batch_callback(|event: KeyboardEvent| {
            match event.key().as_str() {
                "E" | "e" => Some(Direction::Up),
                "S" | "s" => Some(Direction::Left),
                "D" | "d" => Some(Direction::Down),
                "F" | "f" => Some(Direction::Right),
                _ => None,
            }
        });
        
        html! {
            <div tabindex="-1" id="gameplay" {onkeypress}>
            <table>
            { (0..6).map(|row| {
                html! {
                    <tr>
                    { (0..6).map(|column| {
                        html! {
                            <td class={format!("cell-{}", self[Position{row, column}])}>{ self.cell(Position{row, column}) }</td>
                        }
                    }).collect::<Html>() }
                    </tr>
                }
            }).collect::<Html>() }
            </table>
            <p>{ self.shitword() }</p>
            </div>
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("Event: {:?}", msg);
        if !self.is_dead && !self.won {
            self.update_state(msg);
            true
        } else {
            false
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        set_focus();
    }
}
