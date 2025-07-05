use crate::evaluation::{self, EvalTable};
use cozy_chess::*;
use std::str::FromStr;

pub struct AlphaBetaSearcher {
    root_best_move: Move,
    root_score: i32,
    nodes: u64,
    evalt:EvalTable 
}
impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            root_best_move: Move::from_str("a1a4").unwrap(),
            root_score: 0,
            nodes: 0,
            evalt: EvalTable::default(),
        }
    }
    pub fn get_best_move(&mut self, board: &Board, depth: i32) -> String {
        let mut current_depth: i32 = 1;
        let final_move: String;
        self.nodes = 0;
        self.root_best_move = Move::from_str("a1a4").unwrap();

        let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while current_depth <= depth {
            let score: i32 = self.alpha_beta(board, alpha,  beta, current_depth);
            println!("best move {} at depth {} score cp {}", self.root_best_move, current_depth, score);
            current_depth += 1;
        }
        final_move = self.root_best_move.clone().to_string();

        println!(
            "info depth {} score cp {}",
            current_depth - 1,
            self.root_score
        );
        return final_move;
    }
    fn alpha_beta(&mut self, board: &Board, mut alpha: i32, beta: i32, depthleft: i32) -> i32 {
        if depthleft == 0 {
            //return evaluation::eval(board, &mut self.evalt);
            return evaluation::eval_from_scratch(board);
        }
        let mut best_value = -99999999;
        //generate all moves and store them in a vector
        let mut moves = Vec::new();
        board.generate_moves(|p: PieceMoves| {
            for m in p {
                moves.push(m);
            }
            false
        });
        for mov in moves.iter() {
            let mut new_board = board.clone();
            new_board.play_unchecked(*mov);
            let score = -self.alpha_beta(&new_board, -beta, -alpha, depthleft - 1);
            if score > best_value {
                self.root_best_move = *mov; // update the best move
                self.root_score = score; // update the root score
                best_value = score;
                if score > alpha {
                    alpha = score; // alpha acts like max in MiniMax
                }
            }
            if score >= beta {
                return best_value;
            } //  fail soft beta-cutoff, existing the loop here is also fine
        }
        return best_value;
    }
}
