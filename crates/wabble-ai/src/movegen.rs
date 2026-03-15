use wabble_dict::gaddag::{GaddagNode, SEPARATOR};
use wabble_dict::{FstDictionary, Gaddag};
use wabble_words::board::{BOARD_SIZE, Board};
use wabble_words::placement::PlacedTile;
use wabble_words::tile::{Tile, letter_points};

/// A candidate move: tiles to place and the score it would achieve.
#[derive(Debug, Clone)]
pub struct CandidateMove {
    pub tiles: Vec<PlacedTile>,
    pub score: i32,
    pub primary_word: String,
}

/// Cross-check: which letters are valid at a given empty square, considering
/// perpendicular words that would be formed.
type CrossChecks = [[Option<u32>; BOARD_SIZE]; BOARD_SIZE];

/// Bit set of valid letters (bits 0-25 for A-Z).
fn letter_bit(ch: char) -> u32 {
    let upper = ch.to_ascii_uppercase() as u8;
    if !upper.is_ascii_uppercase() {
        return 0;
    }
    1 << (upper - b'A')
}

fn all_letters() -> u32 {
    (1u32 << 26) - 1
}

fn bit_has(set: u32, ch: char) -> bool {
    set & letter_bit(ch) != 0
}

/// Compute cross-checks for all empty squares on the board.
/// For each empty square, determines which letters can legally be placed there
/// (considering perpendicular words formed).
fn compute_cross_checks(board: &Board, dict: &FstDictionary) -> CrossChecks {
    let mut checks = [[None; BOARD_SIZE]; BOARD_SIZE];

    for (row, row_checks) in checks.iter_mut().enumerate() {
        for (col, cell) in row_checks.iter_mut().enumerate() {
            if board.is_occupied(row, col) {
                continue;
            }

            let h_set = cross_check_direction(board, dict, row, col, true);
            let v_set = cross_check_direction(board, dict, row, col, false);
            *cell = Some(h_set & v_set);
        }
    }

    checks
}

/// Check which letters are valid at (row, col) considering the perpendicular word
/// in the given direction. `horizontal` means we're checking what happens when
/// we extend vertically through this square (i.e., the cross-word is vertical).
fn cross_check_direction(
    board: &Board,
    dict: &FstDictionary,
    row: usize,
    col: usize,
    horizontal: bool,
) -> u32 {
    // Collect letters above/left and below/right in the perpendicular direction
    let (prefix, suffix) = if horizontal {
        // Checking the vertical cross-word at this column
        let mut above = Vec::new();
        let mut r = row;
        while r > 0 {
            r -= 1;
            if let Some(t) = board.get(r, col) {
                above.push(t.letter().unwrap_or('?'));
            } else {
                break;
            }
        }
        above.reverse();

        let mut below = Vec::new();
        let mut r = row + 1;
        while r < BOARD_SIZE {
            if let Some(t) = board.get(r, col) {
                below.push(t.letter().unwrap_or('?'));
            } else {
                break;
            }
            r += 1;
        }
        (above, below)
    } else {
        // Checking the horizontal cross-word at this row
        let mut left = Vec::new();
        let mut c = col;
        while c > 0 {
            c -= 1;
            if let Some(t) = board.get(row, c) {
                left.push(t.letter().unwrap_or('?'));
            } else {
                break;
            }
        }
        left.reverse();

        let mut right = Vec::new();
        let mut c = col + 1;
        while c < BOARD_SIZE {
            if let Some(t) = board.get(row, c) {
                right.push(t.letter().unwrap_or('?'));
            } else {
                break;
            }
            c += 1;
        }
        (left, right)
    };

    // If no adjacent tiles in this direction, all letters are valid
    if prefix.is_empty() && suffix.is_empty() {
        return all_letters();
    }

    // Try each letter and see if the formed word is in the dictionary
    let mut valid = 0u32;
    for ch in b'A'..=b'Z' {
        let letter = ch as char;
        let word: String = prefix
            .iter()
            .chain(std::iter::once(&letter))
            .chain(suffix.iter())
            .collect();
        if dict.contains(&word) {
            valid |= letter_bit(letter);
        }
    }
    valid
}

