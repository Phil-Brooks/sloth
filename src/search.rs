use crate::evaluation;
use crate::evaluation::EvalTable;
use cozy_chess::*;
use std::str::FromStr;
use std::time::{Duration, Instant};
const TT_SIZE: usize = 1 << 25;

pub struct AlphaBetaSearcher {
    tt: Vec<TTEntry>, // Transposition Table
    nodes: u64,
    best_move: String,
    prev_best_move: String,
    eval_cache: EvalTable,
    start: Instant,
    limit: Duration,
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
            best_move: String::new(),
            prev_best_move: String::new(),
            eval_cache: Default::default(),
            start: Instant::now(),
            limit: Duration::from_millis(0),
        }
    }
    pub fn get_best_move(&mut self, board: &Board, movetime: u64, depth: i32) -> String {
        self.start = Instant::now();
        let fullmovetime: Duration = Duration::from_millis(movetime);
        let tolerance: Duration = Duration::from_millis(200);
        self.limit = if fullmovetime > 2 * tolerance {
            fullmovetime - tolerance
        } else {
            fullmovetime / 2
        };
        let mut current_depth: i32 = 1;
        self.nodes = 0;
        let alpha: i32 = -99999999;
        let beta: i32 = 99999999;

        while self.start.elapsed() < self.limit && current_depth <= depth {
            self.prev_best_move = self.best_move.clone();
            let score: i32 = self.alpha_beta(board, alpha, beta, current_depth, 0);
            if self.start.elapsed() > self.limit{
                println!("exceeded time limit, returning best move"); 
                return self.prev_best_move.clone();
            }
            println!(
                "info depth {} time {} nodes {} score cp {} pv {}",
                current_depth,
                self.start.elapsed().as_millis(),
                self.nodes,
                score,
                self.getpv(board),
            );
            current_depth += 1;
        }
        let final_move = self.best_move.clone();
        return final_move;
    }
    fn alpha_beta(
        &mut self,
        board: &Board,
        mut alpha: i32,
        mut beta: i32,
        depthleft: i32,
        ply: i32,
    ) -> i32 {
        if self.start.elapsed() > self.limit && ply > 0 {return 0;}
        self.nodes += 1;
        match board.status() {
            GameStatus::Won => return -320000 + ply * 10,
            GameStatus::Drawn => return 0,
            GameStatus::Ongoing => {}
        }
        if depthleft == 0 {
            let eval = self.qs(board, alpha, beta, ply);
            return eval;
        }

        let root: bool = ply == 0;
        let pv_node: bool = beta - alpha > 1;
        // probe TT
        let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        if entry.hash == board.hash() && entry.depth >= depthleft && !root && !pv_node{
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => alpha = alpha.max(entry.score),
                NodeType::UpperBound => beta = beta.min(entry.score),
            }
            if alpha >= beta {
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
                if ply == 0 {
                    self.best_move = mov.to_string();
                }
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
    fn qs(&mut self, board: &Board, mut alpha: i32, beta: i32, ply: i32) -> i32 {
        if self.start.elapsed() > self.limit {return 0;}
        
        self.nodes += 1;
        match board.status() {
            GameStatus::Won => return -320000 + ply * 10,
            GameStatus::Drawn => return 0,
            GameStatus::Ongoing => {}
        }
        let stand_pat = evaluation::eval(board, &mut self.eval_cache);
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        // probe TT
        let entry: TTEntry = self.tt[board.hash() as usize % TT_SIZE];
        if entry.hash == board.hash() {
            match entry.node_type {
                NodeType::Exact => return entry.score,
                NodeType::LowerBound => {
                    if entry.score >= beta {
                        return entry.score;
                    }
                }
                NodeType::UpperBound => {
                    if entry.score <= alpha {
                        return entry.score;
                    }
                }
            }
        }

        //generate all caotures and store them in a vector
        let mut moves = Vec::new();
        let their_pieces = board.colors(!board.side_to_move());
        board.generate_moves(|p| {
            let mut capture_moves = p;
            capture_moves.to &= their_pieces;
            for mv in capture_moves {
                moves.push(mv);
            }
            false
        });
        if moves.is_empty() {
            return stand_pat;
        }
        let mut best_value = stand_pat;
        let mut best_move: Move;
        for mov in moves.iter() {
            let mut new_board = board.clone();
            new_board.play_unchecked(*mov);
            let score = -self.qs(&new_board, -beta, -alpha, ply + 1);
            if score > best_value {
                best_move = *mov;
                best_value = score;
                if best_value > alpha {
                    alpha = best_value; // alpha acts like max in MiniMax
                    if alpha >= beta {
                        let tt_entry: TTEntry = TTEntry {
                            hash: board.hash(),
                            depth: ply,
                            score: best_value,
                            best_move: best_move,
                            node_type: NodeType::LowerBound,
                        };
                        self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
                        return best_value;
                    } else {
                        let tt_entry: TTEntry = TTEntry {
                            hash: board.hash(),
                            depth: ply,
                            score: best_value,
                            best_move: best_move,
                            node_type: NodeType::Exact,
                        };
                        self.tt[board.hash() as usize % TT_SIZE] = tt_entry;
                    }
                } else {
                    let tt_entry: TTEntry = TTEntry {
                        hash: board.hash(),
                        depth: ply,
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
        let mut ct = 0;
        loop {
            ct += 1;
            if ct > 10 {
                break; // prevent infinite loop
            }
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
}
