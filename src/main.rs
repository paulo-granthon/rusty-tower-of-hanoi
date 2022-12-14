use tcod::colors;
use tcod::console::*;

const MIN_WIDTH: i8 = 108;
const MIN_HEIGHT: i8 = 32;
const LIMIT_FPS: i32 = 60;

const SETTINGS_MINMAX: [[usize; 2]; 2] = [
    [3, 7], // Poles
    [1, 12] // Disks
];

const SETTINGS_DEFAULT: [usize; 2] = [3, 3];

// const FONT: &str = "arial10x10";
const FONT: &str = "sb8x8";


fn check_board_inputs (board: &mut Board, key:tcod::input::Key) -> i8 {
    use tcod::input::KeyCode::*;
    match key {

        // action keys
        tcod::input::Key { code: Up, .. }       => if board.grab() {1} else {0},
        tcod::input::Key { code: Down, .. }     => if board.drop() {2} else {0},

        // movement keys
        tcod::input::Key { code: Left, .. }     => { board.move_cursor(-1); 0},
        tcod::input::Key { code: Right, .. }    => { board.move_cursor(1);  0},

        // reset board
        tcod::input::Key { code: Char, printable:'r', .. } => 3,

        // unmaped
        _ => {0}
    }
}

// handle game inputs
fn check_game_input (root: &mut Root, key:tcod::input::Key) -> i32 {
    use tcod::input::KeyCode::*;
    match key {

        // Alt+Enter: toggle fullscreen
        tcod::input::Key {code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
            return 1
        }

        // exit game
        tcod::input::Key { code: Escape, .. } => return 2,
        tcod::input::Key { code: F4, alt: true, .. } => return 3,

        _ => {}
    }

    0
}

#[derive(Debug)]
struct Board {
    // width: i8,
    // height: i8,
    cursor: i8,
    spots: Vec<Vec<usize>>,
    disk_count: usize,
    cur_disk:usize
}

impl Board {
    fn spots(num_spots:usize, num_disks:usize) -> Vec<Vec<usize>> {
        let mut spots = vec![vec![]; num_spots as usize];
        for i in (1..num_disks+1).rev() {
            spots[0].push(i);
        }
        spots
    }
    pub fn new (num_spots:usize, num_disks:usize
        //, margin:i32, padding:i32
    ) -> Self {
        Board {
            cursor:0,
            spots:Self::spots(num_spots, num_disks),
            disk_count:num_disks,
            cur_disk:0,
            // width: margin*2 + (num_spots * num_disks) + (num_spots * padding),
            // height: margin*2 + num_disks + (padding * 2),
        }
    }

    pub fn move_cursor (&mut self, dir: i8) {
        use core::cmp::{min, max};
        self.cursor = max(min(self.cursor + dir, (self.spots.len()-1).try_into().unwrap()), 0);
    }

    pub fn grab (&mut self) -> bool {
        if self.cur_disk != 0 { return false };
        let grabbed = self.spots[self.cursor as usize].pop();
        if grabbed.is_some() { self.cur_disk = grabbed.unwrap(); }
        true
    }

    pub fn drop (&mut self) -> bool {
        if self.cur_disk == 0 { return false };
        let last = self.spots[self.cursor as usize].last();
        if last.is_some() && last.unwrap() < &self.cur_disk { return false; }
        self.spots[self.cursor as usize].push(self.cur_disk);
        self.cur_disk = 0;
        true
    }

    pub fn reset (&mut self) {
        self.spots = Self::spots(self.spots.len(), self.disk_count);
        self.cur_disk = 0;
    }

}