/// Find anchor squares. An anchor is an empty square adjacent to at least one
/// occupied square. On an empty board, only the center is an anchor.
fn find_anchors(board: &Board) -> Vec<(usize, usize)> {
    if board.is_empty() {
        return vec![(7, 7)];
    }

    let mut anchors = Vec::new();
    for row in 0..BOARD_SIZE {
        for col in 0..BOARD_SIZE {
            if board.is_occupied(row, col) {
                continue;
            }
            let adjacent = [
                (row.wrapping_sub(1), col),
                (row + 1, col),
                (row, col.wrapping_sub(1)),
                (row, col + 1),
            ];
            if adjacent
                .iter()
                .any(|&(r, c)| r < BOARD_SIZE && c < BOARD_SIZE && board.is_occupied(r, c))
            {
                anchors.push((row, col));
            }
        }
    }
    anchors
}

/// Generate all valid moves for the given rack on the board.
pub fn generate_moves(
    board: &Board,
    rack: &[Tile],
    gaddag: &Gaddag,
    dict: &FstDictionary,
) -> Vec<CandidateMove> {
    let cross_checks = compute_cross_checks(board, dict);
    let anchors = find_anchors(board);

    let mut rack_letters: Vec<Option<char>> = rack
        .iter()
        .map(|t| match t {
            Tile::Letter(ch, _) => Some(*ch),
            Tile::Blank(_) => None, // None means blank
        })
        .collect();

    let mut moves = Vec::new();

    for &(anchor_row, anchor_col) in &anchors {
        // Generate horizontal moves through this anchor
        gen_moves_at_anchor(
            board,
            &cross_checks,
            gaddag,
            dict,
            &mut rack_letters,
            anchor_row,
            anchor_col,
            true,
            &mut moves,
        );
        // Generate vertical moves through this anchor
        gen_moves_at_anchor(
            board,
            &cross_checks,
            gaddag,
            dict,
            &mut rack_letters,
            anchor_row,
            anchor_col,
            false,
            &mut moves,
        );
    }

    // Deduplicate moves (same tiles at same positions)
    dedup_moves(&mut moves);
    moves
}

/// Generate moves at a specific anchor in one direction.
#[allow(clippy::too_many_arguments)]
fn gen_moves_at_anchor(
    board: &Board,
    cross_checks: &CrossChecks,
    gaddag: &Gaddag,
    dict: &FstDictionary,
    rack: &mut Vec<Option<char>>,
    anchor_row: usize,
    anchor_col: usize,
    horizontal: bool,
    moves: &mut Vec<CandidateMove>,
) {
    // Determine how far left/up we can extend from the anchor
    let (row, col) = (anchor_row, anchor_col);

    // Count existing tiles immediately before the anchor (these form a forced prefix)
    let mut existing_prefix = Vec::new();
    let mut pos = if horizontal { col } else { row };
    while pos > 0 {
        pos -= 1;
        let (r, c) = if horizontal { (row, pos) } else { (pos, col) };
        if board.is_occupied(r, c) {
            let letter = board.get(r, c).unwrap().letter().unwrap_or('?');
            existing_prefix.push(letter);
        } else {
            break;
        }
    }
    existing_prefix.reverse();

    if !existing_prefix.is_empty() {
        // There's an existing prefix before this anchor. Follow it in the GADDAG
        // then extend right/down from the anchor.
        let mut path: Vec<u8> = Vec::new();
        // GADDAG prefix entry: reversed prefix then separator
        // e.g., if existing is [C, A] and anchor starts at R:
        // we follow A, C, >, then extend with rack tiles
        for &ch in existing_prefix.iter().rev() {
            path.push(ch.to_ascii_uppercase() as u8);
        }
        path.push(SEPARATOR);

        if let Some(node) = gaddag.follow_path(&path) {
            // Now extend right/down from the anchor using rack tiles
            let anchor_pos = if horizontal { col } else { row };
            extend_right(
                board,
                cross_checks,
                dict,
                rack,
                node,
                anchor_row,
                anchor_col,
                horizontal,
                anchor_pos,
                &mut Vec::new(),
                &existing_prefix,
                moves,
            );
        }
    } else {
        // No existing prefix. We place tiles starting at the anchor and optionally
        // extending left. max_left = 1 (anchor only) + available empty squares to the left.
        let anchor_pos = if horizontal { col } else { row };
        // Count available squares to the left of the anchor
        let mut left_avail = 0;
        let mut p = anchor_pos;
        while p > 0 {
            p -= 1;
            let (r, c) = if horizontal { (row, p) } else { (p, col) };
            if board.is_occupied(r, c) {
                break;
            }
            // Don't cross another anchor
            let is_anchor = {
                let adj = [
                    (r.wrapping_sub(1), c),
                    (r + 1, c),
                    (r, c.wrapping_sub(1)),
                    (r, c + 1),
                ];
                !board.is_occupied(r, c)
                    && adj.iter().any(|&(ar, ac)| {
                        ar < BOARD_SIZE && ac < BOARD_SIZE && board.is_occupied(ar, ac)
                    })
            };
            if is_anchor && (r, c) != (anchor_row, anchor_col) {
                break;
            }
            left_avail += 1;
        }
        // left_count=0 is the anchor itself, so max_left = 1 + left_avail
        let max_left = 1 + left_avail;

        let root = gaddag.root();
        gen_left(
            board,
            cross_checks,
            gaddag,
            dict,
            rack,
            root,
            anchor_row,
            anchor_col,
            horizontal,
            anchor_pos,
            0,
            max_left,
            &mut Vec::new(),
            &mut Vec::new(),
            moves,
        );
    }
}

