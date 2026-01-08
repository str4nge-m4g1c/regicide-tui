use crate::card::{Card, Suit};
use crate::deck::Deck;
use crate::enemy::Enemy;
use crate::player::Player;
use chrono::Local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameState {
    Playing,
    Victory,
    Defeat(String), // Reason for defeat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub castle_deck: Deck,
    pub tavern_deck: Deck,
    pub discard_pile: Vec<Card>,
    pub current_enemy: Option<Enemy>,
    pub player: Player,
    pub played_cards: Vec<Card>,
    pub shield_value: u8, // Cumulative shield from Spades
    pub total_damage: u8, // Total damage dealt to current enemy
    pub game_state: GameState,
    pub game_log: Vec<String>,
    pub jester_count: u8,         // For solo mode
    pub jesters_used: u8,         // For solo mode
    pub jester_played_this_turn: bool, // Track if Jester was played to skip Step 4
}

impl Game {
    /// Create a new solo game
    pub fn new_solo() -> Self {
        let mut tavern_deck = Deck::create_tavern_deck(0); // 0 Jesters for solo
        let castle_deck = Deck::create_castle_deck();

        let mut player = Player::new("Hero".to_string(), 8);

        // Draw initial hand
        let initial_cards = tavern_deck.draw_multiple(8);
        player.draw_multiple(initial_cards);

        let mut game = Self {
            castle_deck,
            tavern_deck,
            discard_pile: Vec::new(),
            current_enemy: None,
            player,
            played_cards: Vec::new(),
            shield_value: 0,
            total_damage: 0,
            game_state: GameState::Playing,
            game_log: Vec::new(),
            jester_count: 2,
            jesters_used: 0,
            jester_played_this_turn: false,
        };

        // Reveal first enemy
        game.reveal_next_enemy();
        game.log("Game started! Defeat all 12 enemies to win.");

        game
    }

    /// Reveal the next enemy from the castle deck
    fn reveal_next_enemy(&mut self) {
        if let Some(card) = self.castle_deck.draw() {
            let enemy = Enemy::new(card);
            self.log(format!("A {} appears!", enemy.name()));
            self.current_enemy = Some(enemy);
            self.shield_value = 0;
            self.total_damage = 0;
            self.played_cards.clear();
        } else {
            // No more enemies - Victory!
            self.game_state = GameState::Victory;
            self.log("Victory! All enemies have been defeated!");
        }
    }

    /// Add a message to the game log (limited to 100 entries)
    pub fn log<S: Into<String>>(&mut self, message: S) {
        let timestamp = Local::now().format("%H:%M:%S");
        let log_entry = format!("[{}] {}", timestamp, message.into());
        self.game_log.push(log_entry);
        // Keep only the last 100 log entries
        if self.game_log.len() > 100 {
            self.game_log.remove(0);
        }
    }

    /// Validate if cards can be played together
    pub fn validate_play(&self, card_indices: &[usize]) -> Result<(), String> {
        if card_indices.is_empty() {
            return Err("Must select at least one card".to_string());
        }

        let cards: Vec<&Card> = card_indices
            .iter()
            .filter_map(|&i| self.player.hand.get(i))
            .collect();

        if cards.len() != card_indices.len() {
            return Err("Invalid card indices".to_string());
        }

        // Jester must be played alone
        if cards.iter().any(|c| c.is_jester()) {
            if cards.len() > 1 {
                return Err("Jester must be played alone".to_string());
            }
            return Ok(());
        }

        // Single card is always valid
        if cards.len() == 1 {
            return Ok(());
        }

        // Check for Ace + one other card
        let ace_count = cards.iter().filter(|c| c.is_companion()).count();
        if ace_count > 0 {
            if cards.len() == 2 && ace_count == 1 {
                return Ok(()); // Ace + one other card is valid
            } else if cards.len() == 2 && ace_count == 2 {
                return Ok(()); // Ace + Ace is valid
            } else if cards.len() > 2 {
                return Err("Ace can only be paired with one other card".to_string());
            }
            // If we get here, we have aces but not in valid combo - fall through to same-rank check
        }

        // Combo: 2-4 cards of same rank, total <= 10
        // First, ensure we don't have more than 4 cards
        if cards.len() > 4 {
            return Err("Cannot play more than 4 cards at once".to_string());
        }

        let first_rank = cards[0].rank;
        if !cards.iter().all(|c| c.rank == first_rank) {
            return Err(
                "Combo cards must all have the same rank (or use Ace + 1 card)".to_string(),
            );
        }

        let total: u8 = cards.iter().map(|c| c.value()).sum();
        if total > 10 {
            return Err("Combo total must be 10 or less".to_string());
        }

        Ok(())
    }