fn draw_board (root: &mut Root, board: &mut Board) {

    let half:[i8; 2] = [MIN_WIDTH / 2, MIN_HEIGHT / 2];
    let half_len: i8 = (board.spots.len() / 2) as i8;
    let padding = 2;

    // for each spot
    for i in 0..board.spots.len() as i8 {
        let spot_x: i8 = half[0] - (half_len * board.disk_count as i8) - (half_len * padding) + (i * board.disk_count as i8) + (padding * i);
        let spot_y: i8 = half[1] + (board.disk_count as i8 / 2);

        // for each value on spot
        for j in 0..board.disk_count as i32 + 1 {

            // if no value at index on spot, draw bare pole and skip
            if j >= board.spots[i as usize].len() as i32 {
                root.put_char(spot_x as i32, spot_y as i32 - j, '|', BackgroundFlag::None);
                continue;
            }

            draw_piece(root, board.spots[i as usize][j as usize], spot_x, spot_y - j as i8);

        }
        if i == board.cursor {
            // root.put_char(spot_x, 1, '▼', BackgroundFlag::None);
            root.put_char(spot_x.into(), (half[1] - (board.disk_count as i8 / 2) - 4).into(), '@', BackgroundFlag::None);
            if board.cur_disk != 0 {
                draw_piece(root, board.cur_disk, spot_x, half[1] - (board.disk_count as i8 / 2) - 3)
            }
        }
        
    }

}

fn draw_piece (root:&mut Root, value:usize, spot_x:i8, spot_y:i8) {
    let half_j = value as i8 / 2;
    let rem = (value as f32 / 2.0).fract() > 0.0;
    let range_k = [-half_j - 1, half_j + 1];

    for k in range_k[0]..(range_k[1]+1) {

        // define the glyph of the spot 
        let glyph = 
        if value >= 10 && k == -1{
            char::from_digit((value / 10) as u32, 10).unwrap()
        } else if k == 0 {
            char::from_digit((value % 10) as u32, 10).unwrap()
        } else if k == range_k[0] {
            if rem {'{'} else {'['}
        } else if k == range_k[1] {
            if rem {'}'} else {']'}
        } else {
            '='
        };

        // println!("[{}]: {}", k, glyph);
        
        root.put_char((spot_x + k) as i32, spot_y as i32, glyph, BackgroundFlag::None);
    }

}

fn check_game_state(board: &mut Board) -> bool {
    let goal = board.spots.last().unwrap().len();
    // println!("goal: {}", goal);
    goal == board.disk_count
}

fn main() {

    // setup window
    let mut root = Root::initializer()
        .font(format!("fonts/{}.png", FONT), FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(MIN_WIDTH.into(), MIN_HEIGHT.into())
        .title(format!("Rusty Tower of Hanoi v{} by Paulo Granthon", env!("CARGO_PKG_VERSION")))
        .init();
    tcod::system::set_fps(LIMIT_FPS);

    menu(&mut root);
    // play();
}

fn label (root: &mut Root, text: &str, y:i8, anchor:i8, center:bool) {
    let mut txt_spot_x = anchor - if center {text.len() as i8 / 2} else {0};
    for i in text.chars() {
        root.put_char(txt_spot_x.into(), y.into(), i, BackgroundFlag::None);
        txt_spot_x += 1;
    }
}

fn menu(root: &mut Root) {
    
    const BTN_SIZE: i8 = 10;

    let mut menu_cursor = 0;
    let mut settings: [usize; 2] = SETTINGS_DEFAULT;

    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();

        let half:[i8; 2] = [MIN_WIDTH/2, MIN_HEIGHT/2];
        let padding = 2;

        label(root, "Rusty Tower Of Hanoi", 1, half[0], true);
        label(root, "Arrow keys: Change rules", half[1] / 2, half[0], true);

        for i in 0..2 {
            let spot_x: i8 = 2 + half[0] - BTN_SIZE - padding + (i as i8 * BTN_SIZE) + (padding * i as i8);
            let spot_y: i8 = half[1];

            label(root, ["Poles", "Disks"][i], spot_y, spot_x + 3, false);

            let spot_x32 = spot_x as i32;
            let spot_y32 = spot_y as i32;
    
            root.put_char(spot_x32, spot_y32, char::from_digit((settings[i] / 10) as u32, 10).unwrap(), BackgroundFlag::None);
            root.put_char(spot_x32 + 1, spot_y32, char::from_digit((settings[i] % 10) as u32, 10).unwrap(), BackgroundFlag::None);
    
            if i as i32 == menu_cursor {
                root.put_char(spot_x32 + 0, spot_y32 - 2, '/',  BackgroundFlag::None);
                root.put_char(spot_x32 + 1, spot_y32 - 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x32 + 0, spot_y32 + 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x32 + 1, spot_y32 + 2, '/',  BackgroundFlag::None);
            }
    
        }

        label(root, "Press Enter to play!", MIN_HEIGHT - 3, half[0], true);

        root.flush();

        let key = root.wait_for_keypress(true);

        use tcod::input::KeyCode::*;
        use core::cmp::{min, max};

        match check_game_input(root, key) {
            2 | 3 => break,
            0 => {
                match key {
                    tcod::input::Key { code: Left, .. }  => menu_cursor = max(menu_cursor-1, 0),
                    tcod::input::Key { code: Right, .. } => menu_cursor = min(menu_cursor+1, 1),
                    tcod::input::Key { code: Up, .. }   => settings[menu_cursor as usize] = min(settings[menu_cursor as usize] + 1, SETTINGS_MINMAX[menu_cursor as usize][1]),
                    tcod::input::Key { code: Down, .. } => settings[menu_cursor as usize] = max(settings[menu_cursor as usize] - 1, SETTINGS_MINMAX[menu_cursor as usize][0]),
                    tcod::input::Key { code: Enter, .. } => {
                        play(root, settings[0], settings[1]);
                        break;
                    },
                    _=> {}
                }
            }
            _=>{}
        }

    }
}