/// Recursively extend leftward/upward from the anchor, building the GADDAG prefix.
#[allow(clippy::too_many_arguments, clippy::only_used_in_recursion)]
fn gen_left(
    board: &Board,
    cross_checks: &CrossChecks,
    gaddag: &Gaddag,
    dict: &FstDictionary,
    rack: &mut Vec<Option<char>>,
    node: GaddagNode<'_>,
    anchor_row: usize,
    anchor_col: usize,
    horizontal: bool,
    anchor_pos: usize,
    left_count: usize,
    max_left: usize,
    placed: &mut Vec<PlacedTile>,
    prefix_letters: &mut Vec<char>,
    moves: &mut Vec<CandidateMove>,
) {
    // Try to cross the separator and extend right from anchor+1
    // (the anchor position itself is occupied by the first tile placed in gen_left)
    if let Some(sep_node) = node.follow(SEPARATOR) {
        let right_start = anchor_pos + 1;
        extend_right(
            board,
            cross_checks,
            dict,
            rack,
            sep_node,
            anchor_row,
            anchor_col,
            horizontal,
            right_start,
            placed,
            prefix_letters,
            moves,
        );
    }

    if left_count >= max_left {
        return;
    }

    // Try each letter from the rack at this position.
    // left_count=0 places at anchor_pos (the anchor letter in GADDAG terms),
    // left_count=1 places at anchor_pos-1, etc.
    let left_pos = anchor_pos - left_count;
    let (r, c) = if horizontal {
        (anchor_row, left_pos)
    } else {
        (left_pos, anchor_col)
    };

    // Check cross-checks for this position
    let cc = cross_checks[r][c].unwrap_or(all_letters());

    let rack_len = rack.len();
    for i in 0..rack_len {
        let tile = rack[i];
        match tile {
            None => {
                // Blank tile: try each letter
                for ch in b'A'..=b'Z' {
                    let letter = ch as char;
                    if !bit_has(cc, letter) {
                        continue;
                    }
                    if let Some(next) = node.follow(ch) {
                        rack[i] = Some('\0'); // mark as used (sentinel)
                        placed.push(PlacedTile {
                            row: r,
                            col: c,
                            tile: Tile::Blank(Some(letter)),
                        });
                        prefix_letters.push(letter);

                        gen_left(
                            board,
                            cross_checks,
                            gaddag,
                            dict,
                            rack,
                            next,
                            anchor_row,
                            anchor_col,
                            horizontal,
                            anchor_pos,
                            left_count + 1,
                            max_left,
                            placed,
                            prefix_letters,
                            moves,
                        );

                        prefix_letters.pop();
                        placed.pop();
                        rack[i] = None; // restore blank
                    }
                }
            }
            Some(letter) => {
                if !bit_has(cc, letter) {
                    continue;
                }
                let upper = letter.to_ascii_uppercase() as u8;
                if let Some(next) = node.follow(upper) {
                    let saved = rack[i];
                    rack[i] = Some('\0'); // mark as used
                    placed.push(PlacedTile {
                        row: r,
                        col: c,
                        tile: Tile::Letter(letter.to_ascii_uppercase(), letter_points(letter)),
                    });
                    prefix_letters.push(letter.to_ascii_uppercase());

                    gen_left(
                        board,
                        cross_checks,
                        gaddag,
                        dict,
                        rack,
                        next,
                        anchor_row,
                        anchor_col,
                        horizontal,
                        anchor_pos,
                        left_count + 1,
                        max_left,
                        placed,
                        prefix_letters,
                        moves,
                    );

                    prefix_letters.pop();
                    placed.pop();
                    rack[i] = saved;
                }
            }
        }
    }
}

