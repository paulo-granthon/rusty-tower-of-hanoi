[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

# Rusty Tower of Hanoi
Tower of Hanoi implementation with Rust

# What even is a "Tower of Hanoi"?
The Tower of Hanoi is a mathematical game or puzzle consisting of three rods and a number of disks of various diameters, which can slide onto any rod. The puzzle begins with the disks stacked on one rod in order of decreasing size, the smallest at the top, thus approximating a conical shape. The objective of the puzzle is to move the entire stack to the last rod, obeying the following rules:

- Only one disk may be moved at a time.
- Each move consists of taking the upper disk from one of the stacks and placing it on top of another stack or on an empty rod.
- No disk may be placed on top of a disk that is smaller than it.

With 3 disks, the puzzle can be solved in 7 moves. The minimal number of moves required to solve a Tower of Hanoi puzzle is 2n − 1, where n is the number of disks.

# My modifications to the original puzzle
Number of disks and rods can be configured.  

# References & Technologies
Tower of Hanoi - https://en.wikipedia.org/wiki/Tower_of_Hanoi  
Rust - https://www.rust-lang.org/  
rendering (libtcod) - https://crates.io/crates/tcod  

# Screenshots
![alt text](https://github.com/paulo-granthon/rusty-tower-of-hanoi/blob/main/pics/rtoh_game1.png?raw=true)
![alt text](https://github.com/paulo-granthon/rusty-tower-of-hanoi/blob/main/pics/rtoh_game2.png?raw=true)
![alt text](https://github.com/paulo-granthon/rusty-tower-of-hanoi/blob/main/pics/rtoh_menu.png?raw=true)
![alt text](https://github.com/paulo-granthon/rusty-tower-of-hanoi/blob/main/pics/rtoh_win.png?raw=true)
