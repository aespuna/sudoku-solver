//! Sudoku solver
//! Based on this excelent writeup <http://norvig.com/sudoku.html>

use std::convert::TryFrom;
use std::fmt;
use std::time;

/// `Possible` stores all the possible values that can go on a square,
/// from 1 to 9.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Possible(u16);

impl Possible {
    fn new() -> Self {
        // All 9 values are possible by default
        Self(0x1FF)
    }

    fn len(&self) -> u32 {
        self.0.count_ones()
    }

    fn contains(&self, pos: u8) -> bool {
        (1 << (pos - 1)) & self.0 != 0
    }

    fn remove(&self, pos: u8) -> Self {
        // Although this is technically a flip we never add items to
        // the bitset, only discard, hence the name.  A better version
        // without cloning could reuse this method.
        Self((1 << (pos - 1)) ^ self.0)
    }

    /// Returns an iterator over the values that are set
    fn values(&self) -> impl Iterator<Item = u8> {
        let mask = self.0;
        (1..=9).filter(move |i| (1 << (i - 1)) & mask != 0)
    }

    /// Return the first value or a 0. Very useful if we already know
    /// that there is only one value in the set.
    fn n(&self) -> u8 {
        self.values().next().unwrap_or(0)
    }
}

/// `Values` stores all the possible values for every cell in the
/// sudoku.  Its core is the search function, that uses constraint
/// propagation and backtracking to find a possible solution to the
/// sudoku.
#[derive(Clone, Debug)]
struct Values(Vec<Possible>);

impl Values {
    fn new() -> Self {
        Values(vec![Possible::new(); 81])
    }

    fn search(self) -> Option<Self> {
        if self.0.iter().all(|p| p.len() == 1) {
            // Already solved
            return Some(self);
        }

        // Find the first square with the least options
        // This way the probability to correctly "guess" is higher
        // If we later find that there was a contradiction, we removed
        // the most of we can of the possibilities.

        // We can unwrap safely because at least 1 such
        // square exists
        let (_, cell) = self
            .0
            .iter()
            .enumerate()
            .filter(|(_, p)| p.len() > 1)
            .map(|(i, p)| (p.len(), i))
            .min()
            .unwrap();

        // Return the first found solution (if any) while trying to assign
        // the possible values for that cell
        self.0[cell]
            .values()
            .filter_map(|n| self.clone().assign(n, cell)?.search())
            .next()
    }

    fn assign(self, digit: u8, cell: usize) -> Option<Self> {
        let mut values = self.clone();

        // Eliminates all the other possibilities from this cell
        for other_digit in self.0[cell].values().filter(|&d| d != digit) {
            values = values.eliminate(other_digit, cell)?
        }
        Some(values)
    }

    fn eliminate(self, digit: u8, cell: usize) -> Option<Self> {
        let mut possibles = self.0[cell];

        if !possibles.contains(digit) {
            // Was already removed
            return Some(self);
        }

        possibles = possibles.remove(digit);

        let mut values = self.clone();
        values.0[cell] = possibles;

        match possibles.len() {
            0 => {
                // No possible values left: contradiction
                return None;
            }
            1 => {
                // If only one possibility left, eliminate it as a possibility
                // from all its peers
                let d = possibles.n();
                for peer in Sudoku::peers(cell as u8) {
                    values = values.eliminate(d, peer as usize)?
                }
            }
            _ => {}
        }

        // Check if for any unit, this digit can only appear in one
        // cell, if so, assign it to that cell
        for unit in Sudoku::units(cell as u8) {
            let places_for_d: Vec<u8> = unit
                .into_iter()
                .filter(|&p| values.0[p as usize].contains(digit))
                .collect();

            match places_for_d.len() {
                0 => return None,
                1 => {
                    values = values.assign(digit, places_for_d[0] as usize)?;
                }
                _ => {}
            };
        }

        Some(values)
    }
}

/// `Sudoku` contains a sudoku puzzle.
/// Can parse from strings to puzzles and display itself.
/// When calling solve, leverages to `Values::search`.
struct Sudoku([u8; 81]);

impl Sudoku {
    fn solve(&mut self) -> bool {
        let mut values = Values::new();
        for (i, &v) in self.0.iter().enumerate().filter(|(_, &v)| v != 0) {
            if let Some(v) = values.assign(v, i) {
                values = v;
            } else {
                return false;
            }
        }

        if let Some(values) = values.search() {
            for (i, &v) in values.0.iter().enumerate() {
                self.0[i] = v.n();
            }
            true
        } else {
            // We did not find a solution
            false
        }
    }

