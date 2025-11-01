use engine::attacks::knight_attacks;
use engine::square::Square;

fn main() {
    println!("=== TESTING KNIGHT ATTACKS FROM F6 ===\n");

    let f6 = Square::from_coords(5, 5); // f6 = file 5 (f), rank 5 (6th rank, 0-indexed as 5)
    println!("Testing knight on square: {}", f6);

    let attacks = knight_attacks(f6);

    println!("Knight attack bitboard has {} squares", attacks.count());
    println!("\nSquares attacked by knight on f6:");

    for sq in attacks {
        println!("  - {}", sq);
    }

    // Check if d4 is in the attacks
    let d4 = Square::from_coords(3, 3); // d4 = file 3 (d), rank 3 (4th rank, 0-indexed as 3)

    println!("\nChecking if d4 ({}) is in attacks:", d4);
    if attacks.contains(d4) {
        println!("✅ YES - d4 IS in knight attacks from f6");
    } else {
        println!("❌ NO - d4 is NOT in knight attacks from f6 (BUG!)");
    }

    // Verify the attack generation is correct
    println!("\nExpected attacks from f6:");
    let expected = vec!["e4", "g4", "d5", "h5", "d7", "h7", "e8", "g8"];
    println!("  {:?}", expected);

    // Check each expected square
    println!("\nVerifying each expected square:");
    for sq_str in &expected {
        let file = (sq_str.chars().nth(0).unwrap() as u8) - b'a';
        let rank = (sq_str.chars().nth(1).unwrap() as u8) - b'1';
        let sq = Square::from_coords(file, rank);

        if attacks.contains(sq) {
            println!("  ✅ {} is in attacks", sq_str);
        } else {
            println!("  ❌ {} is NOT in attacks (expected!)", sq_str);
        }
    }

    // Specifically check d4 with direct computation
    println!("\n=== MANUAL VERIFICATION ===");
    println!("F6 is at file=5, rank=5");
    println!("D4 is at file=3, rank=3");
    println!("Knight moves: (+/-2, +/-1) or (+/-1, +/-2)");
    println!("From f6 (5,5) to d4 (3,3): delta = (-2, -2)");
    println!("Is (-2, -2) a valid knight move? NO");
    println!("Wait... that means d4 is NOT a legal knight move from f6!");
    println!("\nLet me recalculate:");
    println!("Valid knight deltas:");
    let deltas = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
    for (df, dr) in deltas {
        let new_file = 5i8 + df;
        let new_rank = 5i8 + dr;
        if new_file >= 0 && new_file < 8 && new_rank >= 0 && new_rank < 8 {
            let sq = Square::from_coords(new_file as u8, new_rank as u8);
            println!("  f6 + ({:+3}, {:+3}) = {}", df, dr, sq);
        }
    }
}
