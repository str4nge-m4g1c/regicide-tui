use crate::card::Card;
use crate::game::Game;
use chrono::Local;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

/// Render the main game UI with 3 rows
pub fn render_game(
    f: &mut Frame,
    game: &Game,
    selected_cards: &[usize],
    log_scroll_offset: usize,
    guide_scroll_offset: usize,
    action_prompt: &str,
) {
    // Split into 3 rows
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45), // Row 1 (Castle/Battlefield/Log)
            Constraint::Percentage(30), // Row 2 (Hand)
            Constraint::Percentage(25), // Row 3 (Controls/Guide)
        ])
        .split(f.area());

    // Split row 1 into 3 columns: Castle, Battlefield, Log
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Castle (left)
            Constraint::Percentage(40), // Battlefield (middle)
            Constraint::Percentage(30), // Log (right)
        ])
        .split(main_chunks[0]);

    // Split row 3 into 2 columns: Keyboard Actions, Game Rules
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Keyboard Actions (left)
            Constraint::Percentage(50), // Game Rules Guide (right)
        ])
        .split(main_chunks[2]);

    // Render each pane
    render_castle(f, top_chunks[0], game);
    render_battlefield(f, top_chunks[1], game, action_prompt);
    render_log(f, top_chunks[2], game, log_scroll_offset);
    render_hand(f, main_chunks[1], game, selected_cards);
    render_keyboard_actions(f, bottom_chunks[0]);
    render_game_guide(f, bottom_chunks[1], guide_scroll_offset);
}