    /// Play cards from hand (Step 1 & 2)
    /// Returns true if enemy was defeated (and a new one appeared)
    pub fn play_cards(&mut self, card_indices: Vec<usize>) -> Result<bool, String> {
        // Reset Jester flag at the start of a new turn
        self.jester_played_this_turn = false;

        // Validate the play
        self.validate_play(&card_indices)?;

        // Remove cards from hand
        let cards = self.player.play_cards(card_indices);

        if cards.is_empty() {
            return Err("Failed to play cards".to_string());
        }

        let attack_value: u8 = cards.iter().map(|c| c.value()).sum();

        // Handle Jester special case
        if cards[0].is_jester() {
            self.log("Played Jester - Enemy immunity cancelled!");
            if let Some(ref mut enemy) = self.current_enemy {
                let enemy_suit = enemy.card.suit;
                enemy.cancel_immunity();

                // Special rule: If Jester played against Spades enemy,
                // retroactively apply all previously blocked Spades to shield
                if enemy_suit == Suit::Spades {
                    // Add shield value for each Spades card that was blocked
                    let retroactive_shield: u8 = self
                        .played_cards
                        .iter()
                        .filter(|c| c.suit == Suit::Spades)
                        .map(|c| c.value())
                        .sum();

                    if retroactive_shield > 0 {
                        self.shield_value += retroactive_shield;
                        self.log(format!(
                            "Spades now active! Shield increased by {} (Total: {})",
                            retroactive_shield, self.shield_value
                        ));
                    }
                }
            }
            self.discard_pile.extend(cards);
            // Jester skips Steps 3 and 4 (dealt damage and suffer damage)
            self.jester_played_this_turn = true;
            return Ok(false);
        }

        // Store current enemy for comparison
        let enemy_before = self.current_enemy.as_ref().map(|e| e.card);

        // Log the play
        let card_names: Vec<String> = cards.iter().map(|c| c.display()).collect();
        self.log(format!(
            "Played: {} (Attack: {})",
            card_names.join(", "),
            attack_value
        ));

        // Apply suit powers (Step 2)
        self.apply_suit_powers(&cards, attack_value)?;

        // Store played cards
        self.played_cards.extend(cards);

        // Deal damage (Step 3)
        self.deal_damage(attack_value)?;

        // Check if enemy was defeated (new enemy appeared)
        let enemy_defeated = enemy_before != self.current_enemy.as_ref().map(|e| e.card);

        Ok(enemy_defeated)
    }

    /// Apply suit powers to the cards played
    fn apply_suit_powers(&mut self, cards: &[Card], attack_value: u8) -> Result<(), String> {
        let enemy = self.current_enemy.as_ref().ok_or("No current enemy")?;

        // Collect suits and check immunity
        let mut hearts_power = 0;
        let mut diamonds_power = 0;
        let mut clubs_active = false;
        let mut spades_power = 0;
        let mut log_messages = Vec::new();

        for card in cards {
            match card.suit {
                Suit::Hearts => {
                    if !enemy.is_immune_to(Suit::Hearts) {
                        hearts_power = attack_value;
                    } else {
                        log_messages.push("Hearts power blocked by immunity".to_string());
                    }
                }
                Suit::Diamonds => {
                    if !enemy.is_immune_to(Suit::Diamonds) {
                        diamonds_power = attack_value;
                    } else {
                        log_messages.push("Diamonds power blocked by immunity".to_string());
                    }
                }
                Suit::Clubs => {
                    if !enemy.is_immune_to(Suit::Clubs) {
                        clubs_active = true;
                    } else {
                        log_messages.push(
                            "Clubs power blocked by immunity (double damage negated)".to_string(),
                        );
                    }
                }
                Suit::Spades => {
                    if !enemy.is_immune_to(Suit::Spades) {
                        spades_power = attack_value;
                    } else {
                        log_messages.push("Spades power blocked by immunity".to_string());
                    }
                }
            }
        }

        // Log immunity messages
        for msg in log_messages {
            self.log(msg);
        }

        // Apply Hearts first (heal discard pile into tavern deck)
        if hearts_power > 0 {
            let heal_count = hearts_power.min(self.discard_pile.len() as u8) as usize;
            if heal_count > 0 {
                // Shuffle discard pile
                let mut temp_deck = Deck::new();
                temp_deck.cards = self.discard_pile.clone();
                temp_deck.shuffle();

                // Take cards from shuffled discard
                let healed: Vec<Card> = temp_deck.cards.drain(..heal_count).collect();
                self.discard_pile = temp_deck.cards;

                // Add to bottom of tavern deck
                self.tavern_deck.add_multiple_to_bottom(healed);
                self.log(format!(
                    "Healed {} cards from discard to tavern deck",
                    heal_count
                ));
            }
        }

        // Apply Diamonds (draw cards)
        if diamonds_power > 0 {
            let mut cards_to_draw = diamonds_power as usize;
            let mut drawn = 0;
            while cards_to_draw > 0 && !self.player.is_hand_full() {
                if let Some(card) = self.tavern_deck.draw() {
                    self.player.draw_card(card);
                    drawn += 1;
                    cards_to_draw -= 1;
                } else {
                    break;
                }
            }
            if drawn > 0 {
                self.log(format!("Drew {} cards", drawn));
            }
        }

        // Store clubs status for damage calculation
        if clubs_active {
            self.log("Clubs active - double damage!");
        }

        // Apply Spades (shield - cumulative)
        if spades_power > 0 {
            self.shield_value += spades_power;
            self.log(format!(
                "Shield increased by {} (Total: {})",
                spades_power, self.shield_value
            ));
        }

        Ok(())
    }

