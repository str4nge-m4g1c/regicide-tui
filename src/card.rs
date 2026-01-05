use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub fn symbol(&self) -> char {
        match self {
            Suit::Hearts => '♥',
            Suit::Diamonds => '♦',
            Suit::Clubs => '♣',
            Suit::Spades => '♠',
        }
    }

    pub fn is_red(&self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Ace,   // Animal Companion
    Jack,  // Enemy
    Queen, // Enemy
    King,  // Enemy
    Jester,
}

impl Rank {
    /// Returns the base value of the rank when played or discarded
    pub fn value(&self) -> u8 {
        match self {
            Rank::Ace => 1,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,  // When drawn as a card in hand
            Rank::Queen => 15, // When drawn as a card in hand
            Rank::King => 20,  // When drawn as a card in hand
            Rank::Jester => 0,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Rank::Ace => "A".to_string(),
            Rank::Two => "2".to_string(),
            Rank::Three => "3".to_string(),
            Rank::Four => "4".to_string(),
            Rank::Five => "5".to_string(),
            Rank::Six => "6".to_string(),
            Rank::Seven => "7".to_string(),
            Rank::Eight => "8".to_string(),
            Rank::Nine => "9".to_string(),
            Rank::Ten => "10".to_string(),
            Rank::Jack => "J".to_string(),
            Rank::Queen => "Q".to_string(),
            Rank::King => "K".to_string(),
            Rank::Jester => "*".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
    }

    /// Returns the attack value of the card
    pub fn value(&self) -> u8 {
        self.rank.value()
    }

    /// Returns true if this is an Animal Companion (Ace)
    pub fn is_companion(&self) -> bool {
        self.rank == Rank::Ace
    }

    /// Returns true if this is a Jester
    pub fn is_jester(&self) -> bool {
        self.rank == Rank::Jester
    }

    pub fn display(&self) -> String {
        format!("{}{}", self.rank.display(), self.suit.symbol())
    }
}
