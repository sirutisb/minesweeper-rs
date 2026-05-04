use macroquad::{miniquad::date, prelude::*, rand::{gen_range, srand}};

#[derive(Copy, Clone)]
struct Cell {
    bomb: bool,
    revealed: bool,
    flagged: bool,
    number: u32,
}

const ROWS: usize = 16;
const COLS: usize = 16;
const NUM_BOMBS: usize = 40;

const GAME_WIDTH: usize = 1280;
const GAME_HEIGHT: usize = 1280;

const CELL_WIDTH : f32 = GAME_WIDTH as f32 / COLS as f32;
const CELL_HEIGHT : f32 = GAME_HEIGHT as f32 / ROWS as f32;

const DIGIT_FONT_SIZE : f32 = CELL_WIDTH;
const FLAG_FONT_SIZE : f32 = CELL_WIDTH;
const BOMB_FONT_SIZE : f32 = CELL_WIDTH / 2.0;

#[macroquad::main("Minesweeper")]
async fn main() {
    srand(date::now() as u64);

    request_new_screen_size(GAME_WIDTH as f32, GAME_HEIGHT as f32);

    let mut grid: [[Cell; COLS]; ROWS] = [[Cell {
            bomb: false,
            revealed: false,
            flagged: false,
            number: 0,
    }; COLS]; ROWS];

    let mut started_game = false;

    loop {
        // ----- HANDLE INPUT -----
        let lmouse_down = is_mouse_button_pressed(MouseButton::Left);
        let rmouse_down = is_mouse_button_pressed(MouseButton::Right);
        let (mouse_x, mouse_y) = mouse_position();

        // ----- UPDATE STATE -----
        if rmouse_down {
            let clicked_r : i32 = (mouse_y / CELL_HEIGHT) as i32;
            let clicked_c : i32 = (mouse_x / CELL_WIDTH) as i32;

            if bounds_check(clicked_r, clicked_c) {
                let r = clicked_r as usize;
                let c = clicked_c as usize;
                let cell = &mut grid[r][c];

                if !cell.revealed {
                    cell.flagged = !cell.flagged;
                }
            }
        }

        if lmouse_down {
            let clicked_r : i32 = (mouse_y / CELL_HEIGHT) as i32;
            let clicked_c : i32 = (mouse_x / CELL_WIDTH) as i32;

            if bounds_check(clicked_r, clicked_c) {
                let r = clicked_r as usize;
                let c = clicked_c as usize;

                // The first click should be safe, only generate bombs after first click
                if !started_game {
                    // generate 10 bombs, excluding where we clicked (so we cant insta-die)
                    started_game = true;
                    generate_bombs(&mut grid, NUM_BOMBS, r, c);
                    compute_bomb_neighbours(&mut grid);
                }

                let cell = &mut grid[r][c];
                if cell.revealed || cell.flagged {
                    // do nothing, cant click already revealed or flagged cell
                } else {
                    if cell.bomb {
                        reveal_all_bombs(&mut grid);
                        println!("Game Over! Boom!");
                        // TODO: add reset game functionality
                    } else {
                        // Note: if its already revealed it ignores the cell implicitly
                        bfs_reveal_cells(&mut grid, r, c);
                    }
                }
            }
        }

        // ----- START RENDER -----
        clear_background(GRAY);

        // Draw cells
        for r in 0..ROWS {
            for c in 0..COLS {
                let y : f32 = r as f32 * CELL_HEIGHT;
                let x : f32 = c as f32 * CELL_WIDTH;

                let cell : &Cell = &grid[r][c];
                if !cell.revealed {
                    // Not revealed
                    draw_rectangle(x, y, CELL_WIDTH, CELL_HEIGHT, DARKGRAY);
                    if cell.flagged {
                        let text_to_draw = "F";
                        let measured_size = measure_text(&text_to_draw, None, FLAG_FONT_SIZE as u16, 1.0);

                        // Center the text in the 100x100 cell
                        let text_x = x + (CELL_WIDTH - measured_size.width) / 2.0;
                        let text_y = y + (CELL_HEIGHT + measured_size.offset_y) / 2.0;

                        draw_text(&text_to_draw, text_x, text_y, FLAG_FONT_SIZE, GREEN);
                    }
                } else {
                    // Revealed
                    if !cell.bomb {
                        // show number if > 0
                        if cell.number > 0 {

                            let text_to_draw = cell.number.to_string();
                            let measured_size = measure_text(&text_to_draw, None, DIGIT_FONT_SIZE as u16, 1.0);

                            // Center the text in the 100x100 cell
                            let text_x = x + (CELL_WIDTH - measured_size.width) / 2.0;
                            let text_y = y + (CELL_HEIGHT + measured_size.offset_y) / 2.0;

                            draw_text(&text_to_draw, text_x, text_y, DIGIT_FONT_SIZE, BLUE);
                        }
                    } else {
                        let text_to_draw = "BOMB";
                        let measured_size = measure_text(&text_to_draw, None, BOMB_FONT_SIZE as u16, 1.0);

                        // Center the text in the 100x100 cell
                        let text_x = x + (CELL_WIDTH - measured_size.width) / 2.0;
                        let text_y = y + (CELL_HEIGHT + measured_size.offset_y) / 2.0;

                        draw_text(&text_to_draw, text_x, text_y, BOMB_FONT_SIZE, PURPLE);
                    }
                }
            }
        }
        // Draw grid lines
        for r in 0..=ROWS {
            let y : f32 = r as f32 * CELL_HEIGHT;
            draw_line(0.0, y, CELL_WIDTH * COLS as f32, y, 3.0, BLACK);
        }

        for c in 0..=COLS {
            let x : f32 = c as f32 * CELL_WIDTH;
            draw_line(x, 0.0, x, CELL_HEIGHT * ROWS as f32, 3.0, BLACK);
        }

        next_frame().await
    }
}