    /// Deal damage to the enemy (Step 3)
    fn deal_damage(&mut self, mut attack_value: u8) -> Result<(), String> {
        let enemy = self.current_enemy.as_mut().ok_or("No current enemy")?;

        // Check if clubs were played (check in played_cards for this turn)
        let clubs_played = self
            .played_cards
            .iter()
            .any(|c| c.suit == Suit::Clubs && !enemy.is_immune_to(Suit::Clubs));

        if clubs_played {
            attack_value *= 2;
        }

        self.total_damage += attack_value;
        let max_hp = enemy.max_hp;
        enemy.take_damage(attack_value);

        // Check if enemy is defeated
        let is_defeated = enemy.is_defeated();

        self.log(format!(
            "Dealt {} damage (Total: {}/{})",
            attack_value, self.total_damage, max_hp
        ));

        if is_defeated {
            self.enemy_defeated();
        }

        Ok(())
    }

    /// Handle enemy defeat
    fn enemy_defeated(&mut self) {
        let enemy = self.current_enemy.take().unwrap();

        // Check if defeated with exact damage
        if enemy.defeated_exactly(self.total_damage) {
            self.log(format!("Exact damage! {} captured!", enemy.name()));
            self.tavern_deck.add_to_top(enemy.card);
        } else {
            self.log(format!("{} defeated!", enemy.name()));
            self.discard_pile.push(enemy.card);
        }

        // Discard all played cards
        self.discard_pile.append(&mut self.played_cards);

        // Reveal next enemy
        self.reveal_next_enemy();
    }

    /// Yield turn (skip to enemy attack)
    pub fn yield_turn(&mut self) -> Result<(), String> {
        // Reset Jester flag at the start of a new turn
        self.jester_played_this_turn = false;
        self.log("Yielded turn");
        Ok(())
    }

    /// Enemy attacks (Step 4)
    pub fn enemy_attack(&mut self) -> Result<u8, String> {
        let enemy = self.current_enemy.as_ref().ok_or("No current enemy")?;

        let damage = enemy.get_attack_after_shields(self.shield_value);

        if damage > 0 {
            self.log(format!("Enemy attacks for {} damage!", damage));
        } else {
            self.log("Enemy attack fully blocked by shields!");
        }

        Ok(damage)
    }

    /// Discard cards to survive enemy attack
    pub fn discard_to_survive(&mut self, card_indices: Vec<usize>) -> Result<(), String> {
        let value = self.player.calculate_value(&card_indices);
        let enemy = self.current_enemy.as_ref().ok_or("No current enemy")?;
        let required = enemy.get_attack_after_shields(self.shield_value);

        if value < required {
            return Err(format!(
                "Not enough value (need {}, have {})",
                required, value
            ));
        }

        // Discard the cards
        let discarded = self.player.play_cards(card_indices);
        let card_names: Vec<String> = discarded.iter().map(|c| c.display()).collect();
        self.log(format!(
            "Discarded: {} (Value: {})",
            card_names.join(", "),
            value
        ));
        self.discard_pile.extend(discarded);

        Ok(())
    }

    /// Use a Jester (solo mode only)
    pub fn use_jester(&mut self) -> Result<(), String> {
        if self.jesters_used >= self.jester_count {
            return Err("No Jesters remaining".to_string());
        }

        // Discard hand
        let hand_size = self.player.hand.len();
        let discarded: Vec<Card> = self.player.hand.drain(..).collect();
        self.discard_pile.extend(discarded);

        // Refill to 8 cards
        let cards = self.tavern_deck.draw_multiple(8);
        self.player.draw_multiple(cards);

        self.jesters_used += 1;
        self.log(format!(
            "Used Jester power! Discarded {} cards and drew fresh hand ({} Jesters remaining)",
            hand_size,
            self.jester_count - self.jesters_used
        ));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Rank, Suit};