fn play(root: &mut Root, num_spots:usize, num_disks:usize) {
    
    let mut board: Board = Board::new(num_spots, num_disks/*, 8, 2*/);
    println!("{:?}", board);

    let mut moves = 0;
    let mut last_grabbed_spot = -1;

    // game loop
    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();
    
        label(root, "Stack all disks on the rightmost pole", 1, MIN_WIDTH / 2, true);
        label(root, "You can't stack a disk on top of a smaller disk", 2, MIN_WIDTH / 2, true);

        label(root, "Left/Right: move | UP: pick disk | Down: drop disk", MIN_HEIGHT-3, MIN_WIDTH / 2, true);
        label(root, "R: reset | Esc: main menu", MIN_HEIGHT-2, MIN_WIDTH / 2, true);
        
        draw_board(root, &mut board);

        root.flush();

        let key = root.wait_for_keypress(true);
        match check_board_inputs(&mut board, key) {
            1 => last_grabbed_spot = board.cursor,
            2 => if board.cursor != last_grabbed_spot { moves += 1; println!("{} moves", moves) },
            3 => board.reset(),
            _=>{}
        }

        if check_game_state(&mut board) { 
            win(root, &mut board, moves);
            break;
        }

        match check_game_input(root, key) {
            2 => {
                menu(root);
                break; 
            },
            3 => break,
            _=>{}
        } 
    }

}

fn win (root: &mut Root, board: &mut Board, moves: i32) {
    println!("WIN! :D");

    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();

        let win_feedback = match board.spots.len() {
            3 => { match board.disk_count {
                1 => { "Good job debugging, now play the game -_-'"},
                2 => { "Wow! That was easy huhh...?"},
                3 => { "Well done!"},
                4 => { "Good job!"},
                5 => { "Smart!"},
                6 => { "Very Smart!"},
                7 => { "Amazing!"},
                8 => { "You are crazy!!"},
                9..=10 => { "Wow! That's impressive :o"},
                11 => { "You. Are. A. Legend"},
                12 => { "How in the.....???"},
                _=> { "Hmmmmmm.... that's uhh.. unexpected" }
            }},
            4..=5 => { match board.disk_count {
                1..=2 => { "Good job debugging, now play the game -_-'"},
                3..=5 => { "Well done! I guess..."},
                6..=10 => { "Is it actually hard?"},
                11..=13 => { "This probably requires some thinking"},
                _=> { "Hmmmmmm.... that's uhh.. unexpected" }
            }},
            6.. => { "Good job debugging, now play the game -_-'" }
            _=> { "Wait, that's ilegal!"}

        };

        label(root, "You solved the puzzle! :D", 3, MIN_WIDTH / 2, true);
        label(root, win_feedback, 5, MIN_WIDTH / 2, true);
        label(root, &format!("solved in {} move{}", moves, if moves > 1 {'s'} else { ' ' }), 7, MIN_WIDTH / 2, true);

        draw_board(root, board);

        root.flush();

        root.wait_for_keypress(true);
        // let game_input_result = check_game_input(root, key);
        // if game_input_result > 0 {
            menu(root);
            break;
        // }

    }

}
