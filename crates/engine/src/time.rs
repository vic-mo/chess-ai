use std::time::{Duration, Instant};

/// Time control mode for a search
#[derive(Debug, Clone, PartialEq, Default)]
pub enum TimeControl {
    /// No time limit - search until stopped
    #[default]
    Infinite,

    /// Fixed time per move (in milliseconds)
    MoveTime { millis: u64 },

    /// Clock-based time control
    Clock {
        /// White's remaining time (ms)
        wtime: u64,
        /// Black's remaining time (ms)
        btime: u64,
        /// White's increment per move (ms)
        winc: u64,
        /// Black's increment per move (ms)
        binc: u64,
        /// Number of moves until next time control (None = sudden death)
        movestogo: Option<u32>,
    },

    /// Search to a fixed depth
    Depth { depth: u32 },

    /// Search a fixed number of nodes
    Nodes { nodes: u64 },
}

/// Manages time allocation during search
pub struct TimeManager {
    /// Soft time limit - should stop searching after this
    soft_limit: Option<Instant>,
    /// Hard time limit - must stop searching after this
    hard_limit: Option<Instant>,
    /// When search started
    start_time: Instant,
    /// Time control mode
    time_control: TimeControl,
}

impl TimeManager {
    /// Create a new time manager with given time control
    pub fn new(time_control: TimeControl, is_white: bool) -> Self {
        let start_time = Instant::now();
        let (soft_limit, hard_limit) = Self::calculate_limits(&time_control, is_white, start_time);

        TimeManager {
            soft_limit,
            hard_limit,
            start_time,
            time_control,
        }
    }

    /// Calculate soft and hard time limits based on time control
    fn calculate_limits(
        time_control: &TimeControl,
        is_white: bool,
        start_time: Instant,
    ) -> (Option<Instant>, Option<Instant>) {
        match time_control {
            TimeControl::Infinite | TimeControl::Depth { .. } | TimeControl::Nodes { .. } => {
                (None, None)
            }

            TimeControl::MoveTime { millis } => {
                let hard = start_time + Duration::from_millis(*millis);
                let soft = start_time + Duration::from_millis(millis.saturating_sub(50)); // Leave 50ms buffer
                (Some(soft), Some(hard))
            }

            TimeControl::Clock {
                wtime,
                btime,
                winc,
                binc,
                movestogo,
            } => {
                let (my_time, my_inc) = if is_white {
                    (*wtime, *winc)
                } else {
                    (*btime, *binc)
                };

                // Time allocation strategy
                let allocated = Self::allocate_time(my_time, my_inc, *movestogo);

                let hard = start_time + Duration::from_millis(allocated.hard);
                let soft = start_time + Duration::from_millis(allocated.soft);

                (Some(soft), Some(hard))
            }
        }
    }

    /// Allocate time for this move
    fn allocate_time(
        time_remaining: u64,
        increment: u64,
        movestogo: Option<u32>,
    ) -> TimeAllocation {
        // Safety margin - reserve 100ms or 2% of remaining time
        let safety_margin = time_remaining.min(100).max(time_remaining / 50);
        let available = time_remaining.saturating_sub(safety_margin);

        if available < 100 {
            // Panic mode - very little time left
            return TimeAllocation {
                soft: available / 2,
                hard: available,
            };
        }

        // Calculate base allocation
        let base = match movestogo {
            Some(mtg) if mtg > 0 => {
                // Classical time control - divide by moves to go
                available / (mtg as u64)
            }
            _ => {
                // Sudden death or unknown - assume ~40 moves remaining
                available / 40
            }
        };

        // Add increment (but not all of it - save some for later)
        let base_with_inc = base + (increment * 3) / 4;

        // Soft limit: our target time (can extend if position is unclear)
        let soft = base_with_inc;

        // Hard limit: absolute maximum (typically 3-5x soft limit, but capped)
        let max_multiple = if available > 10000 { 5 } else { 3 };
        let hard = (soft * max_multiple).min(available);

        TimeAllocation { soft, hard }
    }

    /// Check if we should stop searching (soft limit exceeded)
    pub fn should_stop(&self) -> bool {
        if let Some(soft) = self.soft_limit {
            Instant::now() >= soft
        } else {
            false
        }
    }

    /// Check if we must stop searching (hard limit exceeded)
    pub fn must_stop(&self) -> bool {
        if let Some(hard) = self.hard_limit {
            Instant::now() >= hard
        } else {
            false
        }
    }

