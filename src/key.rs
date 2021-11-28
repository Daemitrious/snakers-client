pub enum Key {
    W,
    A,
    S,
    D,
    Q,
}

impl Key {
    pub fn from_char(key: char) -> Option<Self> {
        match key {
            'w' => Some(Self::W),
            'a' => Some(Self::A),
            's' => Some(Self::S),
            'd' => Some(Self::D),
            'q' => Some(Self::Q),
            _ => None,
        }
    }
    pub fn to_byte(self) -> [u8; 1] {
        [match self {
            Key::W => 119,
            Key::A => 97,
            Key::S => 115,
            Key::D => 100,
            Key::Q => 113,
        }]
    }
}

impl Clone for Key {
    fn clone(&self) -> Self {
        match self {
            Self::W => Self::W,
            Self::A => Self::A,
            Self::S => Self::S,
            Self::D => Self::D,
            Self::Q => Self::Q,
        }
    }
}