/// Extend rightward/downward from the current position, placing tiles.
#[allow(clippy::too_many_arguments)]
fn extend_right(
    board: &Board,
    cross_checks: &CrossChecks,
    dict: &FstDictionary,
    rack: &mut Vec<Option<char>>,
    node: GaddagNode<'_>,
    anchor_row: usize,
    anchor_col: usize,
    horizontal: bool,
    current_pos: usize,
    placed: &mut Vec<PlacedTile>,
    prefix_letters: &[char],
    moves: &mut Vec<CandidateMove>,
) {
    let (r, c) = if horizontal {
        (anchor_row, current_pos)
    } else {
        (current_pos, anchor_col)
    };

    if r >= BOARD_SIZE || c >= BOARD_SIZE {
        // Off the board. If we have a complete word, record it.
        if node.is_terminal() && !placed.is_empty() {
            record_move(board, placed, prefix_letters, &[], moves);
        }
        return;
    }

    if board.is_occupied(r, c) {
        // Existing tile on the board - follow it in the GADDAG
        let letter = board.get(r, c).unwrap().letter().unwrap_or('?');
        let upper = letter.to_ascii_uppercase() as u8;
        if let Some(next) = node.follow(upper) {
            let extended_suffix = vec![letter];

            extend_right_with_existing(
                board,
                cross_checks,
                dict,
                rack,
                next,
                anchor_row,
                anchor_col,
                horizontal,
                current_pos + 1,
                placed,
                prefix_letters,
                &extended_suffix,
                moves,
            );
        }
    } else {
        // Empty square - check if current state forms a valid word
        if node.is_terminal() && !placed.is_empty() {
            record_move(board, placed, prefix_letters, &[], moves);
        }

        // Try placing a tile from the rack
        let cc = cross_checks[r][c].unwrap_or(all_letters());
        let rack_len = rack.len();

        for i in 0..rack_len {
            let tile = rack[i];
            if tile == Some('\0') {
                continue; // already used
            }

            match tile {
            None => {
                for ch in b'A'..=b'Z' {
                    let letter = ch as char;
                    if !bit_has(cc, letter) {
                        continue;
                    }
                    if let Some(next) = node.follow(ch) {
                        rack[i] = Some('\0');
                        placed.push(PlacedTile {
                            row: r,
                            col: c,
                            tile: Tile::Blank(Some(letter)),
                        });
                        let mut new_prefix: Vec<char> =
                            prefix_letters.to_vec();
                        new_prefix.push(letter);

                        extend_right(
                            board,
                            cross_checks,
                            dict,
                            rack,
                            next,
                            anchor_row,
                            anchor_col,
                            horizontal,
                            current_pos + 1,
                            placed,
                            &new_prefix,
                            moves,
                        );

                        placed.pop();
                        rack[i] = None;
                    }
                }
            }
            Some(letter) => {
                if !bit_has(cc, letter) {
                    continue;
                }
                let upper = letter.to_ascii_uppercase() as u8;
                if let Some(next) = node.follow(upper) {
                    let saved = rack[i];
                    rack[i] = Some('\0');
                    placed.push(PlacedTile {
                        row: r,
                        col: c,
                        tile: Tile::Letter(
                            letter.to_ascii_uppercase(),
                            letter_points(letter),
                        ),
                    });
                    let mut new_prefix: Vec<char> =
                        prefix_letters.to_vec();
                    new_prefix.push(letter.to_ascii_uppercase());

                    extend_right(
                        board,
                        cross_checks,
                        dict,
                        rack,
                        next,
                        anchor_row,
                        anchor_col,
                        horizontal,
                        current_pos + 1,
                        placed,
                        &new_prefix,
                        moves,
                    );

                    placed.pop();
                    rack[i] = saved;
                }
            }
            }
        }
    }
}

