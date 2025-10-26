# Chess Engine: M5 Implementation Plan

**Milestone:** Time Management & UCI Protocol
**Date:** 2025-10-26
**Status:** 📋 Planning
**Estimated Duration:** 2-3 weeks

---

## Overview

M5 focuses on making the engine **playable** and **interoperable** with chess GUIs. This includes:

1. **Time Management:** Intelligent allocation of search time per move
2. **UCI Protocol:** Full Universal Chess Interface compliance
3. **Real-world Integration:** Works with Arena, CuteChess, Lichess, etc.

---

## Current State (Post-M4)

### ✅ What We Have

- Complete search engine with advanced optimizations
- Depth 9-10 reachable in ~5 seconds
- 66% node reduction vs M3
- 220 tests passing
- Strong tactical play

### ❌ What's Missing

- **No time management** - searches to fixed depth only
- **No UCI protocol** - can't communicate with chess GUIs
- **No real-time control** - can't stop search on command
- **No configurable options** - fixed hash size, etc.
- **No info output** - can't show search progress

### 🎯 Goal

Create a **UCI-compliant** chess engine that:

- Respects time controls (doesn't lose on time)
- Communicates with chess GUIs
- Provides real-time search information
- Can be stopped immediately when requested

---

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│           UCI Binary (uci_main.rs)          │
│  - Parse UCI commands from stdin            │
│  - Send UCI responses to stdout             │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│           UCI Handler (uci.rs)              │
│  - Command parsing                          │
│  - Position management                      │
│  - Search control                           │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│      Time Manager (time.rs)                 │
│  - Calculate time per move                  │
│  - Soft/hard time limits                    │
│  - Emergency time handling                  │
└─────────────┬───────────────────────────────┘
              │
┌─────────────▼───────────────────────────────┐
│         Searcher (search.rs)                │
│  - Check time limits during search          │
│  - Output info messages                     │
│  - Respond to stop command                  │
└─────────────────────────────────────────────┘
```

---

## Session Breakdown

### Session 1-2: Time Management Infrastructure (2-3 days)

**Goal:** Calculate how much time to spend per move

#### Features

**1. Time Control Types**

```rust
#[derive(Debug, Clone)]
pub enum TimeControl {
    /// Infinite time (analysis mode)
    Infinite,

    /// Fixed time per move
    MoveTime { millis: u64 },

    /// Clock-based time control
    Clock {
        wtime: u64,        // White's remaining time (ms)
        btime: u64,        // Black's remaining time (ms)
        winc: u64,         // White's increment (ms)
        binc: u64,         // Black's increment (ms)
        movestogo: Option<u32>, // Moves until time control
    },

    /// Fixed depth
    Depth { depth: u32 },

    /// Fixed node count
    Nodes { nodes: u64 },
}
```

**2. Time Allocation Strategy**

```rust
pub struct TimeManager {
    /// Soft limit: try to finish search by this time
    soft_limit: Option<Instant>,

    /// Hard limit: must stop search by this time
    hard_limit: Option<Instant>,

    /// Start time of current search
    start_time: Instant,
}

impl TimeManager {
    /// Calculate time allocation for a move
    pub fn allocate_time(&mut self, tc: &TimeControl, side: Color) -> TimeAllocation {
        match tc {
            TimeControl::MoveTime { millis } => {
                // Simple: use exactly the specified time
                TimeAllocation {
                    soft_limit: *millis,
                    hard_limit: *millis,
                }
            }

            TimeControl::Clock { wtime, btime, winc, binc, movestogo } => {
                let my_time = if side == Color::White { *wtime } else { *btime };
                let my_inc = if side == Color::White { *winc } else { *binc };

                // Calculate base time per move
                let base_time = if let Some(mtg) = movestogo {
                    // X moves until time control
                    my_time / (*mtg as u64).max(1)
                } else {
                    // Sudden death: estimate ~40 moves remaining
                    my_time / 40
                };

                // Add increment (use 80% to build time)
                let allocated = base_time + (my_inc * 80 / 100);

                // Soft limit: 80% of allocated time
                let soft = allocated * 80 / 100;

                // Hard limit: allocated time (with safety margin)
                let hard = allocated.min(my_time - 100);

                TimeAllocation {
                    soft_limit: soft,
                    hard_limit: hard,
                }
            }

            _ => TimeAllocation::infinite(),
        }
    }

    /// Check if soft limit has been exceeded
    pub fn should_stop_soft(&self) -> bool {
        if let Some(limit) = self.soft_limit {
            self.start_time.elapsed() >= limit
        } else {
            false
        }
    }

    /// Check if hard limit has been exceeded (emergency stop)
    pub fn should_stop_hard(&self) -> bool {
        if let Some(limit) = self.hard_limit {
            self.start_time.elapsed() >= limit
        } else {
            false
        }
    }
}
```

**3. Adaptive Time Management**

- **Extend time** if score drops significantly
- **Reduce time** if position is stable
- **Emergency mode** when time is critically low

#### Files to Create

```
crates/engine/src/
├── time.rs                (NEW - 300 lines)
│   ├── TimeControl enum
│   ├── TimeManager struct
│   └── TimeAllocation struct
└── tests/time_tests.rs    (NEW - 100 lines)
```

#### Success Criteria

- ✅ Correctly allocates time for different time controls
- ✅ Respects soft and hard time limits
- ✅ Handles sudden death (no movestogo)
- ✅ Handles increment-based time controls
- ✅ Emergency time handling works

#### Tests

```rust
#[test]
fn test_movetime_simple() {
    let tc = TimeControl::MoveTime { millis: 5000 };
    let alloc = TimeManager::allocate_time(&tc, Color::White);
    assert_eq!(alloc.soft_limit, 5000);
    assert_eq!(alloc.hard_limit, 5000);
}

#[test]
fn test_clock_with_movestogo() {
    let tc = TimeControl::Clock {
        wtime: 60_000,  // 60 seconds
        btime: 60_000,
        winc: 1000,     // 1 second increment
        binc: 1000,
        movestogo: Some(20),  // 20 moves to next time control
    };
    let alloc = TimeManager::allocate_time(&tc, Color::White);
    // Should allocate ~3000ms base + 800ms increment = ~3800ms
    assert!(alloc.soft_limit > 2500);
    assert!(alloc.soft_limit < 4000);
}

#[test]
fn test_sudden_death() {
    let tc = TimeControl::Clock {
        wtime: 60_000,
        btime: 60_000,
        winc: 0,
        binc: 0,
        movestogo: None,  // Sudden death
    };
    let alloc = TimeManager::allocate_time(&tc, Color::White);
    // Should allocate ~1500ms (60s / 40 moves)
    assert!(alloc.soft_limit > 1000);
    assert!(alloc.soft_limit < 2000);
}
```

---

### Session 3-4: Search Integration (2-3 days)

**Goal:** Integrate time management into search

#### Features

**1. Search Limits**

```rust
#[derive(Debug, Clone)]
pub struct SearchLimits {
    pub time_control: TimeControl,
    pub side_to_move: Color,
}

impl Searcher {
    pub fn search_with_limits(&mut self, board: &Board, limits: SearchLimits) -> SearchResult {
        let mut time_mgr = TimeManager::new();
        let allocation = time_mgr.allocate_time(&limits.time_control, limits.side_to_move);

        // Set time limits
        time_mgr.set_limits(allocation);

        // Iterative deepening with time checks
        for depth in 1..=MAX_DEPTH {
            // Check hard limit before starting new iteration
            if time_mgr.should_stop_hard() {
                break;
            }

            let score = self.search_root(board, depth);

            // Check soft limit after completing iteration
            if time_mgr.should_stop_soft() && depth >= 4 {
                break; // Stop if we've done at least depth 4
            }
        }

        // Return result
    }
}
```

**2. Time Checks During Search**

```rust
impl Searcher {
    fn negamax(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply: u32) -> i32 {
        self.nodes += 1;

        // Check time every 4096 nodes
        if self.nodes & 0xFFF == 0 {
            if self.time_mgr.should_stop_hard() {
                self.stopped = true;
                return 0; // Return immediately
            }
        }

        if self.stopped {
            return 0;
        }

        // ... rest of negamax
    }
}
```

**3. Graceful Stop**

```rust
pub struct Searcher {
    stopped: AtomicBool,  // Can be set from another thread
    // ...
}

impl Searcher {
    pub fn stop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }
}
```

#### Files to Modify

```
crates/engine/src/
├── search.rs              (MODIFY - add time checks)
└── tests/search_tests.rs  (MODIFY - add time limit tests)
```

#### Success Criteria

- ✅ Search respects time limits
- ✅ Stops within 50ms of hard limit
- ✅ Completes at least depth 4 before soft limit
- ✅ Can be stopped immediately with stop()
- ✅ Returns valid move even when stopped

---

### Session 5-7: UCI Protocol Implementation (3-4 days)

**Goal:** Full UCI protocol support

#### UCI Commands to Implement

**1. Identification**

```
>>> uci
<<< id name ChessAI 0.1.0
<<< id author YourName
<<< option name Hash type spin default 64 min 1 max 1024
<<< option name Threads type spin default 1 min 1 max 1
<<< option name MultiPV type spin default 1 min 1 max 10
<<< uciok
```

**2. Readiness**

```
>>> isready
<<< readyok
```

**3. New Game**

```
>>> ucinewgame
(Clear hash table, reset state)
```

**4. Position Setup**

```
>>> position startpos
>>> position startpos moves e2e4 e7e5
>>> position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
>>> position fen <fen> moves e2e4 e7e5
```

**5. Go Command**

```
>>> go infinite
>>> go movetime 5000
>>> go wtime 60000 btime 60000 winc 1000 binc 1000
>>> go wtime 60000 btime 60000 movestogo 20
>>> go depth 10
>>> go nodes 1000000
>>> go mate 3
```

**6. Stop Command**

```
>>> stop
<<< bestmove e2e4 ponder e7e5
```

**7. Info Output**

```
<<< info depth 6 seldepth 8 score cp 25 nodes 125000 nps 1250000 time 100 pv e2e4 e7e5 g1f3
<<< info depth 7 seldepth 9 score cp 30 nodes 450000 nps 1200000 time 375 pv e2e4 e7e5 g1f3 b8c6
```

**8. Options**

```
>>> setoption name Hash value 128
>>> setoption name Threads value 1
>>> setoption name MultiPV value 3
```

#### Implementation

```rust
// crates/engine/src/uci.rs

