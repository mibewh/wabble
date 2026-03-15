use wabble_words::tile::{Tile, letter_points};

/// Evaluate the "leave" value of remaining rack tiles after a move.
/// Better leaves have a balanced mix of vowels and consonants,
/// high-value flexible letters, and avoid duplicates.
pub fn evaluate_leave(remaining: &[Tile]) -> f32 {
    if remaining.is_empty() {
        return 10.0; // Using all tiles is great
    }

    let mut score = 0.0f32;
    let mut vowels = 0;
    let mut consonants = 0;

    for tile in remaining {
        match tile {
            Tile::Blank(_) => {
                score += 25.0; // Blanks are very valuable to keep
            }
            Tile::Letter(ch, _) => {
                let ch = ch.to_ascii_uppercase();
                let pts = letter_points(ch);
                // Prefer keeping flexible, common letters
                score += match ch {
                    'S' => 8.0,  // S is very flexible (plurals)
                    'E' => 5.0,
                    'R' | 'T' | 'N' | 'L' => 3.0,
                    'A' | 'I' | 'O' => 2.0,
                    'D' | 'G' => 1.0,
                    'Q' => -8.0, // Q without U is bad
                    'V' => -3.0,
                    'U' => 1.5,
                    _ if pts >= 8 => -2.0, // High-point tiles are hard to play
                    _ => 0.0,
                };

                if "AEIOU".contains(ch) {
                    vowels += 1;
                } else {
                    consonants += 1;
                }
            }
        }
    }

    // Penalize imbalanced vowel/consonant ratio
    let total = vowels + consonants;
    if total > 0 {
        let ratio = vowels as f32 / total as f32;
        // Ideal ratio is ~0.4 (2 vowels, 3 consonants in a 5-tile leave)
        let deviation = (ratio - 0.4).abs();
        score -= deviation * 10.0;
    }

    // Penalize duplicate letters
    let mut seen = [0u8; 26];
    for tile in remaining {
        if let Tile::Letter(ch, _) = tile {
            let idx = (ch.to_ascii_uppercase() as u8 - b'A') as usize;
            seen[idx] += 1;
            if seen[idx] > 1 {
                score -= 3.0 * (seen[idx] - 1) as f32;
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_leave_is_valuable() {
        let with_blank = vec![Tile::Blank(None)];
        let without = vec![Tile::Letter('V', 4)];
        assert!(evaluate_leave(&with_blank) > evaluate_leave(&without));
    }

    #[test]
    fn s_leave_is_good() {
        let with_s = vec![Tile::Letter('S', 1)];
        let with_v = vec![Tile::Letter('V', 4)];
        assert!(evaluate_leave(&with_s) > evaluate_leave(&with_v));
    }

    #[test]
    fn empty_leave_is_best() {
        let empty: Vec<Tile> = vec![];
        let some = vec![Tile::Letter('A', 1)];
        assert!(evaluate_leave(&empty) > evaluate_leave(&some));
    }
}
