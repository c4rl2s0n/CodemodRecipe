use crate::dump::Position;

/// Convert a UTF-8 byte offset to 0-based line/column (column in UTF-8 bytes).
pub fn byte_to_position(source: &str, byte: usize) -> Position {
    let byte = byte.min(source.len());
    let mut line = 0u32;
    let mut col = 0u32;
    for (i, b) in source.as_bytes().iter().enumerate() {
        if i >= byte {
            break;
        }
        if *b == b'\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Position {
        byte: byte as u32,
        line,
        column: col,
    }
}

pub fn positions_for_range(source: &str, start: usize, end: usize) -> (Position, Position) {
    (
        byte_to_position(source, start),
        byte_to_position(source, end),
    )
}
