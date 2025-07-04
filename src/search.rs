use cozy_chess::*;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct AlphaBetaSearcher {
    root_best_move: Move,
    root_score: i32,
    min_val: i32,
    nodes: u64,
}
impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            root_best_move: Move::from_str("a1a4").unwrap(),
            root_score: 0,
            min_val: - (1 << 30),
            nodes: 0,
        }
    }
    pub fn get_best_move(&mut self, board: &Board, depth: u32) -> String {
        let start_time: Instant = Instant::now();
        //let hard_limit: Duration = Duration::from_millis(time_remaining/10);
        //let soft_limit: Duration = Duration::from_millis(time_remaining/40);
        //do iterative deepening until we run out of time
        let mut current_depth: u32 = 1;
        let final_move: String;
        self.nodes = 0;
        self.root_best_move = Move::from_str("a1a4").unwrap();
        //clear history table
        // self.history_table = vec![vec![vec![0; 64]; 64]; 2];
        //self.history_table = [[[0; 64]; 64]; 2];
        // self.killer_table = vec![Move::from_str("a1a4").unwrap(); 128];

        let mut aspiration_window: i32 = 15;
        let mut alpha: i32 = -99999999;
        let mut beta: i32 = 99999999;

        while current_depth <= depth {
            let score: i32 = 10;//self.pvs(board, current_depth, alpha, beta, 0, start_time, hard_limit, true);
            if score <= alpha || score >= beta {
                //fail high or low, re-search with gradual widening
                aspiration_window *= 2;
                alpha = score - aspiration_window;
                beta = score + aspiration_window;
                continue;
            }
            aspiration_window = 15;
            alpha = score - aspiration_window;
            beta = score + aspiration_window;
            println!("depth {} score cp {} NPS {}k", current_depth, score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
            current_depth += 1;
        }
        final_move = self.root_best_move.clone().to_string();
        //check if final_move is legal
        let mut legal_moves: Vec<String> = Vec::new();
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                legal_moves.push(m.to_string());
            }
            false
        });
        //if move is not legal, print move and fen to stderr and panic
        if !legal_moves.contains(&final_move) {
            panic!("Illegal move {} in position {}. Searched to depth {} with root_best_move {}", final_move, board, current_depth - 1, self.root_best_move);
        }
        println!("info depth {} score cp {} NPS {}k", current_depth - 1, self.root_score, (self.nodes as f32) / (start_time.elapsed().as_secs_f32() *1000.0));
        return final_move;
    }





}