    #[test]
    fn test_jester_skips_step_4() {
        // Test that playing a Jester sets the flag to skip enemy attack
        let mut game = Game::new_solo();

        // Add a Jester to the player's hand
        let jester = Card::new(Suit::Hearts, Rank::Jester);
        game.player.hand.clear();
        game.player.hand.push(jester);

        // Play the Jester
        let result = game.play_cards(vec![0]);

        // Should succeed and not defeat enemy
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false); // enemy not defeated

        // Jester flag should be set to skip Step 4
        assert_eq!(game.jester_played_this_turn, true);

        // Immunity should be cancelled
        assert_eq!(game.current_enemy.as_ref().unwrap().immunity_cancelled, true);
    }

    #[test]
    fn test_jester_flag_resets_on_next_turn() {
        // Test that the Jester flag resets when a new turn starts
        let mut game = Game::new_solo();

        // Manually set the flag
        game.jester_played_this_turn = true;

        // Add a card to hand
        let card = Card::new(Suit::Hearts, Rank::Five);
        game.player.hand.clear();
        game.player.hand.push(card);

        // Play a normal card
        let result = game.play_cards(vec![0]);

        // Should succeed
        assert!(result.is_ok());

        // Jester flag should be reset to false
        assert_eq!(game.jester_played_this_turn, false);
    }

    #[test]
    fn test_jester_flag_resets_on_yield() {
        // Test that the Jester flag resets when yielding
        let mut game = Game::new_solo();

        // Manually set the flag
        game.jester_played_this_turn = true;

        // Yield turn
        let result = game.yield_turn();

        // Should succeed
        assert!(result.is_ok());

        // Jester flag should be reset to false
        assert_eq!(game.jester_played_this_turn, false);
    }

    #[test]
    fn test_solo_jester_power_at_step_4() {
        // Test that solo Jester power can be used at start of Step 4 (discard phase)
        let mut game = Game::new_solo();

        // Setup: Give player only low value cards
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Two));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Two));

        let jesters_before = game.jesters_used;

        // Use Jester power (simulating Step 4 / discard phase)
        let result = game.use_jester();

        // Should succeed
        assert!(result.is_ok());

        // Hand should be refilled to 8 cards
        assert_eq!(game.player.hand.len(), 8, "Hand should be refilled to 8");

        // Jester count should increment
        assert_eq!(
            game.jesters_used,
            jesters_before + 1,
            "Jesters used should increment"
        );

        // Should have 1 Jester remaining (started with 2)
        assert_eq!(game.jester_count - game.jesters_used, 1);
    }

    #[test]
    fn test_solo_jester_power_limit() {
        // Test that solo Jester power can only be used jester_count times
        let mut game = Game::new_solo();

        // Use both Jesters
        assert!(game.use_jester().is_ok());
        assert!(game.use_jester().is_ok());

        // Third attempt should fail
        let result = game.use_jester();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No Jesters remaining");
    }

    #[test]
    fn test_jester_retroactive_spades() {
        // Test that Spades played before Jester against Spades enemy apply retroactively
        let mut game = Game::new_solo();

        // Setup: Enemy is Jack of Spades
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Spades, Rank::Jack)));

        // Setup player hand: 5♠, Jester, 5♥
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Spades, Rank::Five));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Jester));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));

        // Turn 1: Play 5 of Spades against Jack of Spades
        // Should be blocked by immunity, shield stays 0
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(game.shield_value, 0, "Shield should be 0 (blocked by immunity)");
        assert_eq!(game.played_cards.len(), 1, "One card in played_cards");

        // Turn 2: Play Jester to cancel immunity
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(
            game.current_enemy.as_ref().unwrap().immunity_cancelled,
            true,
            "Immunity should be cancelled"
        );
        // Shield should now include the retroactive Spades (value 5)
        assert_eq!(game.shield_value, 5, "Shield should retroactively include the 5♠");

        // Turn 3: Play 5 of Hearts, enemy should attack with reduced damage
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());

        // Enemy attack should be reduced: Jack attacks for 10, shield is 5, so 5 damage
        let enemy_attack = game.current_enemy.as_ref().unwrap().get_attack_after_shields(game.shield_value);
        assert_eq!(
            enemy_attack, 5,
            "Enemy attack should be 5 (10 - 5 shield)"
        );
    }
}
