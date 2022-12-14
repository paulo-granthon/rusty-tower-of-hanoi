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

    // board data
    cursor: i8,
    spots: Vec<Vec<usize>>,
    disk_count: usize,
    grabbed:usize,

    // move count data
    moves:usize,
    last_dropped:usize
}

impl Board {

    // generate the spots of a board
    fn spots(num_spots:usize, num_disks:usize) -> Vec<Vec<usize>> {
        let mut spots = vec![vec![]; num_spots as usize];
        for i in (1..num_disks+1).rev() {
            spots[0].push(i);
        }
        spots
    }

    // create a new Board
    pub fn new (num_spots:usize, num_disks:usize
        //, margin:i32, padding:i32
    ) -> Self {
        Board {
            cursor:0,
            spots:Self::spots(num_spots, num_disks),
            disk_count:num_disks,
            grabbed:0,
            moves:0,
            last_dropped:usize::MAX
            // width: margin*2 + (num_spots * num_disks) + (num_spots * padding),
            // height: margin*2 + num_disks + (padding * 2),
        }
    }

    // moves the cursor between spots
    pub fn move_cursor (&mut self, dir: i8) {
        use core::cmp::{min, max};

        // clamp between 0 and the number of spots-1
        self.cursor = max(min(self.cursor + dir, (self.spots.len()-1).try_into().unwrap()), 0);
    }

    // grabs a disk from the spot currently under cursor
    pub fn grab (&mut self) -> bool {

        // if already grabbed
        if self.grabbed != 0 { return false };

        // get the disk from the spot
        let grabbed = self.spots[self.cursor as usize].pop();

        // match result
        match grabbed { 

            // if some, replace grabbed and return true
            Some(disk) => {self.grabbed = disk; true},

            // otherwise, false
            _=> false
        }
    }

    // returns true if move should be counted for score
    pub fn drop (&mut self) -> bool {

        // if no grabbed disk
        if self.grabbed == 0 { return false };

        // get the topmost disk on the spot
        let spot_top = self.spots[self.cursor as usize].last();

        // if some and < grabbed: invalid
        // if none or >= grabbed: valid
        if spot_top.is_some() && spot_top.unwrap() < &self.grabbed { return false; }

        // push grabbed to spot
        self.spots[self.cursor as usize].push(self.grabbed);

        // match depending on grabbed
        match self.grabbed {

            // grabbed is other than last dropped: count move
            x if x != self.last_dropped => {
                self.last_dropped = self.grabbed; 
                self.moves += 1;
                self.grabbed = 0;
                true
            },

            // otherwise, moved the same disk: don't count move
            _=> {
                self.grabbed = 0;
                false
            }
        }
    }

    // reset board 
    pub fn reset (&mut self) {

        // recreate spots with the same number of disks
        self.spots = Self::spots(self.spots.len(), self.disk_count);

        // reset state and count values
        self.grabbed = 0;
        self.moves = 0;
        self.last_dropped = usize::MAX;
    }


}

// Draws the given Board
fn draw_board (root: &mut Root, board: &mut Board) {

    // calculate the half sizes of the Board, half number of spots and define padding
    let half:[i8; 2] = [MIN_WIDTH / 2, MIN_HEIGHT / 2];
    let half_len: i8 = (board.spots.len() / 2) as i8;
    const PADDING:i8 = 2;

    // for each spot
    for x in 0..board.spots.len() as i8 {

        // calculate the x and y position of the spot so that all spots are in centered on the screen and equally spaced to one another
        let spot_x: i8 = half[0] - (half_len * board.disk_count as i8) - (half_len * PADDING) + (x * board.disk_count as i8) + (PADDING * x);
        let spot_y: i8 = half[1] + (board.disk_count as i8 / 2);

        // for a range of 0 to the number of disks in play + 1
        for y in 0..board.disk_count as i32 + 1 {

            // if no value at index on spot, draw empty pole at position and skip
            if y >= board.spots[x as usize].len() as i32 {
                root.put_char(spot_x as i32, spot_y as i32 - y, '|', BackgroundFlag::None);
                continue;
            }

            // otherwise, draw the disk at this y position of spot
            draw_disk(root, board.spots[x as usize][y as usize], spot_x, spot_y - y as i8);

        }

        // if this is the spot currently selected by the cursor, draw the cursor above it
        if x == board.cursor {
            // root.put_char(spot_x, 1, '▼', BackgroundFlag::None);
            root.put_char(spot_x.into(), (half[1] - (board.disk_count as i8 / 2) - 4).into(), '@', BackgroundFlag::None);

            // if there's a grabbed disk, draw it too
            if board.grabbed != 0 {
                draw_disk(root, board.grabbed, spot_x, half[1] - (board.disk_count as i8 / 2) - 3)
            }
        }
        
    }

}

