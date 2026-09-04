#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BorderLine {
    None = 0,
    Plain = 1,
    Heavy = 2,
    Double = 3,
}

impl BorderLine {
    pub const fn merge(self, other: Self) -> Self {
        let a = self as u8;
        let b = other as u8;
        if a >= b { self } else { other }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderJunction {
    pub up: BorderLine,
    pub right: BorderLine,
    pub down: BorderLine,
    pub left: BorderLine,
    pub rounded: bool,
}

impl BorderJunction {
    pub const fn new(
        up: BorderLine,
        right: BorderLine,
        down: BorderLine,
        left: BorderLine,
    ) -> Self {
        Self {
            up,
            right,
            down,
            left,
            rounded: false,
        }
    }

    pub const fn rounded(
        up: BorderLine,
        right: BorderLine,
        down: BorderLine,
        left: BorderLine,
    ) -> Self {
        Self {
            up,
            right,
            down,
            left,
            rounded: true,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let up = self.up.merge(other.up);
        let right = self.right.merge(other.right);
        let down = self.down.merge(other.down);
        let left = self.left.merge(other.left);
        let is_corner = matches!(
            (up, right, down, left),
            (
                BorderLine::None,
                BorderLine::Plain,
                BorderLine::Plain,
                BorderLine::None
            ) | (
                BorderLine::None,
                BorderLine::None,
                BorderLine::Plain,
                BorderLine::Plain
            ) | (
                BorderLine::Plain,
                BorderLine::None,
                BorderLine::None,
                BorderLine::Plain
            ) | (
                BorderLine::Plain,
                BorderLine::Plain,
                BorderLine::None,
                BorderLine::None
            )
        );
        let rounded = self.rounded && other.rounded && is_corner;
        Self {
            up,
            right,
            down,
            left,
            rounded,
        }
    }
}

pub fn char_to_junction(ch: char) -> Option<BorderJunction> {
    use BorderLine::*;
    match ch {
        '─' => Some(BorderJunction::new(None, Plain, None, Plain)),
        '│' => Some(BorderJunction::new(Plain, None, Plain, None)),
        '┌' => Some(BorderJunction::new(None, Plain, Plain, None)),
        '┐' => Some(BorderJunction::new(None, None, Plain, Plain)),
        '└' => Some(BorderJunction::new(Plain, Plain, None, None)),
        '┘' => Some(BorderJunction::new(Plain, None, None, Plain)),
        '├' => Some(BorderJunction::new(Plain, Plain, Plain, None)),
        '┤' => Some(BorderJunction::new(Plain, None, Plain, Plain)),
        '┬' => Some(BorderJunction::new(None, Plain, Plain, Plain)),
        '┴' => Some(BorderJunction::new(Plain, Plain, None, Plain)),
        '┼' => Some(BorderJunction::new(Plain, Plain, Plain, Plain)),
        '╴' => Some(BorderJunction::new(None, None, None, Plain)),
        '╵' => Some(BorderJunction::new(Plain, None, None, None)),
        '╶' => Some(BorderJunction::new(None, Plain, None, None)),
        '╷' => Some(BorderJunction::new(None, None, Plain, None)),

        '╭' => Some(BorderJunction::rounded(None, Plain, Plain, None)),
        '╮' => Some(BorderJunction::rounded(None, None, Plain, Plain)),
        '╯' => Some(BorderJunction::rounded(Plain, None, None, Plain)),
        '╰' => Some(BorderJunction::rounded(Plain, Plain, None, None)),

        '═' => Some(BorderJunction::new(None, Double, None, Double)),
        '║' => Some(BorderJunction::new(Double, None, Double, None)),
        '╔' => Some(BorderJunction::new(None, Double, Double, None)),
        '╗' => Some(BorderJunction::new(None, None, Double, Double)),
        '╚' => Some(BorderJunction::new(Double, Double, None, None)),
        '╝' => Some(BorderJunction::new(Double, None, None, Double)),
        '╠' => Some(BorderJunction::new(Double, Double, Double, None)),
        '╣' => Some(BorderJunction::new(Double, None, Double, Double)),
        '╦' => Some(BorderJunction::new(None, Double, Double, Double)),
        '╩' => Some(BorderJunction::new(Double, Double, None, Double)),
        '╬' => Some(BorderJunction::new(Double, Double, Double, Double)),

        '━' => Some(BorderJunction::new(None, Heavy, None, Heavy)),
        '┃' => Some(BorderJunction::new(Heavy, None, Heavy, None)),
        '┏' => Some(BorderJunction::new(None, Heavy, Heavy, None)),
        '┓' => Some(BorderJunction::new(None, None, Heavy, Heavy)),
        '┗' => Some(BorderJunction::new(Heavy, Heavy, None, None)),
        '┛' => Some(BorderJunction::new(Heavy, None, None, Heavy)),
        '┣' => Some(BorderJunction::new(Heavy, Heavy, Heavy, None)),
        '┫' => Some(BorderJunction::new(Heavy, None, Heavy, Heavy)),
        '┳' => Some(BorderJunction::new(None, Heavy, Heavy, Heavy)),
        '┻' => Some(BorderJunction::new(Heavy, Heavy, None, Heavy)),
        '╋' => Some(BorderJunction::new(Heavy, Heavy, Heavy, Heavy)),
        '╸' => Some(BorderJunction::new(None, None, None, Heavy)),
        '╹' => Some(BorderJunction::new(Heavy, None, None, None)),
        '╺' => Some(BorderJunction::new(None, Heavy, None, None)),
        '╻' => Some(BorderJunction::new(None, None, Heavy, None)),

        '┍' => Some(BorderJunction::new(None, Heavy, Plain, None)),
        '┎' => Some(BorderJunction::new(None, Plain, Heavy, None)),
        '┑' => Some(BorderJunction::new(None, None, Plain, Heavy)),
        '┒' => Some(BorderJunction::new(None, None, Heavy, Plain)),
        '┕' => Some(BorderJunction::new(Plain, Heavy, None, None)),
        '┖' => Some(BorderJunction::new(Heavy, Plain, None, None)),
        '┙' => Some(BorderJunction::new(Plain, None, None, Heavy)),
        '┚' => Some(BorderJunction::new(Heavy, None, None, Plain)),

        '┝' => Some(BorderJunction::new(Plain, Heavy, Plain, None)),
        '┞' => Some(BorderJunction::new(Heavy, Plain, Plain, None)),
        '┟' => Some(BorderJunction::new(Plain, Plain, Heavy, None)),
        '┠' => Some(BorderJunction::new(Heavy, Plain, Heavy, None)),
        '┡' => Some(BorderJunction::new(Plain, Heavy, Heavy, None)),
        '┢' => Some(BorderJunction::new(Heavy, Heavy, Plain, None)),
        '┥' => Some(BorderJunction::new(Plain, None, Plain, Heavy)),
        '┦' => Some(BorderJunction::new(Heavy, None, Plain, Plain)),
        '┧' => Some(BorderJunction::new(Plain, None, Heavy, Plain)),
        '┨' => Some(BorderJunction::new(Heavy, None, Heavy, Plain)),
        '┩' => Some(BorderJunction::new(Plain, None, Heavy, Heavy)),
        '┪' => Some(BorderJunction::new(Heavy, None, Plain, Heavy)),

        '┭' => Some(BorderJunction::new(None, Plain, Plain, Heavy)),
        '┮' => Some(BorderJunction::new(None, Heavy, Plain, Plain)),
        '┯' => Some(BorderJunction::new(None, Heavy, Plain, Heavy)),
        '┰' => Some(BorderJunction::new(None, Plain, Heavy, Plain)),
        '┱' => Some(BorderJunction::new(None, Plain, Heavy, Heavy)),
        '┲' => Some(BorderJunction::new(None, Heavy, Heavy, Plain)),
        '┵' => Some(BorderJunction::new(Plain, Plain, None, Heavy)),
        '┶' => Some(BorderJunction::new(Plain, Heavy, None, Plain)),
        '┷' => Some(BorderJunction::new(Plain, Heavy, None, Heavy)),
        '┸' => Some(BorderJunction::new(Heavy, Plain, None, Plain)),
        '┹' => Some(BorderJunction::new(Heavy, Plain, None, Heavy)),
        '┺' => Some(BorderJunction::new(Heavy, Heavy, None, Plain)),

        '┽' => Some(BorderJunction::new(Plain, Plain, Plain, Heavy)),
        '┾' => Some(BorderJunction::new(Plain, Heavy, Plain, Plain)),
        '┿' => Some(BorderJunction::new(Plain, Heavy, Plain, Heavy)),
        '╀' => Some(BorderJunction::new(Heavy, Plain, Plain, Plain)),
        '╁' => Some(BorderJunction::new(Plain, Plain, Heavy, Plain)),
        '╂' => Some(BorderJunction::new(Heavy, Plain, Heavy, Plain)),

        '╒' => Some(BorderJunction::new(None, Double, Plain, None)),
        '╓' => Some(BorderJunction::new(None, Plain, Double, None)),
        '╕' => Some(BorderJunction::new(None, None, Plain, Double)),
        '╖' => Some(BorderJunction::new(None, None, Double, Plain)),
        '╘' => Some(BorderJunction::new(Plain, Double, None, None)),
        '╙' => Some(BorderJunction::new(Double, Plain, None, None)),
        '╛' => Some(BorderJunction::new(Plain, None, None, Double)),
        '╜' => Some(BorderJunction::new(Double, None, None, Plain)),

        '╞' => Some(BorderJunction::new(Plain, Double, Plain, None)),
        '╟' => Some(BorderJunction::new(Double, Plain, Double, None)),
        '╡' => Some(BorderJunction::new(Plain, None, Plain, Double)),
        '╢' => Some(BorderJunction::new(Double, None, Double, Plain)),
        '╤' => Some(BorderJunction::new(None, Plain, Double, Plain)),
        '╥' => Some(BorderJunction::new(None, Double, Plain, Double)),
        '╧' => Some(BorderJunction::new(Double, Plain, None, Plain)),
        '╨' => Some(BorderJunction::new(Plain, Double, None, Double)),
        '╪' => Some(BorderJunction::new(Plain, Double, Plain, Double)),
        '╫' => Some(BorderJunction::new(Double, Plain, Double, Plain)),

        _ => Option::None,
    }
}

pub fn junction_to_char(b: BorderJunction) -> char {
    use BorderLine::*;

    if b.rounded {
        match (b.up, b.right, b.down, b.left) {
            (None, Plain, Plain, None) => return '╭',
            (None, None, Plain, Plain) => return '╮',
            (Plain, None, None, Plain) => return '╯',
            (Plain, Plain, None, None) => return '╰',
            _ => {}
        }
    }

    match (b.up, b.right, b.down, b.left) {
        (None, Plain, None, Plain) => '─',
        (Plain, None, Plain, None) => '│',
        (None, Plain, Plain, None) => '┌',
        (None, None, Plain, Plain) => '┐',
        (Plain, Plain, None, None) => '└',
        (Plain, None, None, Plain) => '┘',
        (Plain, Plain, Plain, None) => '├',
        (Plain, None, Plain, Plain) => '┤',
        (None, Plain, Plain, Plain) => '┬',
        (Plain, Plain, None, Plain) => '┴',
        (Plain, Plain, Plain, Plain) => '┼',
        (None, None, None, Plain) => '╴',
        (Plain, None, None, None) => '╵',
        (None, Plain, None, None) => '╶',
        (None, None, Plain, None) => '╷',

        (None, Double, None, Double) => '═',
        (Double, None, Double, None) => '║',
        (None, Double, Double, None) => '╔',
        (None, None, Double, Double) => '╗',
        (Double, Double, None, None) => '╚',
        (Double, None, None, Double) => '╝',
        (Double, Double, Double, None) => '╠',
        (Double, None, Double, Double) => '╣',
        (None, Double, Double, Double) => '╦',
        (Double, Double, None, Double) => '╩',
        (Double, Double, Double, Double) => '╬',

        (None, Heavy, None, Heavy) => '━',
        (Heavy, None, Heavy, None) => '┃',
        (None, Heavy, Heavy, None) => '┏',
        (None, None, Heavy, Heavy) => '┓',
        (Heavy, Heavy, None, None) => '┗',
        (Heavy, None, None, Heavy) => '┛',
        (Heavy, Heavy, Heavy, None) => '┣',
        (Heavy, None, Heavy, Heavy) => '┫',
        (None, Heavy, Heavy, Heavy) => '┳',
        (Heavy, Heavy, None, Heavy) => '┻',
        (Heavy, Heavy, Heavy, Heavy) => '╋',
        (None, None, None, Heavy) => '╸',
        (Heavy, None, None, None) => '╹',
        (None, Heavy, None, None) => '╺',
        (None, None, Heavy, None) => '╻',

        (None, Heavy, Plain, None) => '┍',
        (None, Plain, Heavy, None) => '┎',
        (None, None, Plain, Heavy) => '┑',
        (None, None, Heavy, Plain) => '┒',
        (Plain, Heavy, None, None) => '┕',
        (Heavy, Plain, None, None) => '┖',
        (Plain, None, None, Heavy) => '┙',
        (Heavy, None, None, Plain) => '┚',

        (Plain, Heavy, Plain, None) => '┝',
        (Heavy, Plain, Plain, None) => '┞',
        (Plain, Plain, Heavy, None) => '┟',
        (Heavy, Plain, Heavy, None) => '┠',
        (Plain, Heavy, Heavy, None) => '┡',
        (Heavy, Heavy, Plain, None) => '┢',
        (Plain, None, Plain, Heavy) => '┥',
        (Heavy, None, Plain, Plain) => '┦',
        (Plain, None, Heavy, Plain) => '┧',
        (Heavy, None, Heavy, Plain) => '┨',
        (Plain, None, Heavy, Heavy) => '┩',
        (Heavy, None, Plain, Heavy) => '┪',
        (None, Plain, Plain, Heavy) => '┭',
        (None, Heavy, Plain, Plain) => '┮',
        (None, Heavy, Plain, Heavy) => '┯',
        (None, Plain, Heavy, Plain) => '┰',
        (None, Plain, Heavy, Heavy) => '┱',
        (None, Heavy, Heavy, Plain) => '┲',
        (Plain, Plain, None, Heavy) => '┵',
        (Plain, Heavy, None, Plain) => '┶',
        (Plain, Heavy, None, Heavy) => '┷',
        (Heavy, Plain, None, Plain) => '┸',
        (Heavy, Plain, None, Heavy) => '┹',
        (Heavy, Heavy, None, Plain) => '┺',

        (Plain, Plain, Plain, Heavy) => '┽',
        (Plain, Heavy, Plain, Plain) => '┾',
        (Plain, Heavy, Plain, Heavy) => '┿',
        (Heavy, Plain, Plain, Plain) => '╀',
        (Plain, Plain, Heavy, Plain) => '╁',
        (Heavy, Plain, Heavy, Plain) => '╂',

        (None, Double, Plain, None) => '╒',
        (None, Plain, Double, None) => '╓',
        (None, None, Plain, Double) => '╕',
        (None, None, Double, Plain) => '╖',
        (Plain, Double, None, None) => '╘',
        (Double, Plain, None, None) => '╙',
        (Plain, None, None, Double) => '╛',
        (Double, None, None, Plain) => '╜',

        (Plain, Double, Plain, None) => '╞',
        (Double, Plain, Double, None) => '╟',
        (Plain, None, Plain, Double) => '╡',
        (Double, None, Double, Plain) => '╢',
        (None, Plain, Double, Plain) => '╤',
        (None, Double, Plain, Double) => '╥',
        (Double, Plain, None, Plain) => '╧',
        (Plain, Double, None, Double) => '╨',
        (Plain, Double, Plain, Double) => '╪',
        (Double, Plain, Double, Plain) => '╫',

        _ => {
            let has_double =
                b.up == Double || b.right == Double || b.down == Double || b.left == Double;
            let has_heavy = b.up == Heavy || b.right == Heavy || b.down == Heavy || b.left == Heavy;

            let count = (b.up != None) as u8
                + (b.right != None) as u8
                + (b.down != None) as u8
                + (b.left != None) as u8;

            if count == 4 {
                if has_double {
                    '╬'
                } else if has_heavy {
                    '╋'
                } else {
                    '┼'
                }
            } else if count == 3 {
                if b.up == None {
                    if has_double {
                        '╦'
                    } else if has_heavy {
                        '┳'
                    } else {
                        '┬'
                    }
                } else if b.down == None {
                    if has_double {
                        '╩'
                    } else if has_heavy {
                        '┻'
                    } else {
                        '┴'
                    }
                } else if b.left == None {
                    if has_double {
                        '╠'
                    } else if has_heavy {
                        '┣'
                    } else {
                        '├'
                    }
                } else {
                    if has_double {
                        '╣'
                    } else if has_heavy {
                        '┫'
                    } else {
                        '┤'
                    }
                }
            } else if count == 2 {
                if b.up != None && b.down != None {
                    if has_double {
                        '║'
                    } else if has_heavy {
                        '┃'
                    } else {
                        '│'
                    }
                } else if b.left != None && b.right != None {
                    if has_double {
                        '═'
                    } else if has_heavy {
                        '━'
                    } else {
                        '─'
                    }
                } else if b.down != None && b.right != None {
                    if has_double {
                        '╔'
                    } else if has_heavy {
                        '┏'
                    } else {
                        '┌'
                    }
                } else if b.down != None && b.left != None {
                    if has_double {
                        '╗'
                    } else if has_heavy {
                        '┓'
                    } else {
                        '┐'
                    }
                } else if b.up != None && b.right != None {
                    if has_double {
                        '╚'
                    } else if has_heavy {
                        '┗'
                    } else {
                        '└'
                    }
                } else {
                    if has_double {
                        '╝'
                    } else if has_heavy {
                        '┛'
                    } else {
                        '┘'
                    }
                }
            } else {
                ' '
            }
        }
    }
}

pub fn merge_borders(a: char, b: char) -> Option<char> {
    let j1 = char_to_junction(a)?;
    let j2 = char_to_junction(b)?;
    let merged = j1.merge(j2);
    Some(junction_to_char(merged))
}
