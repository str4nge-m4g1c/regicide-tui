use crate::card::Card;
use crate::game::Game;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Render the main game UI with 2 rows: Top row (Castle/Battlefield/Log), Bottom row (Hand)
pub fn render_game(f: &mut Frame, game: &Game, selected_cards: &[usize]) {
    // Split into top and bottom rows
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Top row
            Constraint::Percentage(40), // Bottom row (Hand)
        ])
        .split(f.area());

    // Split top row into 3 columns: Castle, Battlefield, Log
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Castle (left)
            Constraint::Percentage(40), // Battlefield (middle)
            Constraint::Percentage(30), // Log (right)
        ])
        .split(main_chunks[0]);

    // Render each pane
    render_castle(f, top_chunks[0], game);
    render_battlefield(f, top_chunks[1], game);
    render_log(f, top_chunks[2], game);
    render_hand(f, main_chunks[1], game, selected_cards);
}

/// Render the Castle pane (current enemy)
fn render_castle(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default()
        .title("⚔ The Castle ⚔")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(ref enemy) = game.current_enemy {
        let enemy_card = render_card_large(&enemy.card);
        let hp_bar = enemy.hp_bar(30);
        let stats = format!(
            "HP: {}/{} {}  Attack: {} (After Shields: {})",
            enemy.current_hp,
            enemy.max_hp,
            hp_bar,
            enemy.attack,
            enemy.get_attack_after_shields(game.shield_value)
        );

        let mut text = Text::from(enemy_card);
        text.push_line("");
        text.push_line(Span::styled(stats, Style::default().fg(Color::Red)));

        if enemy.immunity_cancelled {
            text.push_line(Span::styled(
                "⚠ Immunity Cancelled",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            text.push_line(Span::styled(
                format!(
                    "Immune to: {}",
                    match enemy.card.suit {
                        crate::card::Suit::Hearts => "Hearts ♥",
                        crate::card::Suit::Diamonds => "Diamonds ♦",
                        crate::card::Suit::Clubs => "Clubs ♣",
                        crate::card::Suit::Spades => "Spades ♠",
                    }
                ),
                Style::default().fg(Color::Gray),
            ));
        }

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No enemy")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
    }
}

/// Render the Battlefield pane (played cards, shields, damage)
fn render_battlefield(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default()
        .title("⚡ The Battlefield ⚡")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let mut text = Text::default();

    if !game.played_cards.is_empty() {
        let cards: Vec<String> = game.played_cards.iter().map(|c| c.display()).collect();
        text.push_line(Span::styled(
            format!("Played: {}", cards.join(" ")),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
    }

    text.push_line("");
    text.push_line(Span::styled(
        format!("🛡 Active Shield: {}", game.shield_value),
        Style::default().fg(Color::Blue),
    ));
    text.push_line(Span::styled(
        format!("⚔ Total Damage: {}", game.total_damage),
        Style::default().fg(Color::Red),
    ));

    text.push_line("");
    text.push_line(Span::styled(
        format!("Tavern Deck: {} cards", game.tavern_deck.len()),
        Style::default().fg(Color::Green),
    ));
    text.push_line(Span::styled(
        format!("Discard Pile: {} cards", game.discard_pile.len()),
        Style::default().fg(Color::Gray),
    ));
    text.push_line(Span::styled(
        format!("Castle Deck: {} enemies", game.castle_deck.len()),
        Style::default().fg(Color::Yellow),
    ));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
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
fn render_log(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default()
        .title("📜 Game Log")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let log_items: Vec<ListItem> = game
        .game_log
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|msg| ListItem::new(msg.clone()))
        .collect();

    let list = List::new(log_items).block(block);

    f.render_widget(list, area);
}

/// Render a card in large ASCII art format
fn render_card_large(card: &Card) -> String {
    let rank = card.rank.display();
    let suit = card.suit.symbol();

    format!(
        ".-------.\n\
         | {:<5} |\n\
         |       |\n\
         |   {}   |\n\
         |       |\n\
         | {:>5} |\n\
         '-------'",
        rank, suit, rank
    )
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

/// Render victory screen
pub fn render_victory(f: &mut Frame, game: &Game) {
    let block = Block::default()
        .title("🎉 VICTORY! 🎉")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let rank = game.victory_rank();
    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "All enemies have been defeated!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            format!("Victory Rank: {} ⭐", rank),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("Jesters Used: {}/{}", game.jesters_used, game.jester_count),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from("Press 'q' to quit"),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, f.area());
}

/// Render defeat screen
pub fn render_defeat(f: &mut Frame, reason: &str) {
    let block = Block::default()
        .title("💀 DEFEAT 💀")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "You have been defeated!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(reason, Style::default().fg(Color::White))),
        Line::from(""),
        Line::from("Press 'q' to quit"),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, f.area());
}

/// Render help overlay
pub fn render_help(f: &mut Frame) {
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let text = Text::from(vec![
        Line::from("Controls:"),
        Line::from("  1-8: Toggle card selection"),
        Line::from("  Enter: Play selected cards"),
        Line::from("  Space: Yield turn"),
        Line::from("  j: Use Jester power (solo mode)"),
        Line::from("  h: Toggle help"),
        Line::from("  q: Quit"),
        Line::from(""),
        Line::from("Game Rules:"),
        Line::from("  ♥ Hearts: Heal discard into deck"),
        Line::from("  ♦ Diamonds: Draw cards"),
        Line::from("  ♣ Clubs: Double damage"),
        Line::from("  ♠ Spades: Shield against attack"),
        Line::from(""),
        Line::from("Press 'h' to close help"),
    ]);

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    // Create a centered area
    let area = centered_rect(60, 70, f.area());
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