// draws a disk at given position
fn draw_disk (root:&mut Root, value:usize, spot_x:i8, spot_y:i8) {

    // define the half x size of the disk as well as a bool to define if the value of the disk is odd and a range calculated from it's size
    let half_x = value as i8 / 2;
    let odd = (value as f32 / 2.0).fract() > 0.0;
    let range = [-half_x - 1, half_x + 2];

    // for each x in the disk's range
    for x in range[0]..range[1] {

        // define the glyph of the spot 
        let glyph = match x {

            // if value >= 10 and x is -1 it means that this position should contain the first digit of the disk's value
            -1 if value >= 10 => char::from_digit((value / 10) as u32, 10).unwrap(),

            // value < 10 or middle
            0 => char::from_digit((value % 10) as u32, 10).unwrap(),

            // left tip
            _x if _x == range[0] => match odd {
                true => {'{'} 
                _=> {'['}
            },

            // right tip
            _x if _x == range[1] -1 => match odd {
                true => {'}'} 
                _=> {']'}
            },

            // fill any other position
            _=> '='
        };
        
        // draw the glyph defined by the conditions        
        root.put_char((spot_x + x) as i32, spot_y as i32, glyph, BackgroundFlag::None);
    }

}

// checks if player won the game
fn check_game_state(board: &mut Board) -> bool {
    let goal = board.spots.last().unwrap().len();
    goal == board.disk_count
}