/// Render the Castle pane (current enemy) with logo and clock on top
fn render_castle(f: &mut Frame, area: Rect, game: &Game) {
    // Split into top row (logo + date/time) and castle
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Top row (logo and date/time side by side)
            Constraint::Min(10),   // Castle
        ])
        .split(area);

    // Split top row into logo and date/time side by side
    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Logo
            Constraint::Percentage(50), // Date/Time
        ])
        .split(chunks[0]);

    // Render logo with padding
    let logo_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let logo_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "⚔ KINGSLAYER ⚔",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    let logo_text = Paragraph::new(Text::from(logo_lines))
        .block(logo_block)
        .alignment(Alignment::Center);
    f.render_widget(logo_text, top_row[0]);

    // Render date/time with padding
    let now = Local::now();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%H:%M:%S").to_string();
    let clock_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));
    let clock_lines = vec![
        Line::from(Span::styled("Date:", Style::default().fg(Color::Gray))),
        Line::from(Span::styled(
            date_str,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            time_str,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    let clock_text = Paragraph::new(Text::from(clock_lines))
        .block(clock_block)
        .alignment(Alignment::Center);
    f.render_widget(clock_text, top_row[1]);

    // Render castle
    let block = Block::default()
        .title("⚔ The Castle ⚔")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(ref enemy) = game.current_enemy {
        let hp_bar = enemy.hp_bar(20);

        let mut text_lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                enemy.name(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("HP: {}/{}", enemy.current_hp, enemy.max_hp),
                Style::default().fg(Color::Red),
            )),
            Line::from(Span::styled(hp_bar, Style::default().fg(Color::Red))),
            Line::from(""),
            Line::from(Span::styled(
                format!("Attack: {}", enemy.attack),
                Style::default().fg(Color::Red),
            )),
            Line::from(Span::styled(
                format!(
                    "(After Shields: {})",
                    enemy.get_attack_after_shields(game.shield_value)
                ),
                Style::default().fg(Color::Blue),
            )),
            Line::from(""),
        ];

        if enemy.immunity_cancelled {
            text_lines.push(Line::from(Span::styled(
                "⚠ Immunity Cancelled",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            let immune_suit = match enemy.card.suit {
                crate::card::Suit::Hearts => "Hearts ♥",
                crate::card::Suit::Diamonds => "Diamonds ♦",
                crate::card::Suit::Clubs => "Clubs ♣",
                crate::card::Suit::Spades => "Spades ♠",
            };
            text_lines.push(Line::from(Span::styled(
                format!("Immune: {}", immune_suit),
                Style::default().fg(Color::Gray),
            )));
        }

        let paragraph = Paragraph::new(Text::from(text_lines))
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, chunks[1]);
    } else {
        let paragraph = Paragraph::new("No enemy")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, chunks[1]);
    }
}

/// Render the Battlefield pane (played cards, shields, damage)
fn render_battlefield(f: &mut Frame, area: Rect, game: &Game, action_prompt: &str) {
    // Split battlefield into stats rows, battlefield, and action prompt
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Deck stats row (castle/discard/tavern)
            Constraint::Length(4), // Combat stats row (shield/damage) - same size as deck stats
            Constraint::Min(8),    // Battlefield area - takes remaining space
            Constraint::Length(7), // Action prompt frame - increased padding
        ])
        .split(area);

    // Top row: 3 panels for decks
    let deck_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33), // Castle Deck
            Constraint::Percentage(34), // Discard Pile
            Constraint::Percentage(33), // Tavern Deck
        ])
        .split(chunks[0]);

    // Render Castle Deck
    let castle_deck_block = Block::default()
        .title("Castle")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let castle_deck_text = Paragraph::new(Span::styled(
        format!("{} enemies", game.castle_deck.len()),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
    .block(castle_deck_block)
    .alignment(Alignment::Center);
    f.render_widget(castle_deck_text, deck_row[0]);

    // Render Discard Pile
    let discard_block = Block::default()
        .title("Discard")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let discard_text = Paragraph::new(Span::styled(
        format!("{} cards", game.discard_pile.len()),
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    ))
    .block(discard_block)
    .alignment(Alignment::Center);
    f.render_widget(discard_text, deck_row[1]);

    // Render Tavern Deck
    let tavern_block = Block::default()
        .title("Tavern")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));
    let tavern_text = Paragraph::new(Span::styled(
        format!("{} cards", game.tavern_deck.len()),
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
    ))
    .block(tavern_block)
    .alignment(Alignment::Center);
    f.render_widget(tavern_text, deck_row[2]);

    // Second row: 2 panels for combat stats (same height as deck stats)
    let combat_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Active Shield
            Constraint::Percentage(50), // Total Damage
        ])
        .split(chunks[1]);

    // Render Active Shield
    let shield_block = Block::default()
        .title("🛡 Shield")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));
    let shield_text = Paragraph::new(Span::styled(
        format!("{}", game.shield_value),
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD),
    ))
    .block(shield_block)
    .alignment(Alignment::Center);
    f.render_widget(shield_text, combat_row[0]);

    // Render Total Damage
    let damage_block = Block::default()
        .title("⚔ Damage")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));
    let damage_text = Paragraph::new(Span::styled(
        format!("{}", game.total_damage),
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    ))
    .block(damage_block)
    .alignment(Alignment::Center);
    f.render_widget(damage_text, combat_row[1]);

    // Render battlefield area (takes remaining space)
    let played_block = Block::default()
        .title("⚡ The Battlefield ⚡")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let played_text = if !game.played_cards.is_empty() {
        let cards: Vec<String> = game.played_cards.iter().map(|c| c.display()).collect();
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Played Cards:",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                cards.join(" "),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )),
        ])
    } else {
        Text::from(vec![
            Line::from(""),
            Line::from(Span::styled(
                "No cards played yet",
                Style::default().fg(Color::Gray),
            )),
        ])
    };

    let played_paragraph = Paragraph::new(played_text)
        .block(played_block)
        .alignment(Alignment::Center);

    f.render_widget(played_paragraph, chunks[2]);

    // Render action prompt at bottom with colored double-line border
    // Determine border color based on action type
    let border_color = if action_prompt.contains("ATTACK") {
        Color::Red
    } else if action_prompt.contains("DEFEND") {
        Color::Blue
    } else {
        Color::Yellow
    };

    // Clean up the text by removing emoji icons
    let text = action_prompt.replace("⚔️  ", "").replace("🛡️  ", "");

    let prompt_block = Block::default()
        .title("⚡ Next Action ⚡")
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(border_color));

    // Render text with padding
    let text_lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
    ];
    let text_paragraph = Paragraph::new(Text::from(text_lines))
        .block(prompt_block)
        .alignment(Alignment::Center)
        .wrap(ratatui::widgets::Wrap { trim: true });
    f.render_widget(text_paragraph, chunks[3]);
}

