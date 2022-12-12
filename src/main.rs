use tcod::colors;
use tcod::console::*;

const MIN_WIDTH: i32 = 128;
const MIN_HEIGHT: i32 = 32;
const LIMIT_FPS: i32 = 60;

const SETTINGS_MINMAX: [[i32; 2]; 2] = [
    [3, 7], // Poles
    [1, 12] // Disks
];

const SETTINGS_DEFAULT: [i32; 2] = [3, 3];

// const FONT: &str = "arial10x10";
const FONT: &str = "sb8x8";

fn check_board_inputs (board: &mut Board, key:tcod::input::Key, moves:&mut i32) {
    use tcod::input::KeyCode::*;
    match key {
        // movement keys
        tcod::input::Key { code: Up, .. } => board.grab(), // pick
        tcod::input::Key { code: Down, .. } => { if board.drop() { *moves += 1; }},
        tcod::input::Key { code: Left, .. } => board.move_cursor(-1),
        tcod::input::Key { code: Right, .. } => board.move_cursor(1),

        _ => {}
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
        tcod::input::Key { code: Escape, .. } => menu(root),
        tcod::input::Key { code: F4, alt: true, .. } => return 2,

        _ => {}
    }

    0
}

#[derive(Debug)]
struct Board {
    // width: i32,
    // height: i32,
    cursor: i32,
    spots: Vec<Vec<i32>>,
    disk_count: i32,
    cur_disk:i32
}

impl Board {
    pub fn new (num_spots:i32, num_disks:i32
        //, margin:i32, padding:i32
    ) -> Self {
        let mut spots = vec![vec![]; num_spots as usize];
        for i in (1..num_disks+1).rev() {
            spots[0].push(i);
        }
        Board {
            cursor:0,
            spots,
            disk_count:num_disks,
            cur_disk:0,
            // width: margin*2 + (num_spots * num_disks) + (num_spots * padding),
            // height: margin*2 + num_disks + (padding * 2),
        }
    }

    pub fn move_cursor (&mut self, dir: i32) {
        use core::cmp::{min, max};
        self.cursor = max(min(self.cursor + dir, (self.spots.len()-1).try_into().unwrap()), 0);
    }

    pub fn grab (&mut self) {
        if self.cur_disk != 0 { return };
        let grabbed = self.spots[self.cursor as usize].pop();
        if grabbed.is_some() { self.cur_disk = grabbed.unwrap(); }
    }

    pub fn drop (&mut self) -> bool {
        if self.cur_disk == 0 { return false };
        let last = self.spots[self.cursor as usize].last();
        if last.is_some() && last.unwrap() < &self.cur_disk { return false; }
        self.spots[self.cursor as usize].push(self.cur_disk);
        self.cur_disk = 0;
        true
    }

}

fn draw_board (root: &mut Root, board: &mut Board) {

    let half:[i32; 2] = [MIN_WIDTH / 2, MIN_HEIGHT / 2];
    let half_len: i32 = (board.spots.len() / 2) as i32;
    let padding = 2;

    // for each spot
    for i in 0..board.spots.len() {
        let spot_x: i32 = half[0] - (half_len * board.disk_count) - (half_len * padding) + (i as i32 * board.disk_count) + (padding * i as i32);
        let spot_y: i32 = half[1] + (board.disk_count / 2);

        // for each value on spot
        for j in 0..board.disk_count as usize + 1 {

            // if no value at index on spot, draw bare pole and skip
            if j >= board.spots[i].len() {
                root.put_char(spot_x, spot_y - j as i32, '|', BackgroundFlag::None);
                continue;
            }

            draw_piece(root, board.spots[i][j], spot_x, spot_y - j as i32);

        }
        if i as i32 == board.cursor {
            // root.put_char(spot_x, 1, '▼', BackgroundFlag::None);
            root.put_char(spot_x, half[1] - (board.disk_count / 2) - 4, '@', BackgroundFlag::None);
            if board.cur_disk != 0 {
                draw_piece(root, board.cur_disk as i32, spot_x, half[1] - (board.disk_count / 2) - 3)
            }
        }
        
    }

}

fn draw_piece (root:&mut Root, value:i32, spot_x:i32, spot_y:i32) {
    let half_j = value / 2;
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
        
        root.put_char(spot_x + k, spot_y as i32, glyph, BackgroundFlag::None);
    }

}