// main loop
fn main() {

    // setup window
    let mut root = Root::initializer()
        .font(format!("fonts/{}.png", FONT), FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(MIN_WIDTH.into(), MIN_HEIGHT.into())
        .title(format!("Rusty Tower of Hanoi v{} by Paulo Granthon", env!("CARGO_PKG_VERSION")))
        .init();
    tcod::system::set_fps(LIMIT_FPS);

    // open the menu
    menu(&mut root);
}

// renders text on the screen at given position
fn label (root: &mut Root, text: &str, y:i8, anchor:i8, center:bool) {

    // define the starting position of the text. 
    // if center: anchor - half_text_size
    // no center: anchor
    let mut txt_spot_x = anchor - if center {text.len() as i8 / 2} else {0};

    // render each char ad increment position
    for i in text.chars() {
        root.put_char(txt_spot_x.into(), y.into(), i, BackgroundFlag::None);
        txt_spot_x += 1;
    }
}

// opes the game menu with options before playing the game
fn menu(root: &mut Root) {
    
    // define the size of each option on the screen
    const BTN_SIZE: i8 = 10;

    // create a local cursor for the options
    let mut menu_cursor = 0;

    // inicialize data to give to play() 
    let mut settings: [usize; 2] = SETTINGS_DEFAULT;

    // while window is open
    while !root.window_closed() {

        // handle rendering 
        root.set_default_background(colors::BLACK);
        root.clear();

        // define half size of the position and paddding
        let half:[i8; 2] = [MIN_WIDTH/2, MIN_HEIGHT/2];
        const PADDING: i8 = 2;

        // render top labels
        label(root, "Rusty Tower Of Hanoi", 1, half[0], true);
        label(root, "Arrow keys: Change rules", half[1] / 2, half[0], true);

        // for each option
        for i in 0..2 {

            // define it's text position on the screen
            let spot_x: i8 = 2 + half[0] - BTN_SIZE - PADDING + (i as i8 * BTN_SIZE) + (PADDING * i as i8);
            let spot_y: i8 = half[1];

            // render it's text decided by the loop index
            label(root, ["Poles", "Disks"][i], spot_y, spot_x + 3, false);

            // i32 versions of position for the put_char method
            let spot_x32 = spot_x as i32;
            let spot_y32 = spot_y as i32;
    
            // render the current values on settings
            root.put_char(spot_x32, spot_y32, char::from_digit((settings[i] / 10) as u32, 10).unwrap(), BackgroundFlag::None);
            root.put_char(spot_x32 + 1, spot_y32, char::from_digit((settings[i] % 10) as u32, 10).unwrap(), BackgroundFlag::None);
    
            // if this option is selected, render the pseudo cursor
            if i as i32 == menu_cursor {
                root.put_char(spot_x32 + 0, spot_y32 - 2, '/',  BackgroundFlag::None);
                root.put_char(spot_x32 + 1, spot_y32 - 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x32 + 0, spot_y32 + 2, '\\', BackgroundFlag::None);
                root.put_char(spot_x32 + 1, spot_y32 + 2, '/',  BackgroundFlag::None);
            }
    
        }

        // bottom label
        label(root, "Press Enter to play!", MIN_HEIGHT - 3, half[0], true);

        // rendering stuff
        root.flush();

        // wait for keypress at this point
        let key = root.wait_for_keypress(true);

        // import needed code
        use tcod::input::KeyCode::*;
        use core::cmp::{min, max};

        // check if game inputs are triggered 
        match check_game_input(root, key) {

            // quit
            2 | 3 => break,

            // no input
            0 => {

                // check menu inputss
                match key {

                    // move between options
                    tcod::input::Key { code: Left, .. }  => menu_cursor = max(menu_cursor-1, 0),
                    tcod::input::Key { code: Right, .. } => menu_cursor = min(menu_cursor+1, 1),

                    // change option
                    tcod::input::Key { code: Up, .. }   => settings[menu_cursor as usize] = min(settings[menu_cursor as usize] + 1, SETTINGS_MINMAX[menu_cursor as usize][1]),
                    tcod::input::Key { code: Down, .. } => settings[menu_cursor as usize] = max(settings[menu_cursor as usize] - 1, SETTINGS_MINMAX[menu_cursor as usize][0]),

                    // play
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

// play the game with given settings
fn play(root: &mut Root, num_spots:usize, num_disks:usize) {
    
    // create a new board
    let mut board: Board = Board::new(num_spots, num_disks/*, 8, 2*/);
    println!("{:?}", board);

    // game loop
    while !root.window_closed() {

        // rendering
        root.set_default_background(colors::BLACK);
        root.clear();
    
        // top labels
        label(root, "Stack all disks on the rightmost pole", 1, MIN_WIDTH / 2, true);
        label(root, "You can't stack a disk on top of a smaller disk", 2, MIN_WIDTH / 2, true);

        // bottom labels
        label(root, "Left/Right: move | UP: pick disk | Down: drop disk", MIN_HEIGHT-3, MIN_WIDTH / 2, true);
        label(root, "R: reset | Esc: main menu", MIN_HEIGHT-2, MIN_WIDTH / 2, true);
        
        // render the board
        draw_board(root, &mut board);

        // rendering
        root.flush();

        // wait for keypress at this point
        let key = root.wait_for_keypress(true);

        // match board inputs first
        match check_board_inputs(&mut board, key) {

            // reset board
            3 => board.reset(),

            // no board inputs: match game inputs
            _=> match check_game_input(root, key) {

                // back to menu
                2 => {
                    menu(root);
                    break; 
                },

                // exit
                3 => break,

                // no input
                _=>{}
            }
        }

        // check if player solved the puzzle
        if check_game_state(&mut board) { 
            win(root, &mut board);
            break;
        }
    }

}

// called when the player wins the game
fn win (root: &mut Root, board: &mut Board) {

    // define feedback text
    let win_feedback = match board.spots.len() {

        // 3 poles: harder
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

        // 4~5 disks: easy
        4..=5 => { match board.disk_count {
            1..=2 => { "Good job debugging, now play the game -_-'"},
            3..=5 => { "Well done! I guess..."},
            6..=10 => { "Is it actually hard?"},
            11..=13 => { "This probably requires some thinking"},
            _=> { "Hmmmmmm.... that's uhh.. unexpected" }
        }},

        // 6+ disks: pointless
        6.. => { "Good job debugging, now play the game -_-'" }

        // unhandled
        _=> { "Wait, that's ilegal!"}

    };
    
    // print on console as well
    println!("WIN! :D");

    // keep loop while window is open
    while !root.window_closed() {

        // rendering
        root.set_default_background(colors::BLACK);
        root.clear();

        // render labels
        label(root, "You solved the puzzle! :D", 3, MIN_WIDTH / 2, true);
        label(root, win_feedback, 5, MIN_WIDTH / 2, true);
        label(root, &format!("solved in {} move{}", board.moves, if board.moves > 1 {'s'} else { ' ' }), 7, MIN_WIDTH / 2, true);

        // also draw solved board
        draw_board(root, board);

        // rendering
        root.flush();

        // check keypress to leave
        println!("Press (almost) any key to continue");
        while !{use tcod::input::KeyCode::*; match root.wait_for_keypress(true) {
            tcod::input::Key { code: LeftWin, .. } => false,
            tcod::input::Key { code: PrintScreen, .. } => false,
            tcod::input::Key { code: Control, .. } => false,
            tcod::input::Key { code: Char, printable:'c', left_ctrl: true, .. } => false,
            tcod::input::Key { code: Char, printable:'v', left_ctrl: true, .. } => false,
            tcod::input::Key { code: Tab, left_alt: true, .. } => false,
            _=> true
        }} {
            println!("Press (almost) any key to continue");
        }

        // valid key pressed, back to menu
        menu(root);
        break;

    }

}
