mod card;
mod deck;
mod enemy;
mod game;
mod player;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use game::{Game, GameState};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

enum AppState {
    Playing,
    DiscardPhase { required_damage: u8 },
    Victory,
    Defeat,
    RestartConfirmation,
    QuitConfirmation,
}

struct App {
    game: Game,
    selected_cards: Vec<usize>,
    state: AppState,
    show_help: bool,
    log_scroll_offset: usize,
    guide_scroll_offset: usize,
    help_scroll_offset: usize,
}

impl App {
    fn new() -> Self {
        Self {
            game: Game::new_solo(),
            selected_cards: Vec::new(),
            state: AppState::Playing,
            show_help: false,
            log_scroll_offset: 0,
            guide_scroll_offset: 0,
            help_scroll_offset: 0,
        }
    }

    fn scroll_log_up(&mut self) {
        // Scroll up = go back in history (increment offset from bottom)
        let max_scroll = self.game.game_log.len();
        if self.log_scroll_offset < max_scroll {
            self.log_scroll_offset += 1;
        }
    }

    fn scroll_log_down(&mut self) {
        // Scroll down = go forward in history (decrement offset from bottom)
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset -= 1;
        }
    }

    fn reset_log_scroll(&mut self) {
        // Auto-scroll to bottom (latest logs) = offset 0 from bottom
        self.log_scroll_offset = 0;
    }

    fn scroll_guide_up(&mut self) {
        if self.guide_scroll_offset > 0 {
            self.guide_scroll_offset -= 1;
        }
    }

    fn scroll_guide_down(&mut self, max_lines: usize) {
        let max_scroll = max_lines.saturating_sub(10);
        if self.guide_scroll_offset < max_scroll {
            self.guide_scroll_offset += 1;
        }
    }

    fn scroll_help_up(&mut self) {
        if self.help_scroll_offset > 0 {
            self.help_scroll_offset -= 1;
        }
    }

    fn scroll_help_down(&mut self, max_lines: usize) {
        let max_scroll = max_lines.saturating_sub(10);
        if self.help_scroll_offset < max_scroll {
            self.help_scroll_offset += 1;
        }
    }

    fn restart_game(&mut self) {
        self.game = Game::new_solo();
        self.selected_cards.clear();
        self.state = AppState::Playing;
        self.log_scroll_offset = 0;
        self.guide_scroll_offset = 0;
        self.help_scroll_offset = 0;
    }

    fn toggle_card_selection(&mut self, index: usize) {
        if index >= self.game.player.hand_size() {
            return;
        }

        if let Some(pos) = self.selected_cards.iter().position(|&i| i == index) {
            self.selected_cards.remove(pos);
        } else {
            self.selected_cards.push(index);
        }
    }

    fn play_selected_cards(&mut self) {
        if self.selected_cards.is_empty() {
            self.game.log("No cards selected");
            self.reset_log_scroll();
            return;
        }

        // Sort indices for proper removal
        self.selected_cards.sort_unstable();

        match self.game.play_cards(self.selected_cards.clone()) {
            Ok(enemy_defeated) => {
                self.selected_cards.clear();
                self.reset_log_scroll();

                // Check game state
                match self.game.game_state {
                    GameState::Victory => {
                        self.state = AppState::Victory;
                        return;
                    }
                    GameState::Defeat(_) => {
                        self.state = AppState::Defeat;
                        return;
                    }
                    _ => {}
                }

                // Only enemy attacks if no enemy was defeated AND Jester was not played
                // (If enemy was defeated, new enemy appears and waits for player's turn)
                // (If Jester was played, skip Step 4 per rules)
                if !enemy_defeated && !self.game.jester_played_this_turn {
                    // Transition to discard phase (enemy attack)
                    if let Ok(damage) = self.game.enemy_attack() {
                        self.reset_log_scroll();
                        if damage > 0 {
                            // Check if player can survive
                            if !self.game.player.can_survive(damage) {
                                self.state = AppState::Defeat;
                                self.game.game_state =
                                    GameState::Defeat("Cannot survive enemy attack!".to_string());
                            } else {
                                self.state = AppState::DiscardPhase {
                                    required_damage: damage,
                                };
                            }
                        }
                        // If damage is 0, continue to next turn
                    }
                }
            }
            Err(e) => {
                self.game.log(format!("Error: {}", e));
                self.reset_log_scroll();
            }
        }
    }

    fn yield_turn(&mut self) {
        if self.game.yield_turn().is_ok() {
            self.reset_log_scroll();
            // Transition to discard phase
            if let Ok(damage) = self.game.enemy_attack() {
                self.reset_log_scroll();
                if damage > 0 {
                    if !self.game.player.can_survive(damage) {
                        self.state = AppState::Defeat;
                        self.game.game_state =
                            GameState::Defeat("Cannot survive enemy attack!".to_string());
                    } else {
                        self.state = AppState::DiscardPhase {
                            required_damage: damage,
                        };
                    }
                }
            }
        }
    }

    fn discard_selected_cards(&mut self, _required: u8) {
        if self.selected_cards.is_empty() {
            self.game.log("No cards selected to discard");
            self.reset_log_scroll();
            return;
        }

        self.selected_cards.sort_unstable();

        match self.game.discard_to_survive(self.selected_cards.clone()) {
            Ok(_) => {
                self.selected_cards.clear();
                self.reset_log_scroll();
                self.state = AppState::Playing;
                self.game.log("Survived enemy attack! New turn begins.");
                self.reset_log_scroll();
            }
            Err(e) => {
                self.game.log(format!("Error: {}", e));
                self.reset_log_scroll();
            }
        }
    }

    fn use_jester(&mut self) {
        match self.game.use_jester() {
            Ok(_) => {
                self.reset_log_scroll();
            }
            Err(e) => {
                self.game.log(format!("Error: {}", e));
                self.reset_log_scroll();
            }
        }
    }

    fn get_action_prompt(&self) -> String {
        match &self.state {
            AppState::Playing => {
                "⚔️  ATTACK: Select cards (1-8) and press Enter to play, or Space to yield".to_string()
            }
            AppState::DiscardPhase { required_damage } => {
                format!(
                    "🛡️  DEFEND: Enemy attacks! Discard cards worth {} value or more",
                    required_damage
                )
            }
            AppState::Victory => "Press 'r' to Restart or 'q' to Quit".to_string(),
            AppState::Defeat => "Press 'r' to Restart or 'q' to Quit".to_string(),
            AppState::RestartConfirmation => {
                "Restart game? Press 'y' to confirm or 'n' to cancel".to_string()
            }
            AppState::QuitConfirmation => {
                "Quit game? Press 'y' to confirm or 'n' to cancel".to_string()
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            if app.show_help {
                ui::render_help(f, app.help_scroll_offset);
                return;
            }

            let action_prompt = app.get_action_prompt();
            ui::render_game(
                f,
                &app.game,
                &app.selected_cards,
                app.log_scroll_offset,
                app.guide_scroll_offset,
                &action_prompt,
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Global keys
            match key.code {
                KeyCode::Char('q') => {
                    // Transition to quit confirmation instead of immediately quitting
                    app.state = AppState::QuitConfirmation;
                    continue;
                }
                KeyCode::Char('h') => {
                    app.show_help = !app.show_help;
                    // Reset help scroll when closing
                    if !app.show_help {
                        app.help_scroll_offset = 0;
                    }
                    continue;
                }
                KeyCode::Up => {
                    if app.show_help {
                        app.scroll_help_up();
                    } else {
                        app.scroll_log_up();
                    }
                    continue;
                }
                KeyCode::Down => {
                    if app.show_help {
                        let help_line_count = ui::get_help_line_count();
                        app.scroll_help_down(help_line_count);
                    } else {
                        app.scroll_log_down();
                    }
                    continue;
                }
                KeyCode::Left => {
                    if !app.show_help {
                        app.scroll_guide_up();
                    }
                    continue;
                }
                KeyCode::Right => {
                    if !app.show_help {
                        let guide_line_count = ui::get_game_guide_line_count();
                        app.scroll_guide_down(guide_line_count);
                    }
                    continue;
                }
                _ => {}
            }

            // Skip other inputs if help is shown
            if app.show_help {
                continue;
            }

            match &app.state {
                AppState::Playing => match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let digit = c.to_digit(10).unwrap() as usize;
                        // Convert 1-8 to indices 0-7 (1-based numbering for user)
                        if (1..=8).contains(&digit) {
                            app.toggle_card_selection(digit - 1);
                        }
                    }
                    KeyCode::Enter => {
                        app.play_selected_cards();
                    }
                    KeyCode::Char(' ') => {
                        app.yield_turn();
                    }
                    KeyCode::Char('j') => {
                        app.use_jester();
                    }
                    KeyCode::Char('r') => {
                        app.state = AppState::RestartConfirmation;
                    }
                    _ => {}
                },
                AppState::DiscardPhase { required_damage } => match key.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let digit = c.to_digit(10).unwrap() as usize;
                        // Convert 1-8 to indices 0-7 (1-based numbering for user)
                        if (1..=8).contains(&digit) {
                            app.toggle_card_selection(digit - 1);
                        }
                    }
                    KeyCode::Enter => {
                        app.discard_selected_cards(*required_damage);
                    }
                    KeyCode::Char('j') => {
                        // Solo mode: Use Jester power during discard phase (Step 4)
                        app.use_jester();
                    }
                    KeyCode::Char('r') => {
                        app.state = AppState::RestartConfirmation;
                    }
                    _ => {}
                },
                AppState::Victory | AppState::Defeat => {
                    if key.code == KeyCode::Char('r') {
                        app.restart_game();
                    }
                }
                AppState::RestartConfirmation => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        app.restart_game();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Return to previous state - we'll just set to Playing
                        app.state = AppState::Playing;
                    }
                    _ => {}
                },
                AppState::QuitConfirmation => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        return Ok(()); // Actually quit the game
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Return to Playing state
                        app.state = AppState::Playing;
                    }
                    _ => {}
                },
            }
        }
    }
}
