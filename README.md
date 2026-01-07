[![Release](https://github.com/str4nge-m4g1c/kingslayer/actions/workflows/release.yml/badge.svg)](https://github.com/str4nge-m4g1c/kingslayer/actions/workflows/release.yml)

# Kingslayer (Regicide TUI)

A Terminal User Interface (TUI) implementation of the cooperative card game **Regicide**, built in Rust with Ratatui.

## About Regicide

Regicide is a cooperative card game where players work together to defeat 12 powerful enemies (4 Jacks, 4 Queens, and 4 Kings). Players use numbered cards and special Animal Companions (Aces) to attack enemies while managing their hand and surviving enemy counterattacks.

## Features

- **Phase 1 (Complete)**: Solo play mode against AI
- Full game logic implementation with all card suit powers
- Beautiful ASCII TUI with 4-pane layout
- Color-coded cards and game state
- Victory ranking system (Bronze/Silver/Gold)
- Phase 2 (Planned): LAN-based multiplayer for 2-4 players

## Installation

### From Release Binaries

Download the latest release for your platform from the [Releases page](https://github.com/str4nge-m4g1c/regicide-tui/releases):

- **macOS (Apple Silicon)**: `kingslayer-macos-aarch64.tar.gz`
- **macOS (Intel)**: `kingslayer-macos-x86_64.tar.gz`
- **Linux (x86_64)**: `kingslayer-linux-x86_64.tar.gz`
- **Windows (x86_64)**: `kingslayer-windows-x86_64.zip`

Extract and run:

```bash
# macOS/Linux
tar -xzf kingslayer-*.tar.gz
./kingslayer

# Move to PATH (optional)
sudo mv kingslayer /usr/local/bin/
```

### From Source

**Prerequisites:**

- Rust 1.88+ (run `rustup update` to upgrade)
- A terminal with Unicode support

```bash
# Clone the repository
git clone https://github.com/str4nge-m4g1c/regicide-tui.git
cd regicide-tui

# Build in release mode for optimal performance
cargo build --release

# Run the game
./target/release/kingslayer

# Or install to cargo bin
cargo install --path .
```

## How to Play

### Running the Game

```bash
# Or run directly with cargo
cargo run --release

# Or run the compiled binary
./target/release/kingslayer
```

### Controls

- **1-8**: Toggle card selection (select/deselect cards by index)
- **Enter**: Play selected cards / Confirm discard
- **Space**: Yield turn (skip to enemy attack)
- **j**: Use Jester power (solo mode - discard hand and draw fresh)
- **h**: Toggle help overlay
- **q**: Quit game

### Game Layout

The game interface is divided into two rows:

**Top Row (Castle, Battlefield, Game Log):**

- **Castle (Left)**: Current enemy card with HP bar and attack stats
- **Battlefield (Middle)**: Active shields, total damage, and deck counts
- **Game Log (Right)**: Recent game events and actions

**Bottom Row (Your Hand):**

- ASCII art cards displayed horizontally
- Card numbers (1-8) shown below each card
- Card values displayed at the bottom

### Game Rules

Each turn consists of 4 steps:

1. **Input Phase**: Select and play card(s) from your hand
2. **Resolution Phase**: Suit powers activate and damage is applied
3. **Victory/Defeat Check**: Did you defeat the enemy?
4. **Enemy Attack Phase**: Survive by discarding cards

#### Suit Powers

- **♥ Hearts (Heal)**: Shuffle discard pile and move N cards to bottom of deck
- **♦ Diamonds (Draw)**: Draw N cards from the deck
- **♣ Clubs (Double Damage)**: Deal 2x damage to enemy
- **♠ Spades (Shield)**: Reduce enemy attack by N (cumulative)

*N = attack value of cards played*

#### Playing Cards

- **Single Card**: Play any card
- **Combo**: Play 2-4 cards of the same rank (total value ≤ 10)
- **Animal Companion (Ace)**: Value 1, can pair with any one other card to combine values and activate both suits
- **Jester**: Value 0, cancels enemy immunity, skips enemy attack, lets you choose who goes next

#### Enemy Mechanics

- **Immunity**: Each enemy is immune to their suit (e.g., Jack of Hearts is immune to Hearts powers, but damage still applies)
- **Exact Damage**: Defeat an enemy with exactly their HP to capture them (added to top of deck, worth more when drawn!)
- **Overkill**: Excess damage sends enemy to discard pile

| Enemy | HP | Attack |
|-------|----|----|
| Jack  | 20 | 10 |
| Queen | 30 | 15 |
| King  | 40 | 20 |

#### Defeat Conditions

- Cannot discard enough cards to survive enemy attack
- Cannot play any cards or yield

### Solo Mode

In solo mode, you start with 2 Jester powers that let you discard your hand and draw fresh cards. Your victory rank depends on Jester usage:

- **Gold Victory**: 0 Jesters used
- **Silver Victory**: 1 Jester used
- **Bronze Victory**: 2 Jesters used

## Development

See [CLAUDE.md](CLAUDE.md) for detailed architecture and development notes.

### Project Structure

```
src/
├── main.rs      # Application entry point and event loop
├── game.rs      # Core game state and logic
├── card.rs      # Card, Suit, and Rank definitions
├── deck.rs      # Deck operations and construction
├── enemy.rs     # Enemy state and behavior
├── player.rs    # Player state and hand management
└── ui.rs        # Ratatui UI rendering
```

### Running Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

## Roadmap

### Phase 1: Solo Play ✅ (Complete)

- [x] Core game mechanics and rules
- [x] Full suit power implementation (Hearts, Diamonds, Clubs, Spades)
- [x] Enemy AI with immunity system
- [x] Jester power for solo mode
- [x] Victory ranking system (Bronze/Silver/Gold)
- [x] ASCII card rendering with proper layout
- [x] Game state management and turn flow
- [x] Terminal UI with 2-row layout (Castle/Battlefield/Log, Hand)

### Phase 2: Multiplayer (Planned)

- [ ] **Network Layer**
  - [ ] TCP-based networking with Tokio
  - [ ] Host/Client architecture
  - [ ] Game state synchronization via JSON
  - [ ] Connection management and error handling

- [ ] **Multiplayer Game Logic**
  - [ ] Turn-based player rotation (2-4 players)
  - [ ] Communication rules enforcement
  - [ ] Jester count adjustment per player count (0/0/1/2 for 1/2/3/4 players)
  - [ ] Hand size adjustment per player count (8/7/6/5 for 1/2/3/4 players)

- [ ] **UI Enhancements**
  - [ ] Player list with turn indicators
  - [ ] "Waiting for Player X..." states
  - [ ] Other players' hand counts display
  - [ ] Host lobby with connection info (IP/Port)

### Phase 3: Polish & Quality of Life (Future)

- [ ] Save/Load game state
- [ ] Game replay system
- [ ] Statistics tracking (wins, losses, average Jester usage)
- [ ] Customizable themes and colors
- [ ] Sound effects and notifications
- [ ] Tutorial mode for new players
- [ ] Difficulty settings (fewer/more Jesters)
- [ ] Spectator mode for multiplayer games

### Phase 4: Advanced Features (Future)

- [ ] Online matchmaking (beyond LAN)
- [ ] Replay sharing
- [ ] Tournament mode
- [ ] Custom rule variants
- [ ] Achievements system
- [ ] Leaderboards

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Regicide** card game designed by Paul Abrahams, Luke Badger, and Andy Richdale
- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the TUI
- ASCII art card rendering inspired by classic terminal games