/// Render the Hand pane (player's cards)
fn render_hand(f: &mut Frame, area: Rect, game: &Game, selected_cards: &[usize]) {
    let block = Block::default()
        .title(format!(
            "🃏 Your Hand ({}/{}) | Jesters: {}/{}",
            game.player.hand_size(),
            game.player.max_hand_size,
            game.jester_count - game.jesters_used,
            game.jester_count
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    if game.player.hand.is_empty() {
        let paragraph = Paragraph::new("No cards in hand")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    // Generate ASCII art for each card
    let card_arts: Vec<Vec<String>> = game.player.hand.iter().map(render_card_small).collect();

    // Number of lines in a card (should be 5)
    let card_height = 5;

    // Build lines by concatenating each line from all cards horizontally
    let mut text_lines = vec![];

    for line_idx in 0..card_height {
        let mut line_spans = vec![];

        for (card_idx, card_art) in card_arts.iter().enumerate() {
            let is_selected = selected_cards.contains(&card_idx);

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                let card = &game.player.hand[card_idx];
                let color = if card.suit.is_red() {
                    Color::Red
                } else {
                    Color::White
                };
                Style::default().fg(color)
            };

            line_spans.push(Span::styled(card_art[line_idx].clone(), style));
            line_spans.push(Span::raw(" ")); // Space between cards
        }

        text_lines.push(Line::from(line_spans));
    }

    // Add index line below cards (1-based numbering)
    // Each card is 8 chars wide, so index should also be 8 chars
    let mut index_spans = vec![];
    for card_idx in 0..game.player.hand.len() {
        let is_selected = selected_cards.contains(&card_idx);
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        // 1-based index, centered in 8 characters
        let index_str = format!("  [{}]  ", card_idx + 1);
        index_spans.push(Span::styled(format!("{:<8}", index_str), style));
        index_spans.push(Span::raw(" ")); // Space between cards
    }
    text_lines.push(Line::from(index_spans));

    // Add value line below indices
    // Each value label should also be 8 chars wide to match
    let mut value_spans = vec![];
    for card in &game.player.hand {
        let value_str = format!("Val:{}", card.value());
        value_spans.push(Span::styled(
            format!("{:^8}", value_str), // Center the value in 8 chars
            Style::default().fg(Color::Cyan),
        ));
        value_spans.push(Span::raw(" ")); // Space between cards
    }
    text_lines.push(Line::from(value_spans));

    let paragraph = Paragraph::new(Text::from(text_lines))
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render the Log pane (game events)
fn render_log(f: &mut Frame, area: Rect, game: &Game, scroll_offset: usize) {
    let block = Block::default()
        .title("📜 Game Log (↑↓ to scroll)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    // Calculate how many lines can fit in the area (subtract 2 for borders)
    let available_height = area.height.saturating_sub(2) as usize;
    let max_items = available_height.min(100);

    // Get log items with scroll offset from bottom
    // scroll_offset = 0 means show the newest messages
    // scroll_offset > 0 means scroll back in history
    let total_logs = game.game_log.len();
    let end_idx = total_logs.saturating_sub(scroll_offset);
    let start_idx = end_idx.saturating_sub(max_items);

    let log_items: Vec<ListItem> = game.game_log[start_idx..end_idx]
        .iter()
        .map(|msg| ListItem::new(msg.clone()))
        .collect();

    let list = List::new(log_items).block(block);

    f.render_widget(list, area);
}

/// Render a card in compact ASCII art format (for hand display)
/// Returns a vector of strings, one for each line of the card
fn render_card_small(card: &Card) -> Vec<String> {
    let rank = card.rank.display();
    let suit = card.suit.symbol();

    vec![
        ".------.".to_string(),
        format!("|{:<4}  |", rank),
        format!("|  {}   |", suit),
        format!("|  {:>4}|", rank),
        "'------'".to_string(),
    ]
}

/// Render help overlay (scrollable)
pub fn render_help(f: &mut Frame, scroll_offset: usize) {
    let block = Block::default()
        .title("📖 Complete Game Guide (↑↓ to scroll, h to close) 📖")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Build comprehensive help content
    let all_lines = vec![
        Line::from(Span::styled(
            "CONTROLS:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  1-8: Toggle card selection"),
        Line::from("  Enter: Play selected cards"),
        Line::from("  Space: Yield turn"),
        Line::from("  j: Use Jester power (solo mode only)"),
        Line::from("  ↑/↓: Scroll game log (or this help)"),
        Line::from("  ←/→: Scroll game guide"),
        Line::from("  r: Restart game"),
        Line::from("  h: Toggle help overlay"),
        Line::from("  q: Quit game"),
        Line::from(""),
        Line::from(Span::styled(
            "GAME OBJECTIVE:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Defeat all 12 enemies (4 Jacks, 4 Queens, 4 Kings)"),
        Line::from("  to win the game!"),
        Line::from(""),
        Line::from(Span::styled(
            "TURN STRUCTURE:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  1. INPUT: Play card(s) or yield"),
        Line::from("  2. RESOLUTION: Apply damage and suit powers"),
        Line::from("  3. VICTORY/DEFEAT: Check if enemy is defeated"),
        Line::from("  4. ENEMY ATTACK: Discard cards to survive"),
        Line::from(""),
        Line::from(Span::styled(
            "SUIT POWERS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "♥ Hearts - Heal:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Shuffle discard pile and move N cards from"),
        Line::from("  discard to bottom of tavern deck (N = attack value)"),
        Line::from("  CRITICAL: Hearts always resolves before Diamonds!"),
        Line::from(""),
        Line::from(Span::styled(
            "♦ Diamonds - Draw:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Draw N cards from tavern deck (N = attack value)"),
        Line::from("  In multiplayer: Cards distributed among players"),
        Line::from(""),
        Line::from(Span::styled(
            "♣ Clubs - Double Damage:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Attack value counts ×2 against enemy HP"),
        Line::from("  NOTE: If played before Jester against Clubs enemy,"),
        Line::from("        does NOT count double retroactively"),
        Line::from(""),
        Line::from(Span::styled(
            "♠ Spades - Shield:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Reduce enemy attack by N (N = attack value)"),
        Line::from("  Shield effects are CUMULATIVE and PERSISTENT"),
        Line::from("  until the enemy is defeated"),
        Line::from("  NOTE: If played before Jester against Spades enemy,"),
        Line::from("        shields apply retroactively after Jester"),
        Line::from(""),
        Line::from(Span::styled(
            "SPECIAL CARDS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Ace (Animal Companion):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Value: 1"),
        Line::from("  Can pair with any other card (except Jester)"),
        Line::from("  Combines values and activates both suits"),
        Line::from("  Example: Ace of Hearts + 5 of Diamonds = 6 damage,"),
        Line::from("           heal 1, draw 5"),
        Line::from(""),
        Line::from(Span::styled(
            "Jester:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Value: 0"),
        Line::from("  Cancels enemy immunity to suit powers"),
        Line::from("  Skips enemy attack phase (you take no damage!)"),
        Line::from("  Must be played alone (no combos)"),
        Line::from("  Solo mode special: Discard hand, draw 8 cards (j key)"),
        Line::from("  Can activate at start of Step 1 or Step 4"),
        Line::from(""),
        Line::from(Span::styled(
            "COMBO RULES:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Single card: Always valid"),
        Line::from("  Ace + any card: Valid (combines values & suits)"),
        Line::from("  2-4 cards same rank: Valid if total value ≤ 10"),
        Line::from("  Examples:"),
        Line::from("    - Four 2s = 8 total (valid)"),
        Line::from("    - Three 3s = 9 total (valid)"),
        Line::from("    - Two 6s = 12 total (INVALID - exceeds 10)"),
        Line::from(""),
        Line::from(Span::styled(
            "ENEMY STATS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Jack:  20 HP, 10 Attack"),
        Line::from("  Queen: 30 HP, 15 Attack"),
        Line::from("  King:  40 HP, 20 Attack"),
        Line::from(""),
        Line::from(Span::styled(
            "ENEMY IMMUNITY:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Each enemy is IMMUNE to suit powers (NOT damage)"),
        Line::from("  matching their suit"),
        Line::from("  Jester cancels this immunity"),
        Line::from("  Damage always applies regardless of immunity!"),
        Line::from(""),
        Line::from(Span::styled(
            "CRITICAL GAME RULES:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Exact Damage Victory:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  If damage EXACTLY equals enemy HP, enemy is captured"),
        Line::from("  (placed face-down on top of tavern deck)"),
        Line::from("  Otherwise, enemy goes to discard pile"),
        Line::from(""),
        Line::from(Span::styled(
            "Defeated Enemy Cards:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  When drawn or played/discarded:"),
        Line::from("    - Jacks = 10 value"),
        Line::from("    - Queens = 15 value"),
        Line::from("    - Kings = 20 value"),
        Line::from(""),
        Line::from(Span::styled(
            "Yielding:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Skip to Step 4 (enemy attack phase)"),
        Line::from("  CANNOT yield if all other players yielded on"),
        Line::from("  their last turn (multiplayer)"),
        Line::from(""),
        Line::from(Span::styled(
            "VICTORY CONDITIONS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Defeat all 12 enemies (4 Jacks, 4 Queens, 4 Kings)"),
        Line::from(""),
        Line::from(Span::styled(
            "Solo Victory Grades:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Gold Victory: 0 Jesters used"),
        Line::from("  Silver Victory: 1 Jester used"),
        Line::from("  Bronze Victory: 2 Jesters used"),
        Line::from(""),
        Line::from(Span::styled(
            "DEFEAT CONDITIONS:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Cannot discard enough cards to survive enemy attack"),
        Line::from("  Cannot play a card or yield on your turn"),
        Line::from(""),
        Line::from(Span::styled(
            "HAND MANAGEMENT:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Maximum hand size varies by player count:"),
        Line::from("    - Solo: 8 cards"),
        Line::from("    - 2 players: 7 cards each"),
        Line::from("    - 3 players: 6 cards each"),
        Line::from("    - 4 players: 5 cards each"),
        Line::from(""),
        Line::from(Span::styled(
            "DECK STRUCTURE:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Castle Deck (Enemies):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Layered construction:"),
        Line::from("    - Bottom: 4 Kings (suits randomized)"),
        Line::from("    - Middle: 4 Queens (suits randomized)"),
        Line::from("    - Top: 4 Jacks (suits randomized)"),
        Line::from("  Top card is always the current enemy"),
        Line::from(""),
        Line::from(Span::styled(
            "Tavern Deck (Player Deck):",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Standard 52-card deck (Ace-10 only, face cards removed)"),
        Line::from("  Jester count by player count:"),
        Line::from("    - Solo: 2 Jesters"),
        Line::from("    - 2 players: 1 Jester"),
        Line::from("    - 3 players: 0 Jesters"),
        Line::from("    - 4 players: 0 Jesters"),
        Line::from(""),
        Line::from(Span::styled(
            "Press 'h' to close this guide",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    // Create a centered area (larger than before to show more content)
    let area = centered_rect(80, 90, f.area());

    // Calculate how many lines can fit (subtract 2 for borders)
    let available_height = area.height.saturating_sub(2) as usize;
    let total_lines = all_lines.len();
    let start_idx = scroll_offset.min(total_lines.saturating_sub(available_height));
    let end_idx = (start_idx + available_height).min(total_lines);

    let visible_lines: Vec<Line> = all_lines[start_idx..end_idx].to_vec();

    let paragraph = Paragraph::new(Text::from(visible_lines))
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render the keyboard actions pane
fn render_keyboard_actions(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("⌨ Keyboard Controls ⌨")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let text = Text::from(vec![
        Line::from(Span::styled(
            "Card Selection:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  1-8: Toggle card selection"),
        Line::from("  Enter: Play selected cards"),
        Line::from("  Space: Yield turn"),
        Line::from("  j: Use Jester power"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  ↑/↓: Scroll game log"),
        Line::from("  ←/→: Scroll game guide"),
        Line::from(""),
        Line::from(Span::styled(
            "Other:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  r: Restart game"),
        Line::from("  h: Toggle help overlay"),
        Line::from("  q: Quit game"),
    ]);

    let paragraph = Paragraph::new(text).block(block).alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Render the game rules guide pane (scrollable)
fn render_game_guide(f: &mut Frame, area: Rect, scroll_offset: usize) {
    let block = Block::default()
        .title("📖 Game Guide (←/→ to scroll) 📖")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    // Build the full game guide content
    let all_lines = vec![
        Line::from(Span::styled(
            "SUIT POWERS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "♥ Hearts - Heal:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Shuffle discard pile and move N cards"),
        Line::from("  from discard to bottom of tavern deck"),
        Line::from("  (N = attack value)"),
        Line::from(""),
        Line::from(Span::styled(
            "♦ Diamonds - Draw:",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from("  Draw N cards (N = attack value)"),
        Line::from(""),
        Line::from(Span::styled(
            "♣ Clubs - Double Damage:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Attack value counts ×2 against enemy HP"),
        Line::from(""),
        Line::from(Span::styled(
            "♠ Spades - Shield:",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Reduce enemy attack by N (N = attack value)"),
        Line::from("  Shield effects are cumulative!"),
        Line::from(""),
        Line::from(Span::styled(
            "SPECIAL CARDS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Ace (Animal Companion):",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Value: 1"),
        Line::from("  Can pair with any other card (except Jester)"),
        Line::from("  Combines values and activates both suits"),
        Line::from(""),
        Line::from(Span::styled(
            "Jester:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Value: 0"),
        Line::from("  Cancels enemy immunity to suit powers"),
        Line::from("  Skips enemy attack phase"),
        Line::from("  Must be played alone"),
        Line::from("  Solo mode: Discard hand, draw 8 cards (j key)"),
        Line::from(""),
        Line::from(Span::styled(
            "COMBO RULES:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Single card: Always valid"),
        Line::from("  Ace + one card: Valid combination"),
        Line::from("  2-4 cards of same rank: Total value ≤ 10"),
        Line::from(""),
        Line::from(Span::styled(
            "ENEMY STATS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Jack:  20 HP, 10 Attack"),
        Line::from("  Queen: 30 HP, 15 Attack"),
        Line::from("  King:  40 HP, 20 Attack"),
        Line::from(""),
        Line::from(Span::styled(
            "VICTORY CONDITIONS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Defeat all 12 enemies to win!"),
        Line::from("  Exact damage = Enemy captured (face-down on deck)"),
        Line::from("  Otherwise, enemy goes to discard pile"),
        Line::from(""),
        Line::from(Span::styled(
            "DEFEAT CONDITIONS:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  Cannot discard enough to survive enemy attack"),
        Line::from("  Cannot play a card or yield on your turn"),
    ];

    // Calculate how many lines can fit in the area (subtract 2 for borders)
    let available_height = area.height.saturating_sub(2) as usize;
    let total_lines = all_lines.len();
    let start_idx = scroll_offset.min(total_lines.saturating_sub(available_height));
    let end_idx = (start_idx + available_height).min(total_lines);

    let visible_lines: Vec<Line> = all_lines[start_idx..end_idx].to_vec();

    let paragraph = Paragraph::new(Text::from(visible_lines))
        .block(block)
        .alignment(Alignment::Left);

    f.render_widget(paragraph, area);
}

/// Get the total number of lines in the game guide (for scrolling)
pub fn get_game_guide_line_count() -> usize {
    64 // Total lines in the game guide
}

/// Get the total number of lines in the help overlay (for scrolling)
pub fn get_help_line_count() -> usize {
    177 // Total lines in the help overlay
}
