# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Kingslayer (Regicide TUI)** is a Terminal User Interface implementation of the cooperative card game Regicide. The project is being developed in Rust with a two-phase approach:

- **Phase 1 (MVP)**: Solo play against AI ✅ **COMPLETE** (v0.2.3)
- **Phase 2 (Expansion)**: LAN-based multiplayer for 2-4 players (Planned)

### Current Status (v0.2.3)

Phase 1 is fully implemented with:
- Complete solo gameplay with all game mechanics
- Enhanced 3-row TUI layout with real-time clock
- All suit powers (Hearts, Diamonds, Clubs, Spades) working correctly
- Jester mechanics with immunity cancellation
- Victory ranking system (Bronze/Silver/Gold)
- Comprehensive test coverage (~20+ tests)
- 4 critical bug fixes implemented and tested
- Attack/defend visual indicators
- Scrollable game log and help screens
- Quit confirmation dialog

## Tech Stack

- **Language**: Rust
- **TUI Framework**: Ratatui 0.28 (implemented)
- **Terminal Control**: Crossterm 0.28 (implemented)
- **Date/Time**: Chrono 0.4 (implemented for clock display)
- **Randomization**: Rand 0.8 (implemented for deck shuffling)
- **Serialization**: Serde + Serde JSON 1.0 (implemented, ready for Phase 2)
- **Networking** (Phase 2): Tokio with standard TCP sockets (planned)

## Core Architecture

### Game State Structure

The game is structured around several core entities that must maintain strict separation of concerns:

1. **Castle Deck (Enemy Deck)**: Layered construction with 4 Kings (bottom), 4 Queens (middle), 4 Jacks (top). Suits within layers are randomized. The top card is always the current enemy.

2. **Tavern Deck (Player Deck)**: Standard 52-card deck (Ace-10 only, face cards removed) + Jesters. Jester count varies by player count (0 for solo, 0 for 2-player, 1 for 3-player, 2 for 4-player).

3. **Hand Management**: Maximum hand size varies by player count (8 solo, 7 for 2-player, 6 for 3-player, 5 for 4-player).

### Game Loop Architecture

Each turn follows a strict 4-step sequence:

1. **Input Phase**: Validate card play (single card, OR combo of 2-4 matching ranks totaling ≤10, OR Ace + any card)
2. **Resolution Phase**: Check immunity, apply suit powers, deal damage
3. **Victory/Defeat Check**: Compare damage to enemy HP
4. **Enemy Attack Phase**: Player must discard cards with value ≥ enemy attack

### Card System

#### Suit Powers (Critical Game Mechanics)

- **Hearts (Heal)**: Shuffle discard pile, move N cards from discard to bottom of Tavern deck (N = attack value)
- **Diamonds (Draw)**: Draw N cards distributed among players (N = attack value)
- **Clubs (Double Damage)**: Attack value counts ×2 against enemy HP
- **Spades (Shield)**: Reduce enemy attack value for current turn; effects are cumulative and persist until enemy defeated

#### Special Cards

- **Ace (Animal Companion)**: Value 1. Can pair with any other card (except Jester) to combine values and activate both suits.
- **Jester**: Value 0. Cancels enemy immunity. Skips enemy attack phase. Player chooses who goes next. Temporarily changes communication rules.

#### Enemy Stats

- Jack: 20 HP, 10 Attack
- Queen: 30 HP, 15 Attack
- King: 40 HP, 20 Attack

#### Enemy Immunity

Each enemy is immune to suit powers (NOT damage) of cards matching their suit. The Jester cancels this immunity. Note: Spades played before Jester against Spades enemy retroactively apply; Clubs played before Jester against Clubs enemy do NOT count double.

### Critical Game Logic Rules

1. **Exact Damage Victory**: If damage exactly equals enemy HP, enemy is captured (placed face-down on top of Tavern deck). Otherwise, enemy goes to discard pile.

2. **Combo Resolution**: When multiple suit powers trigger, Hearts always resolves before Diamonds.

3. **Defeated Enemy Cards**: Jacks drawn = 10 value, Queens = 15, Kings = 20 when played or discarded.

4. **Yielding**: Players can yield (skip to Step 4) unless all other players yielded on their last turn.

5. **Loss Conditions**:
   - Player cannot discard enough cards to satisfy enemy damage
   - Player cannot play a card or yield on their turn

## TUI Layout Design

The terminal is divided into three rows (implemented in v0.2.x):

**Row 1** - Three columns:
1. **The Castle (Left)**: Kingslayer logo with real-time clock, current enemy card (ASCII art), HP bar (e.g., `[||||||||||]`), attack stat
2. **The Battlefield (Middle)**: Currently played cards, active shield value, damage capability, deck counts, action prompts with visual indicators (⚔️ attack, 🛡️ defend)
3. **Game Log (Right)**: Scrollable event log (e.g., "Player played 5 of Hearts. Healed 5 cards.")

**Row 2** - Full width:
4. **Hand**: Player's cards with selection indicators (inverted colors for selected cards), card numbers (1-8), and values

**Row 3** - Two columns:
5. **Keyboard Actions (Left)**: List of available keyboard commands
6. **Game Rules Guide (Right)**: Scrollable quick reference for suit powers and game mechanics

### Visual Constraints