    /// Iterator containing the cell indices of the row in which
    /// `cell` is
    fn row(cell: u8) -> impl Iterator<Item = u8> {
        let row = cell / 9;
        (0..9).map(move |r| row * 9 + r).filter(move |&r| r != cell)
    }

    /// Iterator containing the cell indices of the column in which
    /// `cell` is
    fn column(cell: u8) -> impl Iterator<Item = u8> {
        let column = cell % 9;

        (0..9)
            .map(move |c| c * 9 + column)
            .filter(move |&c| c != cell)
    }

    /// Iterator containing the cell indices of the square in which
    /// `cell` is.
    fn square(cell: u8) -> impl Iterator<Item = u8> {
        let (row, column) = (cell / 9, cell % 9);
        let (r, c) = (row / 3, column / 3);

        (0..9)
            .map(move |n| 3 * (9 * r + c + 3 * (n / 3)) + n % 3)
            .filter(move |&t| t != cell)
    }

    fn units(i: u8) -> Vec<Vec<u8>> {
        vec![
            Sudoku::row(i).collect(),
            Sudoku::column(i).collect(),
            Sudoku::square(i).collect(),
        ]
    }

    fn peers(i: u8) -> impl Iterator<Item = u8> {
        Sudoku::row(i)
            .chain(Sudoku::column(i))
            .chain(Sudoku::square(i))
    }
}

impl fmt::Display for Sudoku {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = String::new();
        for (i, &n) in self.0.iter().enumerate() {
            if i != 0 && i % 9 == 0 {
                buffer.push_str("|\n");
            }

            if i % 27 == 0 {
                buffer.push_str("+------+------+------+\n");
            }

            if i % 3 == 0 {
                buffer.push_str("|");
            }

            if n != 0 {
                buffer.push_str(&(n.to_string() + " "));
            } else {
                buffer.push_str(". ");
            }
        }
        buffer.push_str("|\n+------+------+------+\n");
        write!(f, "{}", buffer)
    }
}

impl TryFrom<&str> for Sudoku {
    type Error = &'static str;

    /// We expect to read 81 grid data between digits and `.`s.
    /// A dot (`.`) or a `0` means that that particular cell is empty.
    /// All other non-digit values are ignored.
    /// If a grid can not be read, an Err is returned.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut grid = [0u8; 81];

        let mut i = 0;
        for c in value.chars() {
            if i > 80 {
                // No need to read more
                break;
            }

            if c == '.' {
                // We leave the 0 in place and count it as a digit
                i += 1;
                continue;
            }

            if let Some(d) = c.to_digit(10) {
                // If parsing the digit fails (and is not a `.`), we
                // ignore it
                grid[i] = d as u8;
                i += 1;
            }
        }

        if i == 81 {
            Ok(Self(grid))
        } else {
            Err("malformed grid")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_possible() {
        assert_eq!(Possible(0x3).len(), 2);
        assert_eq!(Possible(0x6).len(), 2);
        assert_eq!(Possible(0x1).len(), 1);
        assert_eq!(Possible::new().len(), 9);

        assert_eq!(Possible(0x6).contains(3), true);
        assert_eq!(Possible(0x6).contains(1), false);
        assert_eq!(Possible(0x8).contains(8), false);

        assert_eq!(Possible(0x8).remove(4), Possible(0x0));
        assert_eq!(Possible(0xF).remove(4), Possible(0x7));
    }
}

/// Read puzzles from stdin separated by an empty line, and solve them
fn main() -> Result<(), &'static str> {
    let mut buff = String::new();
    let mut puzzle = String::new();
    while let Ok(n) = std::io::stdin().read_line(&mut buff) {
        if n == 0 {
            // EOF read
            break;
        }

        if buff.trim().is_empty() && !puzzle.trim().is_empty() {
            let mut sudoku = Sudoku::try_from(puzzle.as_ref())?;
            println!("{}", sudoku);
            let t0 = time::Instant::now();
            sudoku.solve();
            let dur = time::Instant::now() - t0;
            let t = dur.as_secs() as f64 + dur.subsec_micros() as f64 * 1e-6;
            println!("{}\n({:.6} seconds)\n", sudoku, t);
            puzzle.clear();
        } else {
            puzzle.push_str(&buff);
        }
        buff.clear();
    }

    Ok(())
}
