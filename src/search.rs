use crate::evaluation;
use crate::evaluation::EvalTable;
use cozy_chess::*;
use std::str::FromStr;
use std::time::{Duration, Instant};
const TT_SIZE: usize = 1 << 24;

pub struct AlphaBetaSearcher {
    tt: Vec<TTEntry>, // Transposition Table
    nodes: u64,
    eval_cache: EvalTable,
}
#[derive(Clone, Copy)]
struct TTEntry {
    // 16 bytes total
    hash: u64,       //4 bytes
    depth: i32,      //2 bytes
    score: i32,      //2 bytes
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
            tt: vec![
                TTEntry {
                    hash: 0,
                    depth: 0,
                    score: 0,
                    best_move: Move::from_str("a1a1").unwrap(),
                    node_type: NodeType::Exact,
                };
                TT_SIZE
            ],
            nodes: 0,
            eval_cache: Default::default(),
        }
    }
    pub fn get_best_move(&mut self, board: &Board, movetime: u64, depth: i32) -> String {
        let start_time: Instant = Instant::now();
        let limit: Duration = Duration::from_millis(movetime / 30);
        let mut current_depth: i32 = 1;
        self.nodes = 0;
        let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while start_time.elapsed() < limit && current_depth <= depth {
            let score: i32 = self.alpha_beta(board, alpha, beta, current_depth, 0);
            println!(
                "info depth {} time {} nodes {} score cp {} pv {}",
                current_depth,
                start_time.elapsed().as_millis(),
                self.nodes,
                score,
                self.getpv(board),
            );
            current_depth += 1;
        }
        let final_move = self.getbm(board);
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
        let mut best_move: Move;
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
            let score = -self.alpha_beta(&new_board, -beta, -alpha, depthleft - 1, ply + 1);
            if score > best_value {
                best_move = *mov;
                best_value = score;
                if best_value > alpha {
                    alpha = best_value; // alpha acts like max in MiniMax
                    if alpha >= beta {
                        let tt_entry: TTEntry = TTEntry {
                            hash: board.hash(),
                            depth: depthleft,
                            score: best_value,
                            best_move: best_move,
                            node_type: NodeType::LowerBound,
                        };
                        self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
                        return best_value;
                    } else {
                        let tt_entry: TTEntry = TTEntry {
                            hash: board.hash(),
                            depth: depthleft,
                            score: best_value,
                            best_move: best_move,
                            node_type: NodeType::Exact,
                        };
                        self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
                    }
                } else {
                    let tt_entry: TTEntry = TTEntry {
                        hash: board.hash(),
                        depth: depthleft,
                        score: best_value,
                        best_move: best_move,
                        node_type: NodeType::UpperBound,
                    };
                    self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
                }
            }
        }
        return best_value;
    }
    fn getpv(&self, iboard: &Board) -> String {
        let mut board = iboard.clone();
        let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        let mut bm = entry.best_move;
        let mut pv = bm.to_string();
        loop {
            board.play_unchecked(bm);
            let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
            if entry.hash == board.hash() {
                bm = entry.best_move;
                pv = format!("{} {}", pv, bm.to_string());
            } else {
                break;
            }
        }
        return pv;
    }
    fn getbm(&self, iboard: &Board) -> String {
        let board = iboard.clone();
        let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        let bm = entry.best_move;
        return bm.to_string();
    }
}