fn check_game_state(board: &mut Board) -> bool {
    let goal = board.spots.last().unwrap().len() as i32;
    // println!("goal: {}", goal);
    goal == board.disk_count
}

fn main() {

    // setup window
    let mut root = Root::initializer()
        .font(format!("fonts/{}.png", FONT), FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(MIN_WIDTH, MIN_HEIGHT)
        .title(format!("Rusty Tower of Hanoi v{} by Paulo Granthon", env!("CARGO_PKG_VERSION")))
        .init();
    tcod::system::set_fps(LIMIT_FPS);

    menu(&mut root);
    // play();
}

fn label (root: &mut Root, text: &str, y:i32, anchor:i32, center:bool) {
    let mut txt_spot_x = anchor - if center {text.len() as i32 / 2} else {0};
    for i in text.chars() {
        root.put_char(txt_spot_x, y as i32, i, BackgroundFlag::None);
        txt_spot_x += 1;
    }
}

fn menu(root: &mut Root) {
    
    const BTN_SIZE: i32 = 10;

    let mut menu_cursor = 0;
    let mut settings: [i32; 2] = SETTINGS_DEFAULT;

    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();

        let half:[i32; 2] = [MIN_WIDTH/2, MIN_HEIGHT/2];
        let padding = 2;

        label(root, "Rusty Tower Of Hanoi", 1, half[0], true);
        label(root, "Arrow keys: Change rules", half[1] / 2, half[0], true);

        for i in 0..2 {
            let spot_x: i32 = 2 + half[0] - BTN_SIZE - padding + (i as i32 * BTN_SIZE) + (padding * i as i32);
            let spot_y: i32 = half[1];

            label(root, ["Poles", "Disks"][i], spot_y, spot_x + 3, false);
    
            root.put_char(spot_x, spot_y as i32, char::from_digit((settings[i] / 10) as u32, 10).unwrap(), BackgroundFlag::None);
            root.put_char(spot_x+1, spot_y as i32, char::from_digit((settings[i] % 10) as u32, 10).unwrap(), BackgroundFlag::None);
    
            if i as i32 == menu_cursor {
                root.put_char(spot_x + 0, spot_y - 2, '/',  BackgroundFlag::None);
                root.put_char(spot_x + 1, spot_y - 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x + 0, spot_y + 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x + 1, spot_y + 2, '/',  BackgroundFlag::None);
            }
    
        }

        label(root, "Press Enter to play!", MIN_HEIGHT - 3, half[0], true);

        root.flush();

        let key = root.wait_for_keypress(true);

        use tcod::input::KeyCode::*;
        use core::cmp::{min, max};

        let game_input_result = check_game_input(root, key);
        if game_input_result > 0 {
            if game_input_result == 2 { break; }
        }

        else { match key {
            tcod::input::Key { code: Left, .. }  => menu_cursor = max(menu_cursor-1, 0),
            tcod::input::Key { code: Right, .. } => menu_cursor = min(menu_cursor+1, 1),
            tcod::input::Key { code: Up, .. }   => settings[menu_cursor as usize] = min(settings[menu_cursor as usize] + 1, SETTINGS_MINMAX[menu_cursor as usize][1]),
            tcod::input::Key { code: Down, .. } => settings[menu_cursor as usize] = max(settings[menu_cursor as usize] - 1, SETTINGS_MINMAX[menu_cursor as usize][0]),
            tcod::input::Key { code: Enter, .. } => {
                play(root, settings[0], settings[1]);
                break;
            },
            _=> {}
        }}

    }
}

fn play(root: &mut Root, num_spots:i32, num_disks:i32) {
    
    let mut board: Board = Board::new(num_spots, num_disks/*, 8, 2*/);
    println!("{:?}", board);

    let mut moves = 0;

    // game loop
    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();
    
        label(root, "Stack all disks on the rightmost pole", 1, MIN_WIDTH / 2, true);
        label(root, "You can't stack a disk on top of a smaller disk", 2, MIN_WIDTH / 2, true);

        label(root, "Left/Right: move | UP: pick disk | Down: drop disk | Esc: main menu", MIN_HEIGHT-2, MIN_WIDTH / 2, true);

        draw_board(root, &mut board);

        root.flush();

        let key = root.wait_for_keypress(true);
        check_board_inputs(&mut board, key, &mut moves);

        if check_game_state(&mut board) { 
            win(root, &mut board, moves);
            break;
        }

        if check_game_input(root, key) == 2 { break; }
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
