//! Display characters on the Lumatrix.

pub mod table;

/// A single character that can be drawn on the the display.
///
/// To draw a sequence of characters, see [`convert_str`].
#[derive(Debug, Clone, Copy)]
pub struct Character {
    pub coordinates: &'static [(usize, usize)],
    pub(crate) width: usize,
    pub(crate) offset: usize,
}
impl Character {
    /// Construct a new character from a slice of coordinates.
    ///
    /// The width is specified separately for intended whitespace.
    const fn new(coordinates: &'static [(usize, usize)], width: usize) -> Self {
        Self {
            coordinates,
            width,
            offset: 0,
        }
    }
}
impl Default for Character {
    fn default() -> Self {
        table::SPACE
    }
}

/// Convert a string to an array of characters for display.
///
/// See [`draw_text_billboard_frame`](crate::Lumatrix::draw_text_billboard_frame).
pub fn convert_str<const L: usize>(text: [u8; L]) -> ([Character; L], usize) {
    let mut result = [Character::default(); L];
    let mut length = 0;
    for (i, b) in text.iter().enumerate() {
        let mut c = Character::from(*b);
        c.offset = length;
        length += c.width + 1;
        result[i] = c;
    }
    (result, length - 1)
}

impl From<u8> for Character {
    fn from(value: u8) -> Self {
        match value {
            b'0' => table::ZERO,
            b'1' => table::ONE,
            b'2' => table::TWO,
            b'3' => table::THREE,
            b'4' => table::FOUR,
            b'5' => table::FIVE,
            b'6' => table::SIX,
            b'7' => table::SEVEN,
            b'8' => table::EIGHT,
            b'9' => table::NINE,
            b'A' => table::A,
            b'B' => table::B,
            b'C' => table::C,
            b'D' => table::D,
            b'E' => table::E,
            b'F' => table::F,
            b'G' => table::G,
            b'H' => table::H,
            b'I' => table::I,
            b'J' => table::J,
            b'K' => table::K,
            b'L' => table::L,
            b'M' => table::M,
            b'N' => table::N,
            b'O' => table::O,
            b'P' => table::P,
            b'Q' => table::Q,
            b'R' => table::R,
            b'S' => table::S,
            b'T' => table::T,
            b'U' => table::U,
            b'V' => table::V,
            b'W' => table::W,
            b'X' => table::X,
            b'Y' => table::Y,
            b'Z' => table::Z,
            b'a' => table::a,
            b'b' => table::b,
            b'c' => table::c,
            b'd' => table::d,
            b'e' => table::e,
            b'f' => table::f,
            b'g' => table::g,
            b'h' => table::h,
            b'i' => table::i,
            b'j' => table::j,
            b'k' => table::k,
            b'l' => table::l,
            b'm' => table::m,
            b'n' => table::n,
            b'o' => table::o,
            b'p' => table::p,
            b'q' => table::q,
            b'r' => table::r,
            b's' => table::s,
            b't' => table::t,
            b'u' => table::u,
            b'v' => table::v,
            b'w' => table::w,
            b'x' => table::x,
            b'y' => table::y,
            b'z' => table::z,
            b' ' => table::SPACE,
            b'&' => table::AMPERSAND,
            b'\'' => table::SIN_QUOTE,
            b'(' => table::L_PARENTH,
            b')' => table::R_PARENTH,
            b'*' => table::ASTERISK,
            b'+' => table::PLUS,
            b'-' => table::MINUS,
            b'=' => table::EQUAL,
            b'.' => table::F_STOP,
            b'!' => table::EXCLAM,
            b'"' => table::D_QUOTE,
            b'#' => table::HASH,
            b'$' => table::DOLLAR,
            b'%' => table::PERCENT,
            b'^' => table::CARET,
            // b'⌄' => D_CARET,
            b',' => table::COMMA,
            b':' => table::COLON,
            b';' => table::S_COLON,
            b'?' => table::QUESTION,
            b'@' => table::AT,
            b'/' => table::SLASH,
            b'<' => table::LESS,
            b'>' => table::MORE,
            b'|' => table::V_BAR,
            b'\\' => table::B_SLASH,
            // b'€' => EURO,
            // b'£' => POUND,
            b'[' => table::L_BRACKET,
            b']' => table::R_BRACKET,
            b'{' => table::L_BRACE,
            b'}' => table::R_BRACE,
            b'_' => table::U_SCORE,
            // b'█' => CURSOR,
            b'~' => table::TILDE,
            _ => panic!("unknown character"),
        }
    }
}