/// Continue extending right when we've hit an existing board tile.
#[allow(clippy::too_many_arguments)]
fn extend_right_with_existing(
    board: &Board,
    cross_checks: &CrossChecks,
    dict: &FstDictionary,
    rack: &mut Vec<Option<char>>,
    node: GaddagNode<'_>,
    anchor_row: usize,
    anchor_col: usize,
    horizontal: bool,
    current_pos: usize,
    placed: &mut Vec<PlacedTile>,
    prefix_letters: &[char],
    existing_suffix: &[char],
    moves: &mut Vec<CandidateMove>,
) {
    let (r, c) = if horizontal {
        (anchor_row, current_pos)
    } else {
        (current_pos, anchor_col)
    };

    if r >= BOARD_SIZE || c >= BOARD_SIZE {
        if node.is_terminal() && !placed.is_empty() {
            record_move(board, placed, prefix_letters, existing_suffix, moves);
        }
        return;
    }

    if board.is_occupied(r, c) {
        let letter = board.get(r, c).unwrap().letter().unwrap_or('?');
        let upper = letter.to_ascii_uppercase() as u8;
        if let Some(next) = node.follow(upper) {
            let mut new_suffix = existing_suffix.to_vec();
            new_suffix.push(letter);
            extend_right_with_existing(
                board,
                cross_checks,
                dict,
                rack,
                next,
                anchor_row,
                anchor_col,
                horizontal,
                current_pos + 1,
                placed,
                prefix_letters,
                &new_suffix,
                moves,
            );
        }
    } else {
        // Empty square after existing tiles
        if node.is_terminal() && !placed.is_empty() {
            record_move(board, placed, prefix_letters, existing_suffix, moves);
        }

        let cc = cross_checks[r][c].unwrap_or(all_letters());
        let rack_len = rack.len();

        for i in 0..rack_len {
            let tile = rack[i];
            if tile == Some('\0') {
                continue;
            }

            match tile {
            None => {
                for ch in b'A'..=b'Z' {
                    let letter = ch as char;
                    if !bit_has(cc, letter) {
                        continue;
                    }
                    if let Some(next) = node.follow(ch) {
                        rack[i] = Some('\0');
                        placed.push(PlacedTile {
                            row: r,
                            col: c,
                            tile: Tile::Blank(Some(letter)),
                        });
                        let mut new_prefix: Vec<char> =
                            prefix_letters.to_vec();
                        new_prefix.extend(existing_suffix);
                        new_prefix.push(letter);

                        extend_right(
                            board,
                            cross_checks,
                            dict,
                            rack,
                            next,
                            anchor_row,
                            anchor_col,
                            horizontal,
                            current_pos + 1,
                            placed,
                            &new_prefix,
                            moves,
                        );

                        placed.pop();
                        rack[i] = None;
                    }
                }
            }
            Some(letter) => {
                if !bit_has(cc, letter) {
                    continue;
                }
                let upper = letter.to_ascii_uppercase() as u8;
                if let Some(next) = node.follow(upper) {
                    let saved = rack[i];
                    rack[i] = Some('\0');
                    placed.push(PlacedTile {
                        row: r,
                        col: c,
                        tile: Tile::Letter(
                            letter.to_ascii_uppercase(),
                            letter_points(letter),
                        ),
                    });
                    let mut new_prefix: Vec<char> =
                        prefix_letters.to_vec();
                    new_prefix.extend(existing_suffix);
                    new_prefix.push(letter.to_ascii_uppercase());

                    extend_right(
                        board,
                        cross_checks,
                        dict,
                        rack,
                        next,
                        anchor_row,
                        anchor_col,
                        horizontal,
                        current_pos + 1,
                        placed,
                        &new_prefix,
                        moves,
                    );

                    placed.pop();
                    rack[i] = saved;
                }
            }
            }
        }
    }
}

/// Record a valid move, computing its score.
fn record_move(
    board: &Board,
    placed: &[PlacedTile],
    prefix_letters: &[char],
    suffix_letters: &[char],
    moves: &mut Vec<CandidateMove>,
) {
    if placed.is_empty() {
        return;
    }

    // Compute the score using the same scoring logic as the game
    let score = score_candidate(board, placed);
    let word: String = prefix_letters.iter().chain(suffix_letters.iter()).collect();

    moves.push(CandidateMove {
        tiles: placed.to_vec(),
        score,
        primary_word: word,
    });
}

/// Score a candidate move (all words formed).
fn score_candidate(board: &Board, placed: &[PlacedTile]) -> i32 {
    use wabble_words::rack::RACK_SIZE;

    // Determine direction
    let horizontal = if placed.len() == 1 {
        true // doesn't matter for scoring, we check both directions
    } else {
        placed[0].row == placed[1].row
    };

    let mut total = 0i32;

    // Score the primary word
    total += score_primary_word(board, placed, horizontal);

    // Score cross-words
    for pt in placed {
        let cross = score_cross_word(board, pt, horizontal);
        total += cross;
    }

    // Bingo bonus
    if placed.len() == RACK_SIZE {
        total += 50;
    }

    total
}

