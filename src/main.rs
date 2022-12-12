use tcod::colors;
use tcod::console::*;

const MIN_WIDTH: i32 = 64;
const MIN_HEIGHT: i32 = 28;
const LIMIT_FPS: i32 = 60;

// const FONT: &str = "arial10x10";
const FONT: &str = "sb8x8";

fn check_board_inputs (board: &mut Board, key:tcod::input::Key) {
    use tcod::input::KeyCode::*;
    match key {
        // movement keys
        tcod::input::Key { code: Up, .. } => board.grab(), // pick
        tcod::input::Key { code: Down, .. } => board.drop(), // drop
        tcod::input::Key { code: Left, .. } => board.move_cursor(-1),
        tcod::input::Key { code: Right, .. } => board.move_cursor(1),

        _ => {}
    }

}

// handle game inputs
fn check_game_input (root: &mut Root, key:tcod::input::Key) -> bool {
    use tcod::input::KeyCode::*;
    match key {

        // Alt+Enter: toggle fullscreen
        tcod::input::Key {code: Enter, alt: true, .. } => {
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }

        // exit game
        tcod::input::Key { code: Escape, .. } => menu(root),
        tcod::input::Key { code: F4, alt: true, .. } => return true,

        _ => {}
    }

    false
}

#[derive(Debug)]
struct Board {
    width: i32,
    height: i32,
    cursor: i32,
    spots: Vec<Vec<i32>>,
    disk_count: i32,
    cur_disk:i32
}

impl Board {
    pub fn new (num_spots:i32, num_disks:i32, margin:i32, padding:i32) -> Self {
        let mut spots = vec![vec![]; num_spots as usize];
        for i in (1..num_disks+1).rev() {
            spots[0].push(i);
        }
        Board {
            cursor:0,
            spots,
            disk_count:num_disks,
            cur_disk:0,
            width: margin*2 + (num_spots * num_disks) + (num_spots * padding),
            height: margin*2 + num_disks + (padding * 2),
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

    pub fn drop (&mut self) {
        if self.cur_disk == 0 { return };
        let last = self.spots[self.cursor as usize].last();
        if last.is_some() && last.unwrap() < &self.cur_disk { return; }
        self.spots[self.cursor as usize].push(self.cur_disk);
        self.cur_disk = 0;
    }

}

fn draw_board (root: &mut Root, board: &mut Board) {
    use std::cmp::max;

    root.set_default_background(colors::BLACK);
    root.clear();

    // root.put_char(0, 0, 'A', BackgroundFlag::None);
    // root.put_char(1, 0, 'B', BackgroundFlag::None);

    let half:[i32; 2] = [max(MIN_WIDTH, board.width)/2, max(MIN_HEIGHT, board.height)/2];
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


    root.flush();

}

fn draw_piece (root:&mut Root, value:i32, spot_x:i32, spot_y:i32) {
    let half_j = value / 2;
    let rem = (value as f32 / 2.0).fract() > 0.0;
    let range_k = [-half_j - 1, half_j + 1];

    for k in range_k[0]..(range_k[1]+1) {

        // define the glyph of the spot 
        let glyph = 
        if value > 10 && k == -1{
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

fn menu(root: &mut Root) {
    
    const SETTINGS_MINMAX: [[i32; 2]; 2] = [[3, 7], [3, 12]];
    const BTN_SIZE: i32 = 8;

    let mut menu_cursor = 0;
    let mut settings: [i32; 2] = [3, 3];

    while !root.window_closed() {

        root.set_default_background(colors::BLACK);
        root.clear();

        let half:[i32; 2] = [MIN_WIDTH/2, MIN_HEIGHT/2];
        let padding = 2;

        for i in 0..2 {
            let spot_x: i32 = half[0] - BTN_SIZE - padding + (i as i32 * BTN_SIZE) + (padding * i as i32);
            let spot_y: i32 = half[1] + (BTN_SIZE / 2);

            root.put_char(spot_x, spot_y as i32, char::from_digit((settings[i] / 10) as u32, 10).unwrap(), BackgroundFlag::None);
            root.put_char(spot_x+1, spot_y as i32, char::from_digit((settings[i] % 10) as u32, 10).unwrap(), BackgroundFlag::None);
    
            if i as i32 == menu_cursor {
                root.put_char(spot_x, half[1] - 2, '@', BackgroundFlag::None);
            }
    
        }


        root.flush();


        let key = root.wait_for_keypress(true);

        use tcod::input::KeyCode::*;
        use core::cmp::{min, max};
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


        if check_game_input(root, key) { break; }

    }
}

fn play(root: &mut Root, num_spots:i32, num_disks:i32) {
// fn play() {
    
    let mut board: Board = Board::new(num_spots, num_disks, 8, 2);
    println!("{:?}", board);
    
    // setup window
    // use std::cmp::max;
    // let mut root = Root::initializer()
    //     .font(format!("fonts/{}.png", FONT), FontLayout::Tcod)
    //     .font_type(FontType::Greyscale)
    //     .size(max(board.width, MIN_WIDTH), max(board.height, MIN_HEIGHT))
    //     .title(format!("Rusty Tower of Hanoi v{} by Paulo Granthon", env!("CARGO_PKG_VERSION")))
    //     .init();
    // tcod::system::set_fps(LIMIT_FPS);

    let mut win = false;

    // game loop
    while !root.window_closed() {

        draw_board(root, &mut board);

        let key = root.wait_for_keypress(true);
        check_board_inputs(&mut board, key);

        if check_game_state(&mut board) {
            win = true;
            break;
        }

        if check_game_input(root, key) { break; }
    }

    // if win
    if win {
        println!("WIN! :D");
        menu(root);
        return;
    }


}