fn bfs_reveal_cells(grid: &mut [[Cell; COLS]; ROWS], start_r: usize, start_c: usize) {
    // invariant1: when calling this we must guarantee that we start with a safe cell
    // invariant2: all items added to queue have 0 neighbouring bombs
    // Note: this may reveal flagged cells but shouldnt matter (we might reveal cells with flagged = true)

    use std::collections::VecDeque;
    let mut q: VecDeque<(usize, usize)> = VecDeque::new();
    grid[start_r][start_c].revealed = true;
    if grid[start_r][start_c].number == 0 {
        q.push_back((start_r, start_c));
    }

    while !q.is_empty() {
        let (r, c) = q.pop_front().unwrap();
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if !bounds_check(nr, nc) {
                    continue;
                }

                let nr = nr as usize;
                let nc = nc as usize;

                // check if not revealed only, this acts as our visited set, and termination condition to never revisit.
                if !grid[nr][nc].revealed {
                    grid[nr][nc].revealed = true;
                    if grid[nr][nc].number == 0 {
                        q.push_back((nr, nc));
                    }
                }
            }
        }
    }
}

fn reveal_all_bombs(grid: &mut [[Cell; COLS]; ROWS]) {
    // can add animation to this later
    for r in 0..ROWS {
        for c in 0..COLS {
            if grid[r][c].bomb {
                grid[r][c].revealed = true;
            }
        }
    }
}

fn compute_bomb_neighbours(grid: &mut [[Cell; COLS]; ROWS]) {
    for r in 0..ROWS {
        for c in 0..COLS {
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }

                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;

                    if !bounds_check(nr, nc) {
                        continue;
                    }

                    if grid[nr as usize][nc as usize].bomb {
                        grid[r][c].number += 1;
                    }
                }
            }
        }
    }
}

fn bounds_check(r: i32, c: i32) -> bool {
    if r < 0 || r >= ROWS as i32 || c < 0 || c >= COLS as i32 {
        false
    } else {
        true
    }
}

fn generate_bombs(grid: &mut [[Cell; COLS]; ROWS], n: usize, exclude_r: usize, exclude_c: usize) {
    assert!(n < ROWS* COLS, "The current grid size requires less than {} bombs.", ROWS * COLS);

    // generate n bombs excluding clicked location
    for _ in 0..n {
        loop {
            let r = gen_range(0, ROWS);
            let c = gen_range(0, COLS);
            if r == exclude_r && c == exclude_c {
                continue;
            }

            if !grid[r][c].bomb {
                grid[r][c].bomb = true;
                break;
            }
        }
    }
}