/// Score the primary word along the placement direction.
fn score_primary_word(board: &Board, placed: &[PlacedTile], horizontal: bool) -> i32 {
    use wabble_words::board::BonusSquare;

    // Find extent of the word
    let (fixed, positions): (usize, Vec<usize>) = if horizontal {
        let row = placed[0].row;
        (row, placed.iter().map(|p| p.col).collect())
    } else {
        let col = placed[0].col;
        (col, placed.iter().map(|p| p.row).collect())
    };

    let min_pos = *positions.iter().min().unwrap();
    let max_pos = *positions.iter().max().unwrap();

    // Extend backward
    let mut start = min_pos;
    while start > 0 {
        let (r, c) = if horizontal {
            (fixed, start - 1)
        } else {
            (start - 1, fixed)
        };
        if board.is_occupied(r, c) {
            start -= 1;
        } else {
            break;
        }
    }

    // Extend forward
    let mut end = max_pos;
    while end + 1 < BOARD_SIZE {
        let (r, c) = if horizontal {
            (fixed, end + 1)
        } else {
            (end + 1, fixed)
        };
        if board.is_occupied(r, c) {
            end += 1;
        } else {
            break;
        }
    }

    // If word is only 1 letter, no score for primary
    if start == end && placed.len() == 1 {
        // Single tile placed: primary word might just be this tile
        // Only score if there are adjacent tiles extending it
        let (r, c) = (placed[0].row, placed[0].col);
        let has_neighbor = if horizontal {
            (c > 0 && board.is_occupied(r, c - 1)) || (c + 1 < BOARD_SIZE && board.is_occupied(r, c + 1))
        } else {
            (r > 0 && board.is_occupied(r - 1, c)) || (r + 1 < BOARD_SIZE && board.is_occupied(r + 1, c))
        };
        if !has_neighbor {
            return 0;
        }
    }

    let mut word_score = 0i32;
    let mut word_mult = 1i32;

    for pos in start..=end {
        let (r, c) = if horizontal { (fixed, pos) } else { (pos, fixed) };

        // Check if this is a newly placed tile
        let is_new = placed.iter().any(|p| p.row == r && p.col == c);

        let tile_points = if is_new {
            let pt = placed.iter().find(|p| p.row == r && p.col == c).unwrap();
            pt.tile.points() as i32
        } else {
            board.get(r, c).map(|t| t.points() as i32).unwrap_or(0)
        };

        if is_new {
            let bonus = wabble_words::board::Board::bonus_at(r, c);
            let letter_mult = match bonus {
                BonusSquare::DoubleLetter => 2,
                BonusSquare::TripleLetter => 3,
                _ => 1,
            };
            word_score += tile_points * letter_mult;
            match bonus {
                BonusSquare::DoubleWord | BonusSquare::Center => word_mult *= 2,
                BonusSquare::TripleWord => word_mult *= 3,
                _ => {}
            }
        } else {
            word_score += tile_points;
        }
    }

    word_score * word_mult
}

/// Score a cross-word formed perpendicular to the placed tile.
fn score_cross_word(board: &Board, pt: &PlacedTile, horizontal: bool) -> i32 {
    use wabble_words::board::BonusSquare;

    let (row, col) = (pt.row, pt.col);

    // Look perpendicular to the main direction
    let (start, end) = if horizontal {
        // Cross-word is vertical
        let mut s = row;
        while s > 0 && board.is_occupied(s - 1, col) {
            s -= 1;
        }
        let mut e = row;
        while e + 1 < BOARD_SIZE && board.is_occupied(e + 1, col) {
            e += 1;
        }
        (s, e)
    } else {
        // Cross-word is horizontal
        let mut s = col;
        while s > 0 && board.is_occupied(row, s - 1) {
            s -= 1;
        }
        let mut e = col;
        while e + 1 < BOARD_SIZE && board.is_occupied(row, e + 1) {
            e += 1;
        }
        (s, e)
    };

    // If no cross-word formed (just the tile itself), return 0
    if start == end {
        return 0;
    }

    let mut word_score = 0i32;
    let mut word_mult = 1i32;

    for pos in start..=end {
        let (r, c) = if horizontal {
            (pos, col)
        } else {
            (row, pos)
        };

        if r == row && c == col {
            // This is the newly placed tile
            let bonus = Board::bonus_at(r, c);
            let letter_mult = match bonus {
                BonusSquare::DoubleLetter => 2,
                BonusSquare::TripleLetter => 3,
                _ => 1,
            };
            word_score += pt.tile.points() as i32 * letter_mult;
            match bonus {
                BonusSquare::DoubleWord | BonusSquare::Center => word_mult *= 2,
                BonusSquare::TripleWord => word_mult *= 3,
                _ => {}
            }
        } else {
            let tile_points = board.get(r, c).map(|t| t.points() as i32).unwrap_or(0);
            word_score += tile_points;
        }
    }

    word_score * word_mult
}

