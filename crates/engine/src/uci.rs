//! UCI (Universal Chess Interface) protocol implementation.

use crate::board::Board;
use crate::io::parse_fen;
use crate::r#move::Move;
use crate::search::{SearchResult, Searcher};
use crate::square::Square;
use crate::time::TimeControl;

/// UCI options configurable by GUI.
#[derive(Debug, Clone)]
pub struct UciOptions {
    pub hash_size_mb: usize,
    pub threads: usize,
    pub multi_pv: usize,
}

impl Default for UciOptions {
    fn default() -> Self {
        Self {
            hash_size_mb: 64,
            threads: 1,
            multi_pv: 1,
        }
    }
}

/// Main UCI protocol handler.
pub struct UciHandler {
    board: Board,
    searcher: Searcher,
    options: UciOptions,
}

impl UciHandler {
    /// Create a new UCI handler.
    pub fn new() -> Self {
        Self {
            board: Board::startpos(),
            searcher: Searcher::new(),
            options: UciOptions::default(),
        }
    }

    /// Handle a UCI command and return optional response.
    ///
    /// Returns None for commands that don't require a response,
    /// or Some(response) for commands that do.
    pub fn handle_command(&mut self, cmd: &str) -> Option<String> {
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "uci" => self.handle_uci(),
            "isready" => Some("readyok".to_string()),
            "ucinewgame" => self.handle_new_game(),
            "position" => self.handle_position(&parts[1..]),
            "go" => self.handle_go(&parts[1..]),
            "stop" => Some("bestmove 0000".to_string()), // Placeholder for now
            "setoption" => self.handle_setoption(&parts[1..]),
            "quit" => None,
            _ => None, // Ignore unknown commands
        }
    }

    /// Handle "uci" command - send identification and options.
    fn handle_uci(&self) -> Option<String> {
        let mut response = String::new();
        response.push_str("id name ChessAI 0.1.0\n");
        response.push_str("id author Chess Engine Developers\n");
        response.push_str("option name Hash type spin default 64 min 1 max 1024\n");
        response.push_str("option name Threads type spin default 1 min 1 max 1\n");
        response.push_str("option name MultiPV type spin default 1 min 1 max 10\n");
        response.push_str("uciok");
        Some(response)
    }

    /// Handle "ucinewgame" command - reset state.
    fn handle_new_game(&mut self) -> Option<String> {
        self.board = Board::startpos();
        self.searcher = Searcher::with_tt_size(self.options.hash_size_mb);
        None
    }

    /// Handle "position" command - set up position.
    fn handle_position(&mut self, args: &[&str]) -> Option<String> {
        if args.is_empty() {
            return None;
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

                match parse_fen(&fen) {
                    Ok(board) => {
                        self.board = board;
                        // Apply moves if present
                        if let Some(idx) = moves_idx {
                            self.apply_moves(&args[idx + 1..]);
                        }
                    }
                    Err(_) => return None, // Invalid FEN, ignore
                }
            }
            _ => return None,
        }

        None
    }

    /// Apply a sequence of moves in UCI format.
    fn apply_moves(&mut self, moves: &[&str]) {
        for move_str in moves {
            if let Some(m) = self.parse_uci_move(move_str) {
                if self.board.is_legal(m) {
                    self.board.make_move(m);
                }
            }
        }
    }

    /// Parse a UCI move string (e.g., "e2e4", "e7e8q").
    fn parse_uci_move(&self, move_str: &str) -> Option<Move> {
        if move_str.len() < 4 {
            return None;
        }

        let from = Square::from_algebraic(&move_str[0..2])?;
        let to = Square::from_algebraic(&move_str[2..4])?;

        // Check if it's a promotion
        let promotion = if move_str.len() >= 5 {
            match move_str.chars().nth(4)? {
                'q' => Some(crate::piece::PieceType::Queen),
                'r' => Some(crate::piece::PieceType::Rook),
                'b' => Some(crate::piece::PieceType::Bishop),
                'n' => Some(crate::piece::PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };

        // Find matching legal move
        let legal_moves = self.board.generate_legal_moves();
        for m in legal_moves.iter() {
            if m.from() == from && m.to() == to {
                // Check promotion matches if applicable
                if let Some(promo_type) = promotion {
                    if m.is_promotion() && m.promotion_piece() == Some(promo_type) {
                        return Some(*m);
                    }
                } else {
                    // If no promotion specified, accept any move
                    return Some(*m);
                }
            }
        }

        None
    }

    /// Handle "go" command - start searching.
    fn handle_go(&mut self, args: &[&str]) -> Option<String> {
        let time_control = self.parse_time_control(args);

        // Determine max depth
        let max_depth = match &time_control {
            TimeControl::Depth { depth } => *depth,
            _ => 64, // Default max depth for time-based search
        };

        // Run search
        let result = self
            .searcher
            .search_with_limit(&self.board, max_depth, time_control);

        // Format bestmove response
        self.format_bestmove(&result)
    }

    /// Parse time control from "go" command arguments.
    fn parse_time_control(&self, args: &[&str]) -> TimeControl {
        let mut i = 0;
        let mut wtime = None;
        let mut btime = None;
        let mut winc = None;
        let mut binc = None;
        let mut movestogo = None;
        let mut movetime = None;
        let mut depth = None;
        let mut nodes = None;

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
                "nodes" => {
                    nodes = args.get(i + 1).and_then(|s| s.parse().ok());
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

        // Construct TimeControl based on parsed values
        if let Some(mt) = movetime {
            TimeControl::MoveTime { millis: mt }
        } else if let Some(d) = depth {
            TimeControl::Depth { depth: d }
        } else if let Some(n) = nodes {
            TimeControl::Nodes { nodes: n }
        } else if let Some(wt) = wtime {
            let bt = btime.unwrap_or(wt);
            TimeControl::Clock {
                wtime: wt,
                btime: bt,
                winc: winc.unwrap_or(0),
                binc: binc.unwrap_or(0),
                movestogo,
            }
        } else {
            TimeControl::Infinite
        }
    }

    /// Format bestmove response.
    fn format_bestmove(&self, result: &SearchResult) -> Option<String> {
        let bestmove = result.best_move.to_uci();

        // Check if we have a ponder move (second move in PV)
        if let Some(&ponder_move) = result.pv.get(1) {
            Some(format!(
                "bestmove {} ponder {}",
                bestmove,
                ponder_move.to_uci()
            ))
        } else {
            Some(format!("bestmove {}", bestmove))
        }
    }

    /// Handle "setoption" command.
    fn handle_setoption(&mut self, args: &[&str]) -> Option<String> {
        // Parse: setoption name <id> [value <x>]
        let mut i = 0;
        let mut name = String::new();
        let mut value = String::new();

        while i < args.len() {
            match args[i] {
                "name" => {
                    // Collect name until "value" or end
                    i += 1;
                    while i < args.len() && args[i] != "value" {
                        if !name.is_empty() {
                            name.push(' ');
                        }
                        name.push_str(args[i]);
                        i += 1;
                    }
                }
                "value" => {
                    // Collect value until end
                    i += 1;
                    while i < args.len() {
                        if !value.is_empty() {
                            value.push(' ');
                        }
                        value.push_str(args[i]);
                        i += 1;
                    }
                }
                _ => i += 1,
            }
        }

        // Apply option
        match name.to_lowercase().as_str() {
            "hash" => {
                if let Ok(size) = value.parse::<usize>() {
                    self.options.hash_size_mb = size.clamp(1, 1024);
                    self.searcher = Searcher::with_tt_size(self.options.hash_size_mb);
                }
            }
            "threads" => {
                if let Ok(threads) = value.parse::<usize>() {
                    self.options.threads = threads.clamp(1, 1);
                }
            }
            "multipv" => {
                if let Ok(multi_pv) = value.parse::<usize>() {
                    self.options.multi_pv = multi_pv.clamp(1, 10);
                }
            }
            _ => {} // Ignore unknown options
        }

        None
    }
}

impl Default for UciHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uci_command() {
        let mut handler = UciHandler::new();
        let response = handler.handle_command("uci");

        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.contains("id name"));
        assert!(resp.contains("id author"));
        assert!(resp.contains("uciok"));
        assert!(resp.contains("option name Hash"));
    }

    #[test]
    fn test_isready_command() {
        let mut handler = UciHandler::new();
        let response = handler.handle_command("isready");

        assert_eq!(response, Some("readyok".to_string()));
    }

    #[test]
    fn test_position_startpos() {
        let mut handler = UciHandler::new();
        handler.handle_command("position startpos");

        assert_eq!(handler.board, Board::startpos());
    }

    #[test]
    fn test_position_startpos_with_moves() {
        let mut handler = UciHandler::new();
        handler.handle_command("position startpos moves e2e4 e7e5");

        // Board should have had two moves applied
        assert_ne!(handler.board, Board::startpos());
    }

    #[test]
    fn test_position_fen() {
        let mut handler = UciHandler::new();
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        handler.handle_command(&format!("position fen {}", fen));

        // Verify FEN was parsed (board is not startpos)
        assert_ne!(handler.board, Board::startpos());
    }

    #[test]
    fn test_ucinewgame() {
        let mut handler = UciHandler::new();

        // Apply some moves
        handler.handle_command("position startpos moves e2e4 e7e5");
        assert_ne!(handler.board, Board::startpos());

        // Reset
        handler.handle_command("ucinewgame");
        handler.handle_command("position startpos");
        assert_eq!(handler.board, Board::startpos());
    }

    #[test]
    fn test_parse_uci_move() {
        let handler = UciHandler::new();

        // Valid moves
        assert!(handler.parse_uci_move("e2e4").is_some());
        assert!(handler.parse_uci_move("g1f3").is_some());

        // Invalid moves
        assert!(handler.parse_uci_move("e2e5").is_none()); // Illegal pawn move
        assert!(handler.parse_uci_move("xyz").is_none()); // Invalid format
    }

    #[test]
    fn test_parse_time_control_infinite() {
        let handler = UciHandler::new();
        let tc = handler.parse_time_control(&["infinite"]);
        assert!(matches!(tc, TimeControl::Infinite));
    }

    #[test]
    fn test_parse_time_control_movetime() {
        let handler = UciHandler::new();
        let tc = handler.parse_time_control(&["movetime", "5000"]);
        assert!(matches!(tc, TimeControl::MoveTime { millis: 5000 }));
    }

    #[test]
    fn test_parse_time_control_depth() {
        let handler = UciHandler::new();
        let tc = handler.parse_time_control(&["depth", "10"]);
        assert!(matches!(tc, TimeControl::Depth { depth: 10 }));
    }

    #[test]
    fn test_parse_time_control_nodes() {
        let handler = UciHandler::new();
        let tc = handler.parse_time_control(&["nodes", "100000"]);
        assert!(matches!(tc, TimeControl::Nodes { nodes: 100000 }));
    }

    #[test]
    fn test_parse_time_control_clock() {
        let handler = UciHandler::new();
        let tc = handler.parse_time_control(&[
            "wtime", "60000", "btime", "60000", "winc", "1000", "binc", "1000",
        ]);

        match tc {
            TimeControl::Clock {
                wtime,
                btime,
                winc,
                binc,
                movestogo,
            } => {
                assert_eq!(wtime, 60000);
                assert_eq!(btime, 60000);
                assert_eq!(winc, 1000);
                assert_eq!(binc, 1000);
                assert_eq!(movestogo, None);
            }
            _ => panic!("Expected Clock time control"),
        }
    }

    #[test]
    fn test_setoption_hash() {
        let mut handler = UciHandler::new();
        handler.handle_command("setoption name Hash value 128");
        assert_eq!(handler.options.hash_size_mb, 128);
    }

    #[test]
    fn test_setoption_multipv() {
        let mut handler = UciHandler::new();
        handler.handle_command("setoption name MultiPV value 3");
        assert_eq!(handler.options.multi_pv, 3);
    }

    #[test]
    fn test_go_command_returns_bestmove() {
        let mut handler = UciHandler::new();
        handler.handle_command("position startpos");
        let response = handler.handle_command("go depth 3");

        assert!(response.is_some());
        let resp = response.unwrap();
        assert!(resp.starts_with("bestmove"));
    }
}
