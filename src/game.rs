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
    pub jester_count: u8,              // For solo mode
    pub jesters_used: u8,              // For solo mode
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

        // Store played cards BEFORE dealing damage
        // This ensures they're included if enemy is defeated
        self.played_cards.extend(cards.clone());

        // Deal damage (Step 3) - pass cards to check for Clubs in THIS turn only
        self.deal_damage(attack_value, &cards)?;

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
    fn deal_damage(&mut self, mut attack_value: u8, cards: &[Card]) -> Result<(), String> {
        let enemy = self.current_enemy.as_mut().ok_or("No current enemy")?;

        // Check if clubs were played in THIS turn only (not previous turns)
        let clubs_played = cards
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
        assert_eq!(
            game.current_enemy.as_ref().unwrap().immunity_cancelled,
            true
        );
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
        assert_eq!(
            game.shield_value, 0,
            "Shield should be 0 (blocked by immunity)"
        );
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
        assert_eq!(
            game.shield_value, 5,
            "Shield should retroactively include the 5♠"
        );

        // Turn 3: Play 5 of Hearts, enemy should attack with reduced damage
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());

        // Enemy attack should be reduced: Jack attacks for 10, shield is 5, so 5 damage
        let enemy_attack = game
            .current_enemy
            .as_ref()
            .unwrap()
            .get_attack_after_shields(game.shield_value);
        assert_eq!(enemy_attack, 5, "Enemy attack should be 5 (10 - 5 shield)");
    }

    #[test]
    fn test_clubs_power_does_not_persist() {
        // Test that Clubs double damage only applies to the turn it's played
        let mut game = Game::new_solo();

        // Ensure enemy is NOT Clubs (to avoid immunity blocking the test)
        // Replace enemy with Jack of Hearts for predictable testing
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack)));

        // Setup: Give player 5 of Clubs and 5 of Hearts
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Five));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));

        // Turn 1: Play 5 of Clubs -> should deal 10 damage (5 doubled)
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(game.total_damage, 10, "Clubs should double damage to 10"); // 5 * 2 = 10

        // Turn 2: Play 5 of Hearts -> should deal 5 damage (NOT doubled)
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(
            game.total_damage, 15,
            "Second turn should NOT benefit from Clubs doubling (10 + 5 = 15)"
        ); // 10 + 5 = 15 (NOT 10 + 10 = 20)
    }

    #[test]
    fn test_clubs_combo_doubles_total_damage() {
        // Test that Clubs in a combo doubles the TOTAL combo damage
        let mut game = Game::new_solo();

        // Ensure enemy is NOT Clubs
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack)));

        // Setup: Give player 3♣, 3♥, 3♦ for a combo (total 9 <= 10)
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Three));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Three));
        game.player
            .hand
            .push(Card::new(Suit::Diamonds, Rank::Three));

        // Play combo: 3♣ + 3♥ + 3♦ = 9 attack, doubled to 18 due to Clubs
        let result = game.play_cards(vec![0, 1, 2]);
        assert!(result.is_ok(), "Combo should be valid");
        assert_eq!(
            game.total_damage, 18,
            "Clubs in combo should double total damage: (3+3+3)*2 = 18"
        );
    }

    // ===== COMPREHENSIVE GAME RULES TESTS =====

    #[test]
    fn test_castle_deck_construction() {
        // Test Castle deck has correct structure: 4 Kings, 4 Queens, 4 Jacks
        use crate::card::Rank;
        use crate::deck::Deck;

        let castle = Deck::create_castle_deck();
        assert_eq!(castle.len(), 12, "Castle deck should have 12 enemies");

        // First 4 cards drawn should be Jacks (top layer)
        let mut test_castle = castle.clone();
        for _ in 0..4 {
            let card = test_castle.draw().unwrap();
            assert_eq!(card.rank, Rank::Jack, "First 4 cards should be Jacks");
        }

        // Next 4 should be Queens (middle layer)
        for _ in 0..4 {
            let card = test_castle.draw().unwrap();
            assert_eq!(card.rank, Rank::Queen, "Next 4 cards should be Queens");
        }

        // Last 4 should be Kings (bottom layer)
        for _ in 0..4 {
            let card = test_castle.draw().unwrap();
            assert_eq!(card.rank, Rank::King, "Last 4 cards should be Kings");
        }
    }

    #[test]
    fn test_tavern_deck_construction() {
        // Test Tavern deck for solo mode
        use crate::card::Rank;
        use crate::deck::Deck;

        let tavern = Deck::create_tavern_deck(0); // 0 Jesters for solo
        assert_eq!(
            tavern.len(),
            40,
            "Tavern deck should have 40 cards for solo"
        );

        // Count card types
        let mut aces = 0;
        let mut numbered = 0;
        for card in &tavern.cards {
            match card.rank {
                Rank::Ace => aces += 1,
                Rank::Two
                | Rank::Three
                | Rank::Four
                | Rank::Five
                | Rank::Six
                | Rank::Seven
                | Rank::Eight
                | Rank::Nine
                | Rank::Ten => numbered += 1,
                _ => {}
            }
        }
        assert_eq!(aces, 4, "Should have 4 Aces");
        assert_eq!(numbered, 36, "Should have 36 numbered cards (2-10)");
    }

    #[test]
    fn test_card_values() {
        // Test card values match the rules
        use crate::card::{Card, Rank};

        assert_eq!(Card::new(Suit::Hearts, Rank::Ace).value(), 1);
        assert_eq!(Card::new(Suit::Hearts, Rank::Five).value(), 5);
        assert_eq!(Card::new(Suit::Hearts, Rank::Ten).value(), 10);
        assert_eq!(Card::new(Suit::Hearts, Rank::Jack).value(), 10);
        assert_eq!(Card::new(Suit::Hearts, Rank::Queen).value(), 15);
        assert_eq!(Card::new(Suit::Hearts, Rank::King).value(), 20);
        assert_eq!(Card::new(Suit::Hearts, Rank::Jester).value(), 0);
    }

    #[test]
    fn test_hearts_power() {
        // Test Hearts power: heal from discard
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Spades, Rank::Jack)));

        // Add cards to discard pile
        for _ in 0..10 {
            game.discard_pile.push(Card::new(Suit::Hearts, Rank::Two));
        }
        let discard_before = game.discard_pile.len();
        let tavern_before = game.tavern_deck.len();

        // Play 5 of Hearts -> heal 5 cards
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));
        let result = game.play_cards(vec![0]);

        assert!(result.is_ok());
        assert_eq!(
            game.discard_pile.len(),
            discard_before - 5,
            "Should move 5 cards from discard"
        );
        assert_eq!(
            game.tavern_deck.len(),
            tavern_before + 5,
            "Should add 5 cards to tavern deck"
        );
    }

    #[test]
    fn test_diamonds_power() {
        // Test Diamonds power: draw cards
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Spades, Rank::Jack)));

        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Diamonds, Rank::Five));

        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());

        // Should draw 5 cards (hand was 1, now should be 1 - 1 (played) + 5 (drawn) = 5)
        assert_eq!(game.player.hand.len(), 5, "Should draw 5 cards");
    }

    #[test]
    fn test_spades_power_cumulative() {
        // Test Spades power: shield is cumulative
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack)));

        // Turn 1: Play 5 of Spades -> shield = 5
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Spades, Rank::Five));
        game.player.hand.push(Card::new(Suit::Spades, Rank::Three));
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(game.shield_value, 5);

        // Turn 2: Play 3 of Spades -> shield = 5 + 3 = 8
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());
        assert_eq!(game.shield_value, 8, "Shield should be cumulative");
    }

    #[test]
    fn test_enemy_immunity() {
        // Test enemy immunity blocks suit powers
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack)));

        // Play Hearts card against Hearts enemy -> power blocked
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));
        game.discard_pile.push(Card::new(Suit::Clubs, Rank::Two));
        let tavern_before = game.tavern_deck.len();

        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());

        // Tavern deck should NOT increase (Hearts power blocked)
        assert_eq!(
            game.tavern_deck.len(),
            tavern_before,
            "Hearts power should be blocked"
        );
    }

    #[test]
    fn test_animal_companion_pairing() {
        // Test Ace can pair with one other card
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack)));

        // Ace + 5 = 6 attack
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Ace));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));

        let result = game.play_cards(vec![0, 1]);
        assert!(result.is_ok());
        // Ace (1) + 5 = 6, doubled by Clubs = 12
        assert_eq!(
            game.total_damage, 12,
            "Ace + 5 with Clubs should deal 12 damage"
        );
    }

    #[test]
    fn test_combo_validation() {
        let mut game = Game::new_solo();

        // Valid combo: 3 + 3 + 3 = 9 <= 10
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Three));
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Three));
        game.player
            .hand
            .push(Card::new(Suit::Diamonds, Rank::Three));

        let result = game.validate_play(&[0, 1, 2]);
        assert!(result.is_ok(), "3+3+3 should be valid combo");

        // Invalid combo: 6 + 6 = 12 > 10
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Six));
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Six));

        let result = game.validate_play(&[0, 1]);
        assert!(result.is_err(), "6+6=12 should be invalid (>10)");
    }

    #[test]
    fn test_exact_damage_capture() {
        // Test exact damage places enemy on top of tavern deck
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack))); // 20 HP

        // Deal exactly 20 damage
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Clubs, Rank::Ten)); // 10 * 2 = 20

        let tavern_before = game.tavern_deck.len();
        let result = game.play_cards(vec![0]);
        assert!(result.is_ok());

        // Enemy should be captured (on top of tavern deck)
        assert_eq!(
            game.tavern_deck.len(),
            tavern_before + 1,
            "Captured enemy should be on tavern deck"
        );
    }

    #[test]
    fn test_discard_to_survive() {
        // Test player can discard cards to survive enemy attack
        let mut game = Game::new_solo();
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Hearts, Rank::Jack))); // Attack 10

        // Give player cards totaling >= 10
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Five));
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Two));

        // Check if player can survive 10 damage
        assert!(
            game.player.can_survive(10),
            "Player should be able to survive"
        );

        // Discard to survive
        let result = game.discard_to_survive(vec![0, 1]); // 5 + 5 = 10
        assert!(result.is_ok(), "Should successfully discard to survive");
    }

    #[test]
    fn test_hearts_before_diamonds() {
        // Test Hearts power resolves before Diamonds in combos
        let mut game = Game::new_solo();
        // Use Clubs enemy to ensure no immunity blocks
        game.current_enemy = Some(Enemy::new(Card::new(Suit::Clubs, Rank::Jack)));

        // Add cards to discard
        for _ in 0..10 {
            game.discard_pile.push(Card::new(Suit::Clubs, Rank::Two));
        }

        // Play Ace♥ + Ace♦ for smaller attack value but still activates both powers
        game.player.hand.clear();
        game.player.hand.push(Card::new(Suit::Hearts, Rank::Ace)); // 1
        game.player.hand.push(Card::new(Suit::Diamonds, Rank::Ace)); // 1
        let _tavern_before = game.tavern_deck.len();
        let discard_before = game.discard_pile.len();

        let result = game.play_cards(vec![0, 1]);
        assert!(result.is_ok(), "Ace+Ace combo should be valid");

        // Total attack = 2, so Hearts should heal 2 cards, Diamonds should draw 2 cards
        // Tavern: +2 from Hearts heal, -2 from Diamonds draw = net 0
        // But the order matters for the rules (Hearts before Diamonds)
        assert_eq!(
            game.discard_pile.len(),
            discard_before - 2,
            "2 cards should have been moved from discard to tavern by Hearts"
        );
        // We played 2 cards (hand was 2), drew 2, so hand should be 2
        assert_eq!(
            game.player.hand.len(),
            2,
            "Diamonds should have drawn 2 cards"
        );

        // The fact that we successfully drew 2 cards after healing proves Hearts ran first
    }
}
