use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engine::board::Board;
use engine::io::parse_fen;
use engine::perft::perft;

fn perft_startpos_depth3(c: &mut Criterion) {
    let board = Board::startpos();
    c.bench_function("perft startpos depth 3", |b| {
        b.iter(|| perft(black_box(&board), 3))
    });
}

fn perft_startpos_depth4(c: &mut Criterion) {
    let board = Board::startpos();
    c.bench_function("perft startpos depth 4", |b| {
        b.iter(|| perft(black_box(&board), 4))
    });
}

fn perft_startpos_depth5(c: &mut Criterion) {
    let board = Board::startpos();
    c.bench_function("perft startpos depth 5", |b| {
        b.iter(|| perft(black_box(&board), 5))
    });
}

fn perft_kiwipete_depth3(c: &mut Criterion) {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    c.bench_function("perft kiwipete depth 3", |b| {
        b.iter(|| perft(black_box(&board), 3))
    });
}

fn perft_kiwipete_depth4(c: &mut Criterion) {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let board = parse_fen(fen).unwrap();
    c.bench_function("perft kiwipete depth 4", |b| {
        b.iter(|| perft(black_box(&board), 4))
    });
}

fn perft_position3_depth4(c: &mut Criterion) {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let board = parse_fen(fen).unwrap();
    c.bench_function("perft position3 depth 4", |b| {
        b.iter(|| perft(black_box(&board), 4))
    });
}

criterion_group!(
    benches,
    perft_startpos_depth3,
    perft_startpos_depth4,
    perft_startpos_depth5,
    perft_kiwipete_depth3,
    perft_kiwipete_depth4,
    perft_position3_depth4,
);
criterion_main!(benches);