fn dedup_moves(moves: &mut Vec<CandidateMove>) {
    moves.sort_by(|a, b| b.score.cmp(&a.score));
    moves.dedup_by(|a, b| {
        if a.tiles.len() != b.tiles.len() {
            return false;
        }
        a.tiles.iter().all(|at| {
            b.tiles
                .iter()
                .any(|bt| at.row == bt.row && at.col == bt.col && at.tile == bt.tile)
        })
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_gaddag() -> (Gaddag, FstDictionary) {
        let words: Vec<&str> = vec![
            "AT", "IT", "TO", "ATE", "EAT", "TEA", "TIE", "CAT", "CAR", "CARE",
            "ACE", "ICE", "TAR", "RAT", "ART", "ARE", "EAR", "ERA", "IRE",
            "THE", "HE", "HI", "HA", "AH", "AN", "IN", "ON", "NO", "OR", "SO",
            "IS", "AS", "DO", "GO", "BE", "ME", "WE",
        ];
        let gaddag_bytes = Gaddag::build(&words).unwrap();
        let gaddag = Gaddag::from_bytes(gaddag_bytes).unwrap();
        let dict_bytes = FstDictionary::build(&words).unwrap();
        let dict = FstDictionary::from_bytes(dict_bytes).unwrap();
        (gaddag, dict)
    }

    #[test]
    fn empty_board_generates_moves() {
        let (gaddag, dict) = build_test_gaddag();
        let board = Board::new();
        let rack = vec![
            Tile::Letter('C', 3),
            Tile::Letter('A', 1),
            Tile::Letter('T', 1),
        ];

        let moves = generate_moves(&board, &rack, &gaddag, &dict);
        assert!(!moves.is_empty(), "should generate moves on empty board");

        // Should find CAT, AT, and others
        let words: Vec<&str> = moves.iter().map(|m| m.primary_word.as_str()).collect();
        assert!(
            moves.iter().any(|m| m.primary_word == "CAT"),
            "should find CAT, found: {:?}",
            words
        );
    }

    #[test]
    fn extends_existing_words() {
        let (gaddag, dict) = build_test_gaddag();
        let mut board = Board::new();
        // Place "AT" on the board
        board.set(7, 7, Tile::Letter('A', 1));
        board.set(7, 8, Tile::Letter('T', 1));

        let rack = vec![
            Tile::Letter('C', 3),
            Tile::Letter('E', 1),
        ];

        let moves = generate_moves(&board, &rack, &gaddag, &dict);
        assert!(!moves.is_empty(), "should find moves extending AT");
    }

    #[test]
    fn cross_checks_filter_invalid() {
        let (_, dict) = build_test_gaddag();
        let mut board = Board::new();
        board.set(7, 7, Tile::Letter('A', 1));
        board.set(7, 8, Tile::Letter('T', 1));

        let checks = compute_cross_checks(&board, &dict);
        // Square at (6, 7) is above 'A' - only letters that form valid 2-letter words
        // ending in A should be valid
        let cc = checks[6][7].unwrap();
        assert_ne!(cc, 0, "some letters should be valid above A");
    }

    #[test]
    fn scoring_includes_bonuses() {
        let board = Board::new();
        // Place on center (double word)
        let placed = vec![
            PlacedTile {
                row: 7,
                col: 7,
                tile: Tile::Letter('A', 1),
            },
            PlacedTile {
                row: 7,
                col: 8,
                tile: Tile::Letter('T', 1),
            },
        ];
        let score = score_candidate(&board, &placed);
        // A(1) + T(1) = 2, center is double word = 4
        assert_eq!(score, 4);
    }
}
