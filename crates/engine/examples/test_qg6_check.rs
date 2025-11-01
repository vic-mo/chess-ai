/// Test if Qg6 gives check in WAC.001
use engine::board::Board;
use engine::io::parse_fen;
use engine::movegen::generate_moves;

fn main() {
    let fen = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
    println!("FEN: {}", fen);

    let board = parse_fen(fen).unwrap();
    let moves = generate_moves(&board);

    // Find Qg6
    for i in 0..moves.len() {
        let mv = moves[i];
        if mv.from().to_algebraic() == "g3" && mv.to().to_algebraic() == "g6" {
            println!("Found Qg6: {}", mv.to_uci());
            println!("Gives check: {}", board.gives_check(mv));
            break;
        }
    }
}
