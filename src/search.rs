use std::time::Instant;
use cozy_chess::*;
use crate::evaluation;

pub struct AlphaBetaSearcher {
    root_score: i32,
    nodes: u64,
    best_move:String,
    curr_pv :String,
}
impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            root_score: 0,
            nodes: 0,
            best_move: "".to_string(),
            curr_pv: "".to_string(),
        }
    }
    pub fn get_best_move(&mut self, board: &Board, depth: i32) -> String {
        let start_time: Instant = Instant::now();
        let mut current_depth: i32 = 1;
        self.nodes = 0;
        let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while current_depth <= depth {
            self.best_move = "".to_string();
            self.root_score = -99999999;
            let score: i32 = self.alpha_beta(board, alpha,  beta, current_depth,0);
            println!("info depth {} time {} nodes 0 score cp {} pv {}", current_depth, start_time.elapsed().as_millis(), score, self.best_move);
            current_depth += 1;
        }
        let final_move = self.best_move.clone();
        return final_move;
    }
    fn alpha_beta(&mut self, board: &Board, mut alpha: i32, beta: i32, depthleft: i32,ply: u32) -> i32 {
        if depthleft == 0 {
            let eval = evaluation::eval_from_scratch(board);
            return eval;
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
            let old_pv = self.curr_pv.clone();
            new_board.play_unchecked(*mov);
            self.curr_pv = (format!("{} {}", self.curr_pv, mov.to_string())).trim().to_string();
            let score = -self.alpha_beta(&new_board, -beta, -alpha, depthleft - 1,ply+1);
            if score > best_value {
                if ply == 0 {
                    self.root_score = score; 
                    self.best_move = self.curr_pv.clone(); 
                }
                best_value = score;
                if score > alpha {
                    alpha = score; // alpha acts like max in MiniMax
                }
            }
            if score >= beta {
                return best_value;
            } //  fail soft beta-cutoff, existing the loop here is also fine
            self.curr_pv = old_pv; // restore the previous principal variation
        }
        return best_value;
    }
}
