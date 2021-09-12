mod utils;

use std::fmt;
use wasm_bindgen::prelude::*;

// use the js_sys create the access the JS Math functions
extern crate js_sys;
extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// This is where JS functions are imported from extern "C"
// The wasm_bindgen macro is used to signify what we want to import from JS or export to JS
#[wasm_bindgen]
// Import the `window.alert` function from the Web.
extern "C" {
    fn alert(s: &str);
}

// This is where the functions and type definitions are defined to be used in JS
#[wasm_bindgen]
// We represent the values of each cells as a u8 type so that it is a single byte
#[repr(u8)]
// We derive a number of attributes we can use on this enum
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    // We set the dead and alive variants to 0 and 1 respectivly so that we can count a cells neighbors
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

// We create a struct which defines the universe consisting of the width and height as u32 types
// and cells which is a vector of cells of lenght width * height
#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

// Implement functions for the Universe struct
impl Universe {
    // Translate the row and column into an index so that we can access any given cell
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    // Count the number of live neighbors for any given cell
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        // Loop over all the rows in the universe
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            // Loop over all the columns in the universe
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }
                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

// Implement the game rules as a match statement and make it availble to JS via the wasm_bindgen macro within the Universe struct
#[wasm_bindgen]
impl Universe {
    // Check the game rules for every tick of the game
    pub fn tick(&mut self) {
        let mut next = self.cells.clone();
        // Loop over all rows in the universe
        for row in 0..self.height {
            // Loop over all columns in the universe
            for col in 0..self.width {
                // Get the index for the cell via the get_index function
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                // Get the live neightbor count for the cell
                let live_neighbors = self.live_neighbor_count(row, col);

                // Check if the cell should be dead or alive
                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => Cell::Dead,
                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => Cell::Alive,
                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => Cell::Dead,
                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => Cell::Alive,
                    // All other cells remain in the same state.
                    (otherwise, _) => otherwise,
                };

                next[idx] = next_cell;
            }
        }
        self.cells = next;
    }

    // Constructor method in order to initialize the universe
    pub fn new(width: u32, height: u32) -> Universe {
        // Log to the browser console that a new universe has been created
        log!("New universe created");
        utils::set_panic_hook();
        // Create a 64x64 grid universe
        // let width = 64;
        // let height = 64;
        // Loop over all cells and assign them either a dead or alive state
        let cells = (0..width * height)
            .map(|_i| {
                // Use the js_sys crate in order to randomly assign dead or alive to a cell
                if js_sys::Math::random() < 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    // Getter function to return the width to be used in JS
    pub fn width(&self) -> u32 {
        self.width
    }

    // Getter function to return the height to be used in JS
    pub fn height(&self) -> u32 {
        self.height
    }

    // Getter function to return the cells to be used in JS
    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    /// Set the width of the universe.
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    /// Set the height of the universe.
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    // Render function which JS can use to render the universe
    pub fn render(&self) -> String {
        self.to_string()
    }

    // Toggle the state of a cell from dead to alive and vice versa
    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn reset(&mut self) {
        self.cells = (0..self.width * self.height)
            .map(|_i| {
                // Use the js_sys crate in order to randomly assign dead or alive to a cell
                if js_sys::Math::random() < 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();
    }
}

// Implement the standard display trait in order to represent the universe in a human readable way
impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                // If a cell is dead print a ◻ and if a cell is alive print a ◼
                let symbol = if cell == Cell::Dead { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}
