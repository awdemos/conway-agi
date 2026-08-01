const MESSAGE_SYMBOLS: &[(u8, char)] = &[
    (0, '·'),
    (1, '▸'),
    (2, '◆'),
    (3, '●'),
    (4, '◐'),
    (5, '◑'),
    (6, '★'),
    (7, '✦'),
    (8, '⚑'),
    (9, '✚'),
    (10, '✱'),
    (11, '⬡'),
    (12, '⬢'),
    (13, '∞'),
    (14, '⟁'),
    (15, '✿'),
];

pub const MAX_MESSAGE_LEN: usize = 40;

#[derive(Clone, Debug, Default)]
pub struct Decoder {
    message: Vec<char>,
    last_symbol: Option<char>,
}

impl Decoder {
    pub fn push_signal(&mut self, signal_type: u8) {
        let symbol = MESSAGE_SYMBOLS
            .iter()
            .find(|(t, _)| *t == signal_type % 16)
            .map(|(_, ch)| *ch)
            .unwrap_or('?');
        if self.last_symbol != Some(symbol) {
            self.message.push(symbol);
            self.last_symbol = Some(symbol);
            if self.message.len() > MAX_MESSAGE_LEN {
                self.message.remove(0);
            }
        }
    }

    pub fn message(&self) -> String {
        self.message.iter().collect()
    }

    pub fn clear(&mut self) {
        self.message.clear();
        self.last_symbol = None;
    }
}
