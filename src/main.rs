use ncurses::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::{thread, time};

#[derive(Copy, Clone)]
enum Direction {
    N,
    E,
    S,
    W,
}

#[derive(Copy, Clone)]
enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

#[derive(Copy, Clone)]
enum Square {
    FULL(Shape),
    EMPTY,
}

#[derive(Clone)]
struct Player {
    data: Vec<Square>,
    shape: Shape,
    direction: Direction,
    y: isize,
    x: isize,
}

struct Board {
    window: WINDOW,
    next_window: WINDOW,
    held_window: WINDOW,
    lines_window: WINDOW,
    level_window: WINDOW,
    data: Vec<Square>,
    player: Player,
    held: Option<Shape>,
    grab_bag: Vec<Player>,
    lines: usize,
}

impl Board {
    fn new(window: WINDOW, next_window:WINDOW, held_window:WINDOW, lines_window: WINDOW, level_window: WINDOW) -> Board {
        let mut data: Vec<Square> = Vec::new();
        for _ in 0..400 {
            data.push(Square::EMPTY);
        }

        let mut grab_bag = Board::make_grab_bag();
        let player = grab_bag.pop().unwrap();

        Board {
            window,
            next_window,
            held_window,
            lines_window,
            level_window,
            data,
            player,
            held: None,
            grab_bag,
            lines: 0,
        }
    }

    fn get(&self, y: usize, x: usize) -> &Square {
        &self.data[y * 10 + x]
    }

    fn get_mut(&mut self, y: usize, x: usize) -> &mut Square {
        &mut self.data[y * 10 + x]
    }

    fn gravity(&self) -> usize {
        let level = self.lines / 10;
        ((0.8 - ((level as f64 - 1.0) * 0.007)).powi(level as i32 - 1) * 100.0) as usize
    }

    fn draw_score(&self) {
        mvwprintw(self.lines_window, 1, 1, &format!("{:>3}", self.lines));
        wrefresh(self.lines_window);

        mvwprintw(self.level_window, 1, 1, &format!("{:>3}", self.lines / 10));
        wrefresh(self.level_window);
    }

    fn draw_next(&mut self){
        let next = self.grab_bag.pop().unwrap();

        let color = match next.shape {
            Shape::I => COLOR_PAIR(1),
            Shape::O => COLOR_PAIR(2),
            Shape::T => COLOR_PAIR(3),
            Shape::S => COLOR_PAIR(4),
            Shape::Z => COLOR_PAIR(5),
            Shape::J => COLOR_PAIR(6),
            Shape::L => COLOR_PAIR(7),
        };

        wattron(self.next_window, color);
        for y in 0..2{
            wmove(self.next_window, (1 + y) as i32, 1 as i32);
            for x in 0..4{
                match next.data[y * 4 + x]{
                    Square::FULL(_) => {waddch(self.next_window, '@' as chtype);}
                    Square::EMPTY => {waddch(self.next_window, ' ' as chtype);}
                }
            }
        }
        wattroff(self.next_window, color);

        wrefresh(self.next_window);

        self.grab_bag.push(next);

    }

    fn draw_held(&mut self){
        let held = Board::make_piece(self.held.unwrap());

        let color = match held.shape {
            Shape::I => COLOR_PAIR(1),
            Shape::O => COLOR_PAIR(2),
            Shape::T => COLOR_PAIR(3),
            Shape::S => COLOR_PAIR(4),
            Shape::Z => COLOR_PAIR(5),
            Shape::J => COLOR_PAIR(6),
            Shape::L => COLOR_PAIR(7),
        };

        wattron(self.held_window, color);
        for y in 0..2{
            wmove(self.held_window, (1 + y) as i32, 1 as i32);
            for x in 0..4{
                match held.data[y * 4 + x]{
                    Square::FULL(_) => {waddch(self.held_window, '@' as chtype);}
                    Square::EMPTY => {waddch(self.held_window, ' ' as chtype);}
                }
            }
        }
        wattroff(self.held_window, color);

        wrefresh(self.held_window);
    }

