use ncurses::*;

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

    let board = Board::new(window);

    board.draw();

    getch();
    endwin();
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

enum Shape {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

enum Square {
    FULL(Shape),
    EMPTY,
}

struct Board {
    window: WINDOW,
    data: Vec<Square>,
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
        Board { window, data }
    }

    fn get(&self, y: usize, x: usize) -> &Square {
        &self.data[y * 10 + x]
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
        wrefresh(self.window);
    }
}
