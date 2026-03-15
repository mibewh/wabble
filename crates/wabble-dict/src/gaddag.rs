use fst::raw::Fst;

/// Separator byte in GADDAG entries. Separates the reversed prefix from
/// the remaining suffix. Chosen to sort before all uppercase letters.
pub const SEPARATOR: u8 = b'>';

/// A GADDAG (Graph Directed Acyclic Word Graph) stored as an FST.
///
/// For each word of length n, the GADDAG contains n entries. For example,
/// "CARE" produces:
///   - C>ARE  (anchor at C, read forward)
///   - AC>RE  (anchor at A, reversed prefix AC, then forward RE)
///   - RAC>E  (anchor at R, reversed prefix RAC, then forward E)
///   - ERAC>  (anchor at E, fully reversed)
///
/// This allows efficient enumeration of all words passing through any
/// anchor square during AI move generation.
pub struct Gaddag {
    fst: Fst<Vec<u8>>,
}

impl Gaddag {
    /// Load a pre-built GADDAG from FST bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, fst::Error> {
        let fst = Fst::new(bytes)?;
        Ok(Self { fst })
    }

    /// Build GADDAG FST bytes from a list of words. Words are normalized to
    /// uppercase. Returns raw FST bytes for saving to disk.
    pub fn build(words: &[&str]) -> Result<Vec<u8>, fst::Error> {
        let mut entries: Vec<Vec<u8>> = Vec::new();

        for word in words {
            let word = word.trim().to_ascii_uppercase();
            if !word.chars().all(|c| c.is_ascii_alphabetic()) {
                continue;
            }
            let bytes: Vec<u8> = word.into_bytes();
            let n = bytes.len();
            if n < 2 {
                continue;
            }

            for i in 0..n {
                let mut entry = Vec::with_capacity(n + 1);
                // Reversed prefix: bytes[i], bytes[i-1], ..., bytes[0]
                for j in (0..=i).rev() {
                    entry.push(bytes[j]);
                }
                entry.push(SEPARATOR);
                // Remaining suffix: bytes[i+1], ..., bytes[n-1]
                for &b in &bytes[(i + 1)..n] {
                    entry.push(b);
                }
                entries.push(entry);
            }
        }

        entries.sort();
        entries.dedup();

        let mut builder = fst::SetBuilder::memory();
        for entry in &entries {
            builder.insert(entry)?;
        }
        builder.into_inner()
    }

    /// Get the root node for traversal.
    pub fn root(&self) -> GaddagNode<'_> {
        GaddagNode {
            node: self.fst.root(),
            fst: &self.fst,
        }
    }

    /// Traverse from root following a sequence of bytes.
    pub fn follow_path(&self, path: &[u8]) -> Option<GaddagNode<'_>> {
        let mut node = self.fst.root();
        for &byte in path {
            let idx = node.find_input(byte)?;
            let trans = node.transition(idx);
            node = self.fst.node(trans.addr);
        }
        Some(GaddagNode {
            node,
            fst: &self.fst,
        })
    }
}

/// A node in the GADDAG, used for traversal during move generation.
pub struct GaddagNode<'a> {
    node: fst::raw::Node<'a>,
    fst: &'a Fst<Vec<u8>>,
}

impl<'a> GaddagNode<'a> {
    /// Whether this node marks the end of a complete GADDAG entry (valid word).
    pub fn is_terminal(&self) -> bool {
        self.node.is_final()
    }

    /// Follow an edge labeled with the given byte.
    pub fn follow(&self, byte: u8) -> Option<GaddagNode<'a>> {
        let idx = self.node.find_input(byte)?;
        let trans = self.node.transition(idx);
        Some(GaddagNode {
            node: self.fst.node(trans.addr),
            fst: self.fst,
        })
    }

    /// Get all outgoing transitions from this node.
    pub fn transitions(&self) -> Vec<(u8, GaddagNode<'a>)> {
        (0..self.node.len())
            .map(|i| {
                let trans = self.node.transition(i);
                (
                    trans.inp,
                    GaddagNode {
                        node: self.fst.node(trans.addr),
                        fst: self.fst,
                    },
                )
            })
            .collect()
    }

    /// Check if this node has a transition for the given byte.
    pub fn has_edge(&self, byte: u8) -> bool {
        self.node.find_input(byte).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_and_traverse() {
        let words = vec!["CAT", "CAR", "CARE"];
        let bytes = Gaddag::build(&words).unwrap();
        let gaddag = Gaddag::from_bytes(bytes).unwrap();

        // "CAT" produces entries: C>AT, AC>T, TAC>
        // Starting at root, follow 'C', should exist
        let c_node = gaddag.root().follow(b'C').unwrap();
        // From C, follow '>' to get words starting with C
        let sep_node = c_node.follow(SEPARATOR).unwrap();
        // From C>, follow 'A' for CA...
        let a_node = sep_node.follow(b'A').unwrap();
        // From C>A, follow 'T' for CAT
        let t_node = a_node.follow(b'T').unwrap();
        assert!(t_node.is_terminal());
        // From C>A, follow 'R' for CAR
        let r_node = a_node.follow(b'R').unwrap();
        assert!(r_node.is_terminal());
        // From C>AR, follow 'E' for CARE
        let e_node = r_node.follow(b'E').unwrap();
        assert!(e_node.is_terminal());
    }

    #[test]
    fn reversed_prefix_traversal() {
        let words = vec!["CAT"];
        let bytes = Gaddag::build(&words).unwrap();
        let gaddag = Gaddag::from_bytes(bytes).unwrap();

        // Entry: TAC> (fully reversed)
        let node = gaddag.follow_path(&[b'T', b'A', b'C', SEPARATOR]);
        assert!(node.is_some());
        assert!(node.unwrap().is_terminal());

        // Entry: AC>T
        let node = gaddag.follow_path(&[b'A', b'C', SEPARATOR, b'T']);
        assert!(node.is_some());
        assert!(node.unwrap().is_terminal());
    }

    #[test]
    fn transitions_enumeration() {
        let words = vec!["CAT", "COT", "CUT"];
        let bytes = Gaddag::build(&words).unwrap();
        let gaddag = Gaddag::from_bytes(bytes).unwrap();

        // From root, should have edges for multiple letters
        let root_transitions = gaddag.root().transitions();
        assert!(!root_transitions.is_empty());

        // C> should lead to A, O, U
        let c_node = gaddag.root().follow(b'C').unwrap();
        let sep_node = c_node.follow(SEPARATOR).unwrap();
        let children: Vec<u8> = sep_node.transitions().iter().map(|(b, _)| *b).collect();
        assert!(children.contains(&b'A'));
        assert!(children.contains(&b'O'));
        assert!(children.contains(&b'U'));
    }

    #[test]
    fn case_insensitive_build() {
        let words = vec!["cat", "Cat", "CAT"];
        let bytes = Gaddag::build(&words).unwrap();
        let gaddag = Gaddag::from_bytes(bytes).unwrap();

        // Should work the same regardless of input case
        let node = gaddag.follow_path(&[b'C', SEPARATOR, b'A', b'T']);
        assert!(node.is_some());
        assert!(node.unwrap().is_terminal());
    }
}
