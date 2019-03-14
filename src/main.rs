use ncurses::*;
use rand::thread_rng;
use rand::seq::SliceRandom;
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
    lines_window: WINDOW,
    level_window: WINDOW,
    data: Vec<Square>,
    player: Player,
    grab_bag: Vec<Player>,
    lines: usize,
}

impl Board {
    fn new(window: WINDOW, lines_window: WINDOW, level_window: WINDOW) -> Board {
        let mut data: Vec<Square> = Vec::new();
        for _ in 0..400 {
            data.push(Square::EMPTY);
        } 

        let mut grab_bag = Board::make_grab_bag();
        let player = grab_bag.pop().unwrap();

        Board {
            window,
            lines_window,
            level_window,
            data,
            player,
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

    fn gravity(&self) -> usize{
        let level = self.lines / 10;
        ((0.8 - ((level as f64 - 1.0) * 0.007)).powi(level as i32 - 1) * 100.0) as usize
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

        let p_y = self.player.y;
        let p_x = self.player.x;

        wattron(self.window, color);
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

    fn make_piece(map: i32, shape: Shape) -> Player{
        let mut data = Vec::new();
        let mut i = 0b1;
        
        for _ in 0..16{
            if i & map != 0{
                data.push(Square::FULL(shape));
            } else {
                data.push(Square::EMPTY);
            }
            i <<= 1;
        }

        Player{
            data,
            shape,
            direction: Direction::N,
            y: 18,
            x: 3,
        }

    }

    fn make_grab_bag() -> Vec<Player>{
        let mut bag = Vec::new();

        bag.push(Board::make_piece(0b1111_0000, Shape::I));
        bag.push(Board::make_piece(0b0110_0110, Shape::O));
        bag.push(Board::make_piece(0b0111_0010, Shape::T));
        bag.push(Board::make_piece(0b0011_0110, Shape::S));
        bag.push(Board::make_piece(0b0110_0011, Shape::Z));
        bag.push(Board::make_piece(0b0111_0001, Shape::J));
        bag.push(Board::make_piece(0b0111_0100, Shape::L));

        let mut rng = thread_rng();
        let slice: &mut [Player] = &mut bag;
        slice.shuffle(&mut rng);

        bag
    }

    fn get_new_piece(&mut self){
        if self.grab_bag.len() == 1{
            self.grab_bag.append(&mut Board::make_grab_bag());
        }
        self.player = self.grab_bag.pop().unwrap();
    }


    fn clear_line(&mut self, y:usize){
        let mut new_data = Vec::new();
        for _ in 0..10{
            new_data.push(Square::EMPTY);
        }
        for _ in 0..10{
            self.data.remove(10 * y);
        }
        new_data.append(&mut self.data);
        self.data = new_data;
    }

    fn draw_score(&self){
        mvwprintw(self.lines_window, 1, 1, &format!("{:>3}", self.lines));
        wrefresh(self.lines_window);

        mvwprintw(self.level_window, 1, 1, &format!("{:>3}", self.lines/10));
        wrefresh(self.level_window);    
    }

    fn check_lines(& mut self) -> bool{
        //Returns true if game should continue
        for y in 0..20{
            for x in 0..10{
                if let Square::FULL(_) = self.get(y, x){
                    return false;
                }
            }
        }

        let mut sum = 0;
        for y in (20..40).rev(){
            loop{
                for x in 0..10{
                    if let Square::FULL(_) = self.get(y, x){
                        sum += 1;
                    }
                }
                if sum == 10{
                    self.clear_line(y);
                    self.lines += 1;
                    sum = 0;
                } else{
                    sum=0;
                    break;
                }
            }
        }

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

    fn lock(&mut self){
        let p_y = self.player.y;
        let p_x = self.player.x;

        for y in 0..4{
            for x in 0..4{
                if 0 <= (y + p_y) && (y + p_y) <= 39 && 0 <= (x + p_x) && (x + p_x) <= 9 {
                    if let Square::FULL(_) = self.player.data[(y * 4 + x) as usize] {
                        *self.get_mut((p_y + y) as usize, (p_x + x) as usize) = self.player.data[(y * 4 + x) as usize];
                    }
                }
            }
        }
    } 

    fn move_player(&mut self, direction: Direction) -> LockResult{
        let new_player = match direction {
            Direction::N => {
                self.player.clone()
            },
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

        if !self.collision(&new_player){
            self.player = new_player;
            return LockResult::Unlock
        } else{
            if let Direction::S = direction{
                return LockResult::Lock;
            }
        }
        LockResult::NoChange
    }

    fn hard_drop(&mut self) -> LockResult{
        loop{
            let lock = self.move_player(Direction::S);
            if let LockResult::Lock = lock{
                break;
            }
        }
        LockResult::Lock
    }

    fn rotate_player(&mut self, direction: Direction) -> LockResult{
        let mut rotated_data = self.player.data.clone();
        let rotated_data = match direction{
            Direction::E => { 
                match self.player.shape {
                    Shape::O => {rotated_data},
                    Shape::I => {
                        for y in 0..4{
                            for x in 0..4{
                                rotated_data[y * 4 + x] = self.player.data[(3 - x) * 4 + y];
                            }
                        }
                        rotated_data
                    },
                    _ => {
                        for y in 0..3{
                            for x in 0..3{
                                rotated_data[y * 4 + x] = self.player.data[(2 - x) * 4 + y];
                            } 
                        }
                        rotated_data

                    }
                }
            },
            Direction::W => { 
                match self.player.shape {
                    Shape::O => {rotated_data},
                    Shape::I => {
                        for y in 0..4{
                            for x in 0..4{
                                rotated_data[y * 4 + x] = self.player.data[x * 4 + (3 - y)];
                            }
                        }
                        rotated_data
                    },
                    _ => {
                        for y in 0..3{
                            for x in 0..3{
                                rotated_data[y * 4 + x] = self.player.data[x * 4 + (2 - y)];
                            } 
                        }
                        rotated_data

                    }
                }
            },
            _ => {rotated_data}
        };

        let rotated_player = Player{data: rotated_data, ..self.player.clone()};

        if !self.collision(&rotated_player){
            self.player = rotated_player;
            return LockResult::Unlock;
        } else {
            return LockResult::NoChange;
        }
    }
}

enum LockResult{
    Lock, Unlock, NoChange
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

    mvprintw(y - 2, x + 3, "cursed");
    refresh();

    mvprintw(y + 2, x + 12 + 2, "LINES");
    let lines_window = newwin(3, 5, y + 3, x + 12 + 2);
    box_(lines_window, 0, 0);
    wrefresh(lines_window);


    mvprintw(y + 7, x + 12 + 2, "LEVEL");
    let level_window = newwin(3, 5, y + 8, x + 12 + 2);
    box_(level_window, 0, 0);
    wrefresh(level_window);


    let mut board = Board::new(window, lines_window, level_window);

    board.draw();

    let ten_millis = time::Duration::from_millis(10);
    let mut time = 0;
    let mut lock_time = 0;
    let mut lock = false;


    loop {
        let mut set_lock = match getch() {
            48 => break, //hold
            56 => {board.hard_drop()}, //hard drop
            52 => board.move_player(Direction::W),
            54 => board.move_player(Direction::E),
            50 => board.move_player(Direction::S),
            51|55 => board.rotate_player(Direction::W),
            49|53|57 => board.rotate_player(Direction::E),
            _ => {LockResult::NoChange}
        };

        match set_lock {
            LockResult::Lock => {
                lock = true;
            },
            LockResult::Unlock => {
                lock = false;
                lock_time = 0;
                time = 0;
            },
            LockResult::NoChange => {}
        }

        if time >= board.gravity() && !lock{
            time = 0;
            set_lock = board.move_player(Direction::S);
            if let LockResult::Lock = set_lock{
                lock_time = 0;
                lock = true;
            }
        }

        if lock {
            lock_time += 1;
            if lock_time >= 50{
                board.lock();

                let cont = board.check_lines();
                if cont {
                    board.get_new_piece();
                    lock = false;
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

    println!("You lost at tetris!!");
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