pub struct UciHandler {
    board: Board,
    searcher: Searcher,
    options: UciOptions,
}

#[derive(Debug, Clone)]
pub struct UciOptions {
    pub hash_size_mb: usize,
    pub threads: usize,
    pub multi_pv: usize,
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            board: Board::startpos(),
            searcher: Searcher::new(),
            options: UciOptions::default(),
        }
    }

    pub fn handle_command(&mut self, cmd: &str) -> Option<String> {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();

        match parts.get(0) {
            Some(&"uci") => self.handle_uci(),
            Some(&"isready") => Some("readyok".to_string()),
            Some(&"ucinewgame") => self.handle_new_game(),
            Some(&"position") => self.handle_position(&parts[1..]),
            Some(&"go") => self.handle_go(&parts[1..]),
            Some(&"stop") => self.handle_stop(),
            Some(&"setoption") => self.handle_setoption(&parts[1..]),
            Some(&"quit") => None, // Signal to exit
            _ => Some(format!("Unknown command: {}", cmd)),
        }
    }

    fn handle_uci(&self) -> Option<String> {
        let mut response = String::new();
        response.push_str("id name ChessAI 0.1.0\n");
        response.push_str("id author YourName\n");
        response.push_str("option name Hash type spin default 64 min 1 max 1024\n");
        response.push_str("option name Threads type spin default 1 min 1 max 1\n");
        response.push_str("option name MultiPV type spin default 1 min 1 max 10\n");
        response.push_str("uciok");
        Some(response)
    }

    fn handle_position(&mut self, args: &[&str]) -> Option<String> {
        if args.is_empty() {
            return Some("Invalid position command".to_string());
        }

        match args[0] {
            "startpos" => {
                self.board = Board::startpos();
                // Apply moves if present
                if let Some(idx) = args.iter().position(|&x| x == "moves") {
                    self.apply_moves(&args[idx + 1..]);
                }
            }
            "fen" => {
                // Find where FEN ends (either "moves" or end of args)
                let moves_idx = args.iter().position(|&x| x == "moves");
                let fen_end = moves_idx.unwrap_or(args.len());
                let fen = args[1..fen_end].join(" ");

                self.board = parse_fen(&fen).ok()?;

                // Apply moves if present
                if let Some(idx) = moves_idx {
                    self.apply_moves(&args[idx + 1..]);
                }
            }
            _ => return Some("Invalid position command".to_string()),
        }

        None // No response needed
    }

    fn apply_moves(&mut self, moves: &[&str]) {
        for move_str in moves {
            if let Some(m) = self.parse_move(move_str) {
                self.board.make_move(m);
            }
        }
    }

    fn parse_move(&self, move_str: &str) -> Option<Move> {
        // Parse UCI move format (e.g., "e2e4", "e7e8q")
        // ...
    }

    fn handle_go(&mut self, args: &[&str]) -> Option<String> {
        // Parse go arguments
        let tc = self.parse_time_control(args);

        // Start search
        let limits = SearchLimits {
            time_control: tc,
            side_to_move: self.board.side_to_move(),
        };

        let result = self.searcher.search_with_limits(&self.board, limits);

        // Send bestmove
        let bestmove = result.best_move.to_uci();
        let ponder = result.pv.get(1).map(|m| m.to_uci()).unwrap_or_default();

        if ponder.is_empty() {
            Some(format!("bestmove {}", bestmove))
        } else {
            Some(format!("bestmove {} ponder {}", bestmove, ponder))
        }
    }

    fn parse_time_control(&self, args: &[&str]) -> TimeControl {
        let mut i = 0;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;
        let mut movestogo = None;
        let mut movetime = None;
        let mut depth = None;

        while i < args.len() {
            match args[i] {
                "infinite" => return TimeControl::Infinite,
                "movetime" => {
                    movetime = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "depth" => {
                    depth = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "wtime" => {
                    wtime = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "btime" => {
                    btime = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "winc" => {
                    winc = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "binc" => {
                    binc = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                "movestogo" => {
                    movestogo = args.get(i + 1).and_then(|s| s.parse().ok());
                    i += 2;
                }
                _ => i += 1,
            }
        }

        // Construct TimeControl
        if let Some(mt) = movetime {
            TimeControl::MoveTime { millis: mt }
        } else if let Some(d) = depth {
            TimeControl::Depth { depth: d }
        } else if let Some(wt) = wtime {
            TimeControl::Clock {
                wtime: wt,
                btime: btime.unwrap_or(wt),
                winc: winc.unwrap_or(0),
                binc: binc.unwrap_or(0),
                movestogo,
            }
        } else {
            TimeControl::Infinite
        }
    }
}
```

#### Binary Entry Point

```rust
// crates/engine/bin/uci_main.rs

use std::io::{self, BufRead};
use engine::uci::UciHandler;

fn main() {
    let mut handler = UciHandler::new();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if let Some(response) = handler.handle_command(&line) {
            println!("{}", response);
        }

        if line.trim() == "quit" {
            break;
        }
    }
}
```

#### Files to Create

```
crates/engine/src/
├── uci.rs                 (NEW - 600 lines)
│   ├── UciHandler struct
│   ├── Command parsing
│   └── Response generation
├── bin/
│   └── uci_main.rs        (NEW - 50 lines)
└── tests/uci_tests.rs     (NEW - 200 lines)
```

#### Success Criteria

- ✅ Passes UCI protocol tests
- ✅ Works with Arena GUI
- ✅ Works with CuteChess CLI
- ✅ Responds to stop within 50ms
- ✅ Outputs info messages during search
- ✅ Handles all go command variants

---

### Session 8-9: Info Output During Search (2 days)

**Goal:** Provide real-time search information

#### Features

**1. Info Messages**

```
info depth 6 seldepth 8 score cp 25 nodes 125000 nps 1250000 time 100 hashfull 150 tbhits 0 pv e2e4 e7e5 g1f3
```

**Components:**

- `depth`: Current search depth
- `seldepth`: Selective search depth (max depth reached in any line)
- `score cp`: Score in centipawns (or `score mate N`)
- `nodes`: Nodes searched
- `nps`: Nodes per second
- `time`: Time elapsed (ms)
- `hashfull`: Hash table fullness (permille, 0-1000)
- `tbhits`: Tablebase hits (future)
- `pv`: Principal variation

**2. Implementation**

```rust
impl Searcher {
    fn search_with_limits(&mut self, board: &Board, limits: SearchLimits) -> SearchResult {
        // ...

        for depth in 1..=MAX_DEPTH {
            let start = Instant::now();
            let score = self.search_root(board, depth);
            let elapsed = start.elapsed().as_millis() as u64;

            // Output info
            let pv = self.extract_pv(board, depth);
            self.output_info(SearchInfo {
                depth,
                seldepth: self.seldepth,
                score,
                nodes: self.nodes,
                time: elapsed,
                pv,
            });

            // Check time
            if time_mgr.should_stop_soft() {
                break;
            }
        }
    }

    fn output_info(&self, info: SearchInfo) {
        let nps = if info.time > 0 {
            info.nodes * 1000 / info.time
        } else {
            0
        };

        let hashfull = self.tt.hashfull();

        let score_str = if info.score.abs() > MATE_SCORE - 100 {
            let mate_in = (MATE_SCORE - info.score.abs()) / 2;
            format!("mate {}", if info.score > 0 { mate_in } else { -mate_in })
        } else {
            format!("cp {}", info.score)
        };

        let pv_str = info.pv.iter()
            .map(|m| m.to_uci())
            .collect::<Vec<_>>()
            .join(" ");

        println!(
            "info depth {} seldepth {} score {} nodes {} nps {} time {} hashfull {} pv {}",
            info.depth,
            info.seldepth,
            score_str,
            info.nodes,
            nps,
            info.time,
            hashfull,
            pv_str
        );
    }
}
```

**3. Selective Depth Tracking**

```rust
impl Searcher {
    seldepth: u32,  // Maximum depth reached

    fn negamax(&mut self, board: &Board, depth: i32, alpha: i32, beta: i32, ply: u32) -> i32 {
        self.seldepth = self.seldepth.max(ply);
        // ...
    }
}
```

#### Files to Modify

```
crates/engine/src/
├── search.rs              (MODIFY - add info output)
└── uci.rs                 (MODIFY - capture info output)
```

#### Success Criteria

- ✅ Info output every iteration
- ✅ Accurate node counts and NPS
- ✅ Correct PV display
- ✅ Mate scores displayed correctly
- ✅ Hash fullness reported

---

### Session 10: UCI Options (1 day)

**Goal:** Configurable engine options

#### Options to Implement

```
option name Hash type spin default 64 min 1 max 1024
option name Threads type spin default 1 min 1 max 1
option name MultiPV type spin default 1 min 1 max 10
```

#### Implementation

```rust
impl UciHandler {
    fn handle_setoption(&mut self, args: &[&str]) -> Option<String> {
        // Parse: setoption name <name> value <value>
        let name_idx = args.iter().position(|&x| x == "name")?;
        let value_idx = args.iter().position(|&x| x == "value")?;

        let name = args[name_idx + 1..value_idx].join(" ");
        let value = args[value_idx + 1..].join(" ");

        match name.as_str() {
            "Hash" => {
                if let Ok(size) = value.parse::<usize>() {
                    self.options.hash_size_mb = size.clamp(1, 1024);
                    self.searcher = Searcher::with_tt_size(self.options.hash_size_mb);
                }
            }
            "Threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.options.threads = threads.clamp(1, 1);
                }
            }
            "MultiPV" => {
                if let Ok(multipv) = value.parse::<usize>() {
                    self.options.multi_pv = multipv.clamp(1, 10);
                }
            }
            _ => {}
        }

        None
    }
}
```

---

### Session 11: Testing & Integration (2 days)

**Goal:** Comprehensive testing with real chess GUIs

#### Test Suite

**1. Unit Tests**

```rust
#[test]
fn test_uci_command_uci() {
    let handler = UciHandler::new();
    let response = handler.handle_command("uci").unwrap();
    assert!(response.contains("id name"));
    assert!(response.contains("uciok"));
}

#[test]
fn test_position_startpos() {
    let mut handler = UciHandler::new();
    handler.handle_command("position startpos");
    // Verify board is at starting position
}

#[test]
fn test_position_with_moves() {
    let mut handler = UciHandler::new();
    handler.handle_command("position startpos moves e2e4 e7e5");
    // Verify moves were applied
}

#[test]
fn test_go_movetime() {
    let mut handler = UciHandler::new();
    handler.handle_command("position startpos");
    let start = Instant::now();
    handler.handle_command("go movetime 1000");
    let elapsed = start.elapsed().as_millis();
    // Should take approximately 1000ms
    assert!(elapsed > 900 && elapsed < 1100);
}

#[test]
fn test_stop_command() {
    let mut handler = UciHandler::new();
    handler.handle_command("position startpos");
    // Start search in another thread
    // Send stop command
    // Verify search stops within 50ms
}
```

**2. Integration Tests**

```bash
# Test with CuteChess CLI
cutechess-cli -engine cmd=./target/release/chess-ai \
  -engine cmd=stockfish \
  -each tc=40/60 -rounds 10

# Test with Arena
# 1. Add engine in Arena GUI
# 2. Play games at different time controls
# 3. Verify no crashes or time losses

# Test with Lichess
# Run engine with fishnet or lichess-bot
```

**3. Protocol Compliance**

```bash
# Use UCI protocol tester
# Verify all required commands work
# Check for protocol violations
```

#### Files to Create

```
crates/engine/tests/
├── integration_uci.rs     (NEW - 300 lines)
└── protocol_tests.rs      (NEW - 200 lines)
```

---

## Success Criteria

### Must Have ✅

- [ ] Time management works correctly for all time control types
- [ ] UCI protocol fully implemented
- [ ] Works with Arena GUI
- [ ] Works with CuteChess CLI
- [ ] Respects time controls (no time losses)
- [ ] Responds to stop within 50ms
- [ ] Info output during search
- [ ] Configurable options (Hash, Threads, MultiPV)
- [ ] All tests passing (250+ tests)

### Nice to Have 🎯

- [ ] Works with Lichess (via lichess-bot)
- [ ] Adaptive time management (extend on score drop)
- [ ] Pondering support (think on opponent's time)
- [ ] searchmoves restriction
- [ ] mate command support

---

## Timeline

### Optimistic (2 weeks)

- Sessions 1-2: Time Management (2 days)
- Sessions 3-4: Search Integration (2 days)
- Sessions 5-7: UCI Protocol (3 days)
- Session 8-9: Info Output (2 days)
- Session 10: Options (1 day)
- Session 11: Testing (2 days)

### Realistic (3 weeks)

- Sessions 1-2: Time Management (3 days)
- Sessions 3-4: Search Integration (3 days)
- Sessions 5-7: UCI Protocol (4 days)
- Session 8-9: Info Output (2 days)
- Session 10: Options (1 day)
- Session 11: Testing (3 days)

### Pessimistic (4 weeks)

- Add 1 week buffer for debugging and edge cases

---

## Risks & Mitigation

### Risk 1: Thread Safety Issues

**Mitigation:** Use atomic types for stopped flag, careful with Arc/Mutex

### Risk 2: Time Management Too Conservative

**Mitigation:** Tunable parameters, testing with different time controls

### Risk 3: UCI Protocol Edge Cases

**Mitigation:** Extensive testing with multiple GUIs, protocol validator

### Risk 4: Performance Regression

**Mitigation:** Benchmark before/after, minimize overhead in search

---

## Deliverables

1. **Working UCI binary** (`target/release/chess-ai`)
2. **Time management module** (`time.rs`)
3. **UCI handler** (`uci.rs`)
4. **Integration tests** (GUI compatibility verified)
5. **Documentation** (M5_COMPLETION.md)

---

## After M5

With M5 complete, the engine will be:

- ✅ **Playable** - Works with chess GUIs
- ✅ **Competitive** - Can play timed games
- ✅ **Observable** - Shows search progress
- ✅ **Configurable** - Adjustable options

**Next steps:**

- M6: Advanced Evaluation (stronger positional understanding)
- M7: Opening Book & Tablebases (chess knowledge)
- M8: Tuning & Testing (strength optimization)

---

**Document Version:** 1.0
**Created:** 2025-10-26
**Status:** Ready to implement