- Strict ASCII/Unicode text only
- No graphical assets or copyrighted character art
- Card rendering example:
```
.-------.
| 5     |
|   ♥   |
|     5 |
'-------'
```
- Use color codes: Red for Hearts/Diamonds, Blue/White for Spades/Clubs
- Visual indicators implemented:
  - ⚔️ for attack phase prompts
  - 🛡️ for defend/discard phase prompts
  - Inverted colors for selected cards
  - HP bars with visual fill indicators
  - Kingslayer logo (⚔ KINGSLAYER ⚔)

## Phase 2: Multiplayer Architecture

### Network Model

- **Host**: Runs game logic authority, listens on port 5555, displays local IP
- **Clients**: Connect via TCP, send input events, receive state objects

### Communication Protocol

Message types (JSON-serialized):
- `HANDSHAKE`: Client sends name; host assigns player ID
- `GAME_STATE`: Host sends full board state
- `PLAYER_ACTION`: Client sends action (e.g., `{"action": "PLAY", "cards": ["Ace of Spades", "King of Hearts"]}`)
- `ERROR`: Host sends invalid move notification

### Multiplayer-Specific Rules

Turn order is clockwise. TUI must display "Waiting for Player X..." when not local user's turn.

## Development Sequence

Phase 1 implementation (v0.1.0 - v0.2.3):

1. **Data Structures** ✅: Implemented `Card`, `Deck`, `Player`, `Enemy` structs. Built `Deck::shuffle()` and castle construction.
   - Location: `src/card.rs`, `src/deck.rs`, `src/enemy.rs`, `src/player.rs`

2. **TUI Skeleton** ✅: Initialized Ratatui, created enhanced 3-row layout, implemented ASCII card renderer.
   - Location: `src/ui.rs`, `src/main.rs`

3. **Solo Game Loop** ✅: Wired input handling to game logic, implemented "discard to survive" calculation.
   - Location: `src/game.rs`, `src/main.rs`
   - Includes 4 critical bug fixes for game mechanics

4. **Testing** ✅: Added comprehensive unit test coverage (~20+ tests).
   - Location: `src/game.rs` (test module)

5. **Polish** ✅: Added colors, ASCII art, visual indicators, clock display, scrolling.
   - Location: `src/ui.rs`

Phase 2 (upcoming):

6. **Network Layer**: Build Server (Host) and Client classes, implement JSON GameState serializer.
7. **Multiplayer Logic**: Implement turn-based rotation, player state management.

## Key Implementation Notes

- **Immunity Logic**: Must carefully track which suit powers trigger vs. are blocked. Damage always applies regardless of immunity.
- **Shield Persistence**: Shield effects from Spades are cumulative and persist until the enemy is defeated, not just for one turn.
- **Combo Validation**: When validating combos, ensure total value ≤10 and all cards have same rank (except Animal Companion combos).
- **Jester Timing**: When Jester is played against Spades enemy, previously played Spades begin working retroactively.
- **Communication Rules**: In multiplayer, players cannot reveal hand contents. Exception: After Jester is played, players may express desire to go next ("I have a good play" allowed; "I have a 10 of Clubs" forbidden).
- **Terminal Resizing**: TUI should handle terminal resizing gracefully without breaking layout.
- **Client Disconnection**: If a client disconnects, host should pause the game.

## Critical Bug Fixes (v0.2.x)

The following bugs were identified and fixed during Phase 1 development:

### Defect #1: Jester Enemy Attack Skip
**Issue**: When Jester was played, the enemy attack phase (Step 4) was not being skipped correctly.
**Fix**: Modified game logic to properly skip Step 4 when Jester is played. The game now transitions directly from Step 3 to the next player's turn.
**Location**: `src/game.rs` - Jester play handling

### Defect #2: Solo Jester Timing
**Issue**: In solo mode, the Jester power could only be activated at the start of Step 1, not during Step 4 (discard phase) as per rules.
**Fix**: Added `can_use_jester` check and allowed Jester activation during both Step 1 and Step 4.
**Location**: `src/game.rs` - Solo Jester power implementation

### Defect #3: Spades Retroactive Application
**Issue**: When Jester cancelled a Spades enemy's immunity, previously played Spades were not being counted retroactively.
**Fix**: Implemented tracking of Spades played before Jester, with retroactive application once immunity is cancelled. Shield values are recalculated when Jester is played against Spades enemies.
**Location**: `src/game.rs` - Shield application logic

### Defect #4: Clubs Power Persistence
**Issue**: Clubs double damage effect was persisting across turns instead of applying only to the current turn.
**Fix**: Ensured Clubs multiplier only applies during damage calculation for the current attack, not stored as persistent state. The `clubs_multiplier` is calculated fresh each turn based on cards played.
**Location**: `src/game.rs` - Damage calculation

## Testing

As of v0.2.3, the project includes comprehensive test coverage (~20+ unit tests) covering:
- Card and deck operations (shuffling, drawing, construction)
- Enemy state management and immunity
- Game state transitions and turn flow
- Suit power application (Hearts, Diamonds, Clubs, Spades)
- Jester mechanics and immunity cancellation
- Victory and defeat conditions
- Shield persistence and retroactive application
- Combo validation

Run tests with: `cargo test`

## Game Balance Data

Solo difficulty levels are determined by Jester usage:
- 0 Jesters used = Gold Victory
- 1 Jester used = Silver Victory
- 2 Jesters used = Bronze Victory

In solo mode, Jester power: "Discard hand and refill to 8 cards (doesn't count as drawing for immunity)". Can activate at start of Step 1 or Step 4.