    fn draw(&self) {
        for y in 20..40 {
            wmove(self.window, y - 19 as i32, 1 as i32);
            for x in 0..10 {
                if let Square::FULL(s) = self.get(y as usize, x as usize) {
                    let color = match s {
                        Shape::I => COLOR_PAIR(1),
                        Shape::O => COLOR_PAIR(2),
                        Shape::T => COLOR_PAIR(3),
                        Shape::S => COLOR_PAIR(4),
                        Shape::Z => COLOR_PAIR(5),
                        Shape::J => COLOR_PAIR(6),
                        Shape::L => COLOR_PAIR(7),
                    };
                    wattron(self.window, color);
                    waddch(self.window, '#' as chtype);
                    wattroff(self.window, color);
                } else {
                    waddch(self.window, ' ' as chtype);
                };
            }
        }

        let color = match self.player.shape {
            Shape::I => COLOR_PAIR(1),
            Shape::O => COLOR_PAIR(2),
            Shape::T => COLOR_PAIR(3),
            Shape::S => COLOR_PAIR(4),
            Shape::Z => COLOR_PAIR(5),
            Shape::J => COLOR_PAIR(6),
            Shape::L => COLOR_PAIR(7),
        };

        let ghost = self.bottom_out();

        let p_y = ghost.y;
        let p_x = ghost.x;

        wattron(self.window, color | A_STANDOUT());
        for y in 0..4 {
            for x in 0..4 {
                if 20 <= (y + p_y) && (y + p_y) <= 39 && 0 <= (x + p_x) && (x + p_x) <= 9 {
                    match ghost.data[(y * 4 + x) as usize] {
                        Square::FULL(_) => {
                            mvwaddch(
                                self.window,
                                (y + p_y - 20 + 1) as i32,
                                (x + p_x + 1) as i32,
                                '#' as chtype,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        wattroff(self.window, A_STANDOUT());

        let p_y = self.player.y;
        let p_x = self.player.x;

        for y in 0..4 {
            for x in 0..4 {
                if 20 <= (y + p_y) && (y + p_y) <= 39 && 0 <= (x + p_x) && (x + p_x) <= 9 {
                    match self.player.data[(y * 4 + x) as usize] {
                        Square::FULL(_) => {
                            mvwaddch(
                                self.window,
                                (y + p_y - 20 + 1) as i32,
                                (x + p_x + 1) as i32,
                                '#' as chtype,
                            );
                        }
                        _ => {}
                    }
                }
            }
        }
        wattroff(self.window, color);

        wrefresh(self.window);
    }

    fn make_piece(shape: Shape) -> Player {
        let mut data = Vec::new();
        let mut i = 0b1;

        let map = match shape {
            Shape::I => 0b1111_0000,
            Shape::O => 0b0110_0110,
            Shape::T => 0b0111_0010,
            Shape::S => 0b0011_0110,
            Shape::Z => 0b0110_0011,
            Shape::J => 0b0111_0001,
            Shape::L => 0b0111_0100,
        };

        for _ in 0..16 {
            if i & map != 0 {
                data.push(Square::FULL(shape));
            } else {
                data.push(Square::EMPTY);
            }
            i <<= 1;
        }

        Player {
            data,
            shape,
            direction: Direction::N,
            y: 18,
            x: 3,
        }
    }

    fn make_grab_bag() -> Vec<Player> {
        let mut bag = Vec::new();

        bag.push(Board::make_piece(Shape::I));
        bag.push(Board::make_piece(Shape::O));
        bag.push(Board::make_piece(Shape::T));
        bag.push(Board::make_piece(Shape::S));
        bag.push(Board::make_piece(Shape::Z));
        bag.push(Board::make_piece(Shape::J));
        bag.push(Board::make_piece(Shape::L));

        let mut rng = thread_rng();
        let slice: &mut [Player] = &mut bag;
        slice.shuffle(&mut rng);

        bag
    }

    fn get_new_piece(&mut self) {
        if self.grab_bag.len() == 1 {
            let mut new_bag = Board::make_grab_bag();
            new_bag.append(&mut self.grab_bag);
            self.grab_bag = new_bag;
        }
        self.player = self.grab_bag.pop().unwrap();
    }


    fn check_lines(&mut self) -> bool {
        //Returns true if game should continue
        for y in 0..20 {
            for x in 0..10 {
                if let Square::FULL(_) = self.get(y, x) {
                    return false;
                }
            }
        }

        let mut blocks = 0;
        let mut cleared_lines = 0;
        for y in (20..40).rev() {
            for x in 0..10 {
                if let Square::FULL(_) = self.get(y, x) {
                    blocks += 1;
                }
            }
            if blocks == 10 {
                for _ in 0..10 {
                    self.data.remove(10 * y);
                }
                cleared_lines += 1;
                self.lines += 1;
                blocks = 0;
            } else {
                blocks = 0;
            }
        }

        let mut new_data = Vec::new();
        for _ in 0..cleared_lines * 10 {
            new_data.push(Square::EMPTY);
        }

        new_data.append(&mut self.data);
        self.data = new_data;

        self.draw_score();

        return true;
    }

    fn collision(&self, new_player: &Player) -> bool {
        let p_y = new_player.y;
        let p_x = new_player.x;

        for y in 0..4 {
            for x in 0..4 {
                if 0 <= (y + p_y) && (y + p_y) <= 39 && 0 <= (x + p_x) && (x + p_x) <= 9 {
                    match (
                        self.get((y + p_y) as usize, (x + p_x) as usize),
                        new_player.data[(y * 4 + x) as usize],
                    ) {
                        (Square::FULL(_), Square::FULL(_)) => return true,
                        _ => {}
                    }
                } else {
                    match new_player.data[(y * 4 + x) as usize] {
                        Square::FULL(_) => return true,
                        _ => {}
                    }
                }
            }
        }
        false
    }

    fn lock(&mut self) {
        let p_y = self.player.y;
        let p_x = self.player.x;

        for y in 0..4 {
            for x in 0..4 {
                if 0 <= (y + p_y) && (y + p_y) <= 39 && 0 <= (x + p_x) && (x + p_x) <= 9 {
                    if let Square::FULL(_) = self.player.data[(y * 4 + x) as usize] {
                        *self.get_mut((p_y + y) as usize, (p_x + x) as usize) =
                            self.player.data[(y * 4 + x) as usize];
                    }
                }
            }
        }
    }

    fn move_player(&mut self, direction: Direction) -> LockResult {
        let new_player = match direction {
            Direction::N => self.player.clone(),
            Direction::E => Player {
                x: self.player.x + 1,
                ..self.player.clone()
            },
            Direction::S => Player {
                y: self.player.y + 1,
                ..self.player.clone()
            },
            Direction::W => Player {
                x: self.player.x - 1,
                ..self.player.clone()
            },
        };

        if !self.collision(&new_player) {
            self.player = new_player;
            return LockResult::Unlock;
        } else {
            if let Direction::S = direction {
                return LockResult::Lock;
            }
        }
        LockResult::NoChange
    }

    fn bottom_out(&self) -> Player {
        let mut new_player = self.player.clone();
        loop {
            if self.collision(&new_player) {
                new_player = Player {
                    y: new_player.y - 1,
                    ..new_player
                };
                break;
            } else {
                new_player = Player {
                    y: new_player.y + 1,
                    ..new_player
                }
            }
        }
        return new_player;
    }

    fn hard_drop(&mut self) -> LockResult {
        self.player = self.bottom_out();

        LockResult::Lock
    }

    fn super_rotation_system(
        &self,
        rotated_player: Player,
        direction: Direction,
    ) -> Option<Player> {
        let possible_kicks = match self.player.shape {
            Shape::O => [(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
            Shape::I => match (self.player.direction, direction) {
                (Direction::N, Direction::W) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (Direction::N, Direction::E) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Direction::E, Direction::W) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Direction::E, Direction::E) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
                (Direction::S, Direction::W) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                (Direction::S, Direction::E) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
                (Direction::W, Direction::W) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
                (Direction::W, Direction::E) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
                _ => panic!(),
            },
            _ => match (self.player.direction, direction) {
                (Direction::N, Direction::W) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (Direction::N, Direction::E) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Direction::E, Direction::W) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (Direction::E, Direction::E) => [(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
                (Direction::S, Direction::W) => [(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
                (Direction::S, Direction::E) => [(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
                (Direction::W, Direction::W) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                (Direction::W, Direction::E) => [(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
                _ => panic!(),
            },
        };

        for (x, y) in possible_kicks.iter() {
            let test = Player {
                y: rotated_player.y + y,
                x: rotated_player.x + x,
                ..rotated_player.clone()
            };
            if !self.collision(&test) {
                return Some(test);
            }
        }
        None
    }

    fn rotate_player(&mut self, direction: Direction) -> LockResult {
        let mut rotated_data = self.player.data.clone();
        let rotated_data = match direction {
            Direction::E => match self.player.shape {
                Shape::O => rotated_data,
                Shape::I => {
                    for y in 0..4 {
                        for x in 0..4 {
                            rotated_data[y * 4 + x] = self.player.data[(3 - x) * 4 + y];
                        }
                    }
                    rotated_data
                }
                _ => {
                    for y in 0..3 {
                        for x in 0..3 {
                            rotated_data[y * 4 + x] = self.player.data[(2 - x) * 4 + y];
                        }
                    }
                    rotated_data
                }
            },
            Direction::W => match self.player.shape {
                Shape::O => rotated_data,
                Shape::I => {
                    for y in 0..4 {
                        for x in 0..4 {
                            rotated_data[y * 4 + x] = self.player.data[x * 4 + (3 - y)];
                        }
                    }
                    rotated_data
                }
                _ => {
                    for y in 0..3 {
                        for x in 0..3 {
                            rotated_data[y * 4 + x] = self.player.data[x * 4 + (2 - y)];
                        }
                    }
                    rotated_data
                }
            },
            _ => rotated_data,
        };

        let new_direction = match (self.player.direction, direction) {
            (Direction::N, Direction::W) => Direction::W,
            (Direction::N, Direction::E) => Direction::E,
            (Direction::E, Direction::W) => Direction::N,
            (Direction::E, Direction::E) => Direction::S,
            (Direction::S, Direction::W) => Direction::E,
            (Direction::S, Direction::E) => Direction::W,
            (Direction::W, Direction::W) => Direction::S,
            (Direction::W, Direction::E) => Direction::N,
            _ => panic!(),
        };

        let rotated_player = Player {
            data: rotated_data,
            direction: new_direction,
            ..self.player.clone()
        };

        let rotated_player = self.super_rotation_system(rotated_player, direction);

        if let Some(p) = rotated_player {
            self.player = p;
            return LockResult::Unlock;
        } else {
            return LockResult::NoChange;
        }
    }

    fn hold(&mut self) {
        match self.held {
            Some(s) => {
                let new_player = Board::make_piece(s);
                self.held = Some(self.player.shape);
                self.player = new_player;
            }
            None => {
                self.held = Some(self.player.shape);
                self.get_new_piece();
            }
        }
    }
}

enum LockResult {
    Lock,
    Unlock,
    NoChange,
}

fn main() {
    initscr();
    cbreak();
    noecho();
    keypad(stdscr(), true);
    nodelay(stdscr(), true);
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    start_color();
    initialize_color();

    refresh();

    let mut max_y = 0;
    let mut max_x = 0;

    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let y = (max_y - 22) / 2;
    let x = (max_x - 12) / 2;

    let window = newwin(22, 12, y, x);
    box_(window, 0, 0);

    mvprintw(y - 2, x + 3, "tetris");
    refresh();

    mvprintw(y + 2, x + 14, "LINES");
    let lines_window = newwin(3, 5, y + 3, x + 14);
    box_(lines_window, 0, 0);
    wrefresh(lines_window);

    mvprintw(y + 7, x + 14, "LEVEL");
    let level_window = newwin(3, 5, y + 8, x + 14);
    box_(level_window, 0, 0);
    wrefresh(level_window);

    mvprintw(y + 1 , x - 8, " NEXT");
    let next_window = newwin(4, 6, y + 2, x - 8);
    box_(next_window, 0, 0);
    wrefresh(next_window);

    mvprintw(y + 8, x - 8, " HELD");
    let held_window = newwin(4, 6, y + 9, x - 8);
    box_(held_window, 0, 0);
    wrefresh(held_window);


    let mut board = Board::new(window, next_window, held_window, lines_window, level_window);

    board.draw();
    board.draw_next();

    let ten_millis = time::Duration::from_millis(10);
    let mut time = 0;
    let mut lock_time = 0;
    let mut lock = false;
    let mut can_hold = true;

    loop {
        let mut set_lock = match getch() {
            48|99 => {
                if can_hold {
                    board.hold();
                    board.draw_held();
                    board.draw_next();
                    can_hold = false;
                    LockResult::Unlock
                } else {
                    LockResult::NoChange
                }
            }
            56|32 => {
                lock_time = 40;
                board.hard_drop()
            }
            52|KEY_LEFT  => board.move_player(Direction::W),
            54|KEY_RIGHT=> board.move_player(Direction::E),
            50|KEY_DOWN=> board.move_player(Direction::S),
            51|55|122=> board.rotate_player(Direction::W),
            49|53|57|120|KEY_UP => board.rotate_player(Direction::E),
            27 => break,
            _ => LockResult::NoChange,
        };

        match set_lock {
            LockResult::Lock => {
                lock = true;
            }
            LockResult::Unlock => {
                lock = false;
                lock_time = 0;
                //time = 0;
            }
            LockResult::NoChange => {}
        }

        if time >= board.gravity() && !lock {
            time = 0;
            set_lock = board.move_player(Direction::S);
            if let LockResult::Lock = set_lock {
                lock_time = 0;
                lock = true;
            }
        }

        if lock {
            lock_time += 1;
            if lock_time >= 50 {
                board.lock();

                let cont = board.check_lines();
                if cont {
                    board.get_new_piece();
                    board.draw_next();
                    lock = false;
                    can_hold = true;
                    lock_time = 0;
                    time = 0;
                } else {
                    break;
                }
            }
        }

        time += 1;

        board.draw();
        thread::sleep(ten_millis);
    }

    endwin();

    println!("You lost at tetris!! You got {} lines!", board.lines);
}

fn initialize_color() {
    init_pair(1, COLOR_CYAN, COLOR_BLACK);
    init_pair(2, COLOR_YELLOW, COLOR_BLACK);
    init_pair(3, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(4, COLOR_GREEN, COLOR_BLACK);
    init_pair(5, COLOR_RED, COLOR_BLACK);
    init_pair(6, COLOR_BLUE, COLOR_BLACK);
    init_pair(7, COLOR_WHITE, COLOR_BLACK);
}