    /// Get elapsed time since search started (in milliseconds)
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Get time control
    pub fn time_control(&self) -> &TimeControl {
        &self.time_control
    }

    /// Check if depth limit has been reached
    pub fn depth_limit_reached(&self, current_depth: u32) -> bool {
        matches!(self.time_control, TimeControl::Depth { depth } if current_depth >= depth)
    }

    /// Check if node limit has been reached
    pub fn node_limit_reached(&self, current_nodes: u64) -> bool {
        matches!(self.time_control, TimeControl::Nodes { nodes } if current_nodes >= nodes)
    }
}

/// Time allocation result
#[derive(Debug, Clone, Copy)]
struct TimeAllocation {
    /// Soft limit - target time to use
    soft: u64,
    /// Hard limit - maximum time allowed
    hard: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_time_control() {
        let tm = TimeManager::new(TimeControl::Infinite, true);
        assert!(!tm.should_stop());
        assert!(!tm.must_stop());
        assert!(!tm.depth_limit_reached(100));
        assert!(!tm.node_limit_reached(1_000_000));
    }

    #[test]
    fn test_move_time() {
        let tm = TimeManager::new(TimeControl::MoveTime { millis: 1000 }, true);

        // Immediately after creation, should not stop
        assert!(!tm.should_stop());
        assert!(!tm.must_stop());

        // Should have limits set
        assert!(tm.soft_limit.is_some());
        assert!(tm.hard_limit.is_some());
    }

    #[test]
    fn test_clock_time_control() {
        let tm = TimeManager::new(
            TimeControl::Clock {
                wtime: 60000, // 1 minute
                btime: 60000,
                winc: 1000, // 1 second increment
                binc: 1000,
                movestogo: Some(20),
            },
            true,
        );

        assert!(tm.soft_limit.is_some());
        assert!(tm.hard_limit.is_some());
    }

    #[test]
    fn test_depth_limit() {
        let tm = TimeManager::new(TimeControl::Depth { depth: 10 }, true);

        assert!(!tm.depth_limit_reached(5));
        assert!(!tm.depth_limit_reached(9));
        assert!(tm.depth_limit_reached(10));
        assert!(tm.depth_limit_reached(11));

        // No time limits for depth control
        assert!(!tm.should_stop());
        assert!(!tm.must_stop());
    }

    #[test]
    fn test_node_limit() {
        let tm = TimeManager::new(TimeControl::Nodes { nodes: 10000 }, true);

        assert!(!tm.node_limit_reached(5000));
        assert!(!tm.node_limit_reached(9999));
        assert!(tm.node_limit_reached(10000));
        assert!(tm.node_limit_reached(15000));

        // No time limits for node control
        assert!(!tm.should_stop());
        assert!(!tm.must_stop());
    }

    #[test]
    fn test_elapsed_time() {
        let tm = TimeManager::new(TimeControl::Infinite, true);
        std::thread::sleep(Duration::from_millis(10));
        assert!(tm.elapsed_ms() >= 10);
    }

    #[test]
    fn test_time_allocation_classical() {
        let alloc = TimeManager::allocate_time(60000, 0, Some(40));

        // Should allocate roughly 1/40th of time (1500ms)
        // With safety margin removed
        assert!(alloc.soft >= 1000 && alloc.soft <= 2000);

        // Hard limit should be higher
        assert!(alloc.hard > alloc.soft);
    }

    #[test]
    fn test_time_allocation_with_increment() {
        let alloc = TimeManager::allocate_time(60000, 1000, None);

        // Should include 75% of increment
        assert!(alloc.soft >= 1000); // At least some base time + increment

        // Hard limit should allow for longer think
        assert!(alloc.hard > alloc.soft);
    }

    #[test]
    fn test_time_allocation_panic_mode() {
        let alloc = TimeManager::allocate_time(50, 0, None);

        // In panic mode with < 100ms
        assert!(alloc.soft <= 25); // Half of available
        assert!(alloc.hard <= 50); // All available
    }

    #[test]
    fn test_time_control_equality() {
        assert_eq!(TimeControl::Infinite, TimeControl::Infinite);
        assert_eq!(
            TimeControl::MoveTime { millis: 1000 },
            TimeControl::MoveTime { millis: 1000 }
        );
        assert_ne!(
            TimeControl::MoveTime { millis: 1000 },
            TimeControl::MoveTime { millis: 2000 }
        );
    }

    #[test]
    fn test_default_time_control() {
        let tc: TimeControl = Default::default();
        assert_eq!(tc, TimeControl::Infinite);
    }
}
