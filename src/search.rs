use crate::evaluation;
use crate::evaluation::EvalTable;
use cozy_chess::*;
use std::time::{Instant, Duration};
use std::str::FromStr;
const TT_SIZE: usize = 1 << 24;

pub struct AlphaBetaSearcher {
    tt: Vec<TTEntry>, // Transposition Table
    root_score: i32,
    nodes: u64,
    best_move: String,
    curr_pv: String,
    eval_cache: EvalTable,
}
#[derive(Clone, Copy)]
struct TTEntry { // 16 bytes total
    hash: u64, //4 bytes
    depth: i32, //2 bytes
    score: i32, //2 bytes
    best_move: Move, // 8 bytes
    node_type: NodeType,
}
#[derive(Clone, Copy)]
enum NodeType {
    Exact,
    LowerBound,
    UpperBound,
}

impl AlphaBetaSearcher {
    pub fn new() -> Self {
        AlphaBetaSearcher {
            tt: vec![TTEntry {
                hash: 0,
                depth: 0,
                score: 0,
                best_move: Move::from_str("a1a1").unwrap(),
                node_type: NodeType::Exact,
            }; TT_SIZE],
            root_score: 0,
            nodes: 0,
            best_move: "".to_string(),
            curr_pv: "".to_string(),
            eval_cache: Default::default(),
        }
    }
    pub fn get_best_move_depth(&mut self, board: &Board, depth: i32) -> String {
        let start_time: Instant = Instant::now();
        let mut current_depth: i32 = 1;
        self.nodes = 0;
        self.best_move = "".to_string();
            let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while current_depth <= depth {
            let score: i32 = self.alpha_beta(board, alpha, beta, current_depth, 0);
            println!(
                "info depth {} time {} nodes {} score cp {} pv {}",
                current_depth,
                start_time.elapsed().as_millis(),
                self.nodes,
                score,
                self.best_move
            );
            current_depth += 1;
        }
        //TODO
        //let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        //println!("ttbestmove: {} final move {}" , entry.best_move.to_string(), self.best_move);
        
        let final_move = self.best_move.clone();
        return final_move;
    }
    pub fn get_best_move_time(&mut self, board: &Board, movetime: u64) -> String {
        let start_time: Instant = Instant::now();
        let limit: Duration = Duration::from_millis(movetime/30);
        let mut current_depth: i32 = 1;
        self.nodes = 0;
        self.best_move = "".to_string();
        let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while start_time.elapsed() < limit {
            let score: i32 = self.alpha_beta(board, alpha, beta, current_depth, 0);
            println!(
                "info depth {} time {} nodes {} score cp {} pv {}",
                current_depth,
                start_time.elapsed().as_millis(),
                self.nodes,
                score,
                self.best_move
            );
            current_depth += 1;
        }
        //TODO
        //let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        //println!("ttbestmove: {} final move {}" , entry.best_move.to_string(), self.best_move);
        
        let final_move = self.best_move.clone();
        return final_move;
    }
    fn alpha_beta(
        &mut self,
        board: &Board,
        mut alpha: i32,
        beta: i32,
        depthleft: i32,
        ply: u32,
    ) -> i32 {
        self.nodes += 1;
        match board.status() {
            GameStatus::Won => return -320000 + (ply as i32) * 10, 
            GameStatus::Drawn => return 0,
            GameStatus::Ongoing => {}
        }
        if depthleft == 0 {
            let eval = evaluation::eval(board, &mut self.eval_cache);
            return eval;
        }

        let root: bool = ply == 0;
        let pv_node: bool = beta - alpha > 1;
        // probe TT
        let mut new_alpha: i32 = alpha;
        let mut new_beta: i32 = beta;
        let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        if entry.hash == board.hash() && entry.depth >= depthleft && !root && !pv_node {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => new_alpha = alpha.max(entry.score),
                NodeType::UpperBound => new_beta = beta.min(entry.score),
            }
            if new_alpha >= new_beta {
                return entry.score;
            }
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
            self.curr_pv = (format!("{} {}", self.curr_pv, mov.to_string()))
                .trim()
                .to_string();
            let score = -self.alpha_beta(&new_board, -beta, -alpha, depthleft - 1, ply + 1);
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
        let node_type: NodeType = if best_value <= alpha {
            NodeType::UpperBound
        } else if best_value >= beta {
            NodeType::LowerBound
        } else {
            NodeType::Exact
        };
        let tt_entry: TTEntry = TTEntry {
            hash: board.hash(),
            depth: depthleft,
            score: best_value,
            best_move: Move::from_str(self.best_move.as_str()).unwrap(),
            node_type: node_type,
        };
        self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
        return best_value;
    }
}
