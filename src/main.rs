use ncurses::*;

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
    data: Vec<Square>,
    player: Player,
}

impl Board {
    fn new(window: WINDOW) -> Board {
        let mut data: Vec<Square> = Vec::new();
        for i in 0..400 {
            if i >= 350 {
                data.push(Square::FULL(Shape::I));
            } else if i <= 350 && i >= 300 {
                data.push(Square::FULL(Shape::T));
            } else {
                data.push(Square::EMPTY);
            }
        }

        let mut player_data = Vec::new();

        player_data.push(Square::EMPTY);
        for _ in 1..4 {
            player_data.push(Square::FULL(Shape::L));
        }
        for _ in 4..5 {
            player_data.push(Square::EMPTY);
        }
        for _ in 5..16 {
            player_data.push(Square::EMPTY);
        }

        let player = Player {
            data: player_data,
            shape: Shape::L,
            direction: Direction::N,
            y: 20,
            x: 5,
        };

        Board {
            window,
            data,
            player,
        }
    }

    fn get(&self, y: usize, x: usize) -> &Square {
        &self.data[y * 10 + x]
    }

    fn get_mut(&mut self, y: usize, x: usize) -> &mut Square {
        &mut self.data[y * 10 + x]
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

    fn check_board(&self) -> bool{
        //Returns false if game should end
        for y in 0..21{
            for x in 0..10{
                if let Square::FULL(_) = self.get(y, x){
                    return false;
                }
            }
        }
        return true;
    }

    fn new_piece(&mut self){
        let mut data = Vec::new();
        data.push(Square::FULL(Shape::L));
        for _ in 1..16{
            data.push(Square::EMPTY);
        }
        
        self.player = Player{
            data,
            shape: Shape::L,
            direction: Direction::N,
            x:5,
            y:19
        }
    }

    fn dump(&mut self){
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

    fn move_player(&mut self, direction: Direction) -> bool{
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
        } else{
            if let Direction::S = direction{
                self.dump();
                if self.check_board(){
                    self.new_piece();
                    return true;
                } else {
                    return false;
                }
            }
        }
        return true;
    }
}

fn main() {
    initscr();
    cbreak();
    noecho();
    keypad(stdscr(), true);

    start_color();
    initialize_color();

    refresh();

    let mut max_y = 0;
    let mut max_x = 0;

    getmaxyx(stdscr(), &mut max_y, &mut max_x);

    let window = newwin(22, 12, (max_y - 22) / 2, (max_x - 12) / 2);
    box_(window, 0, 0);

    let mut board = Board::new(window);

    board.draw();

    loop {
        match getch() {
            KEY_F1 => break,
            KEY_RIGHT => {board.move_player(Direction::E);},
            KEY_DOWN => if !board.move_player(Direction::S){break;},
            KEY_LEFT => {board.move_player(Direction::W);},
            _ => {}
        }
        board.draw();
    }

    endwin();

    println!("You lost at tetris!!");
}

fn initialize_color() {
    //Forces tetris colors on termial, some colors are
    //different from the name due to limitations
    init_color(COLOR_CYAN, 0, 1000, 1000);
    init_color(COLOR_YELLOW, 1000, 1000, 0);
    init_color(COLOR_MAGENTA, 500, 0, 500);
    init_color(COLOR_GREEN, 0, 1000, 0);
    init_color(COLOR_RED, 1000, 0, 0);
    init_color(COLOR_BLUE, 0, 0, 1000);
    init_color(COLOR_WHITE, 1000, 500, 0);

    init_pair(1, COLOR_CYAN, COLOR_BLACK);
    init_pair(2, COLOR_YELLOW, COLOR_BLACK);
    init_pair(3, COLOR_MAGENTA, COLOR_BLACK);
    init_pair(4, COLOR_GREEN, COLOR_BLACK);
    init_pair(5, COLOR_RED, COLOR_BLACK);
    init_pair(6, COLOR_BLUE, COLOR_BLACK);
    init_pair(7, COLOR_WHITE, COLOR_BLACK);
}
