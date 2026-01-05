use crate::card::{Card, Rank, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enemy {
    pub card: Card,
    pub max_hp: u8,
    pub current_hp: u8,
    pub attack: u8,
    pub immunity_cancelled: bool, // True if Jester has been played
}

impl Enemy {
    pub fn new(card: Card) -> Self {
        let (max_hp, attack) = match card.rank {
            Rank::Jack => (20, 10),
            Rank::Queen => (30, 15),
            Rank::King => (40, 20),
            _ => panic!("Cannot create enemy from non-face card"),
        };

        Self {
            card,
            max_hp,
            current_hp: max_hp,
            attack,
            immunity_cancelled: false,
        }
    }

    /// Check if the enemy is immune to a card's suit power
    pub fn is_immune_to(&self, card_suit: Suit) -> bool {
        !self.immunity_cancelled && self.card.suit == card_suit
    }

    /// Apply damage to the enemy
    pub fn take_damage(&mut self, damage: u8) {
        self.current_hp = self.current_hp.saturating_sub(damage);
    }

    /// Check if the enemy is defeated
    pub fn is_defeated(&self) -> bool {
        self.current_hp == 0
    }

    /// Check if the enemy was defeated with exact damage
    pub fn defeated_exactly(&self, total_damage: u8) -> bool {
        total_damage == self.max_hp
    }

    /// Cancel the enemy's immunity (Jester effect)
    pub fn cancel_immunity(&mut self) {
        self.immunity_cancelled = true;
    }

    /// Get the enemy's current attack value after shields
    pub fn get_attack_after_shields(&self, shield_value: u8) -> u8 {
        self.attack.saturating_sub(shield_value)
    }

    pub fn name(&self) -> String {
        format!(
            "{} of {}",
            match self.card.rank {
                Rank::Jack => "Jack",
                Rank::Queen => "Queen",
                Rank::King => "King",
                _ => "Unknown",
            },
            match self.card.suit {
                Suit::Hearts => "Hearts",
                Suit::Diamonds => "Diamonds",
                Suit::Clubs => "Clubs",
                Suit::Spades => "Spades",
            }
        )
    }

    /// Returns the HP bar as a visual representation
    pub fn hp_bar(&self, width: usize) -> String {
        let filled = ((self.current_hp as f32 / self.max_hp as f32) * width as f32) as usize;
        let empty = width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), " ".repeat(empty))
    }
}
