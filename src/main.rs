//! Sloth - Rust chess engine
//!
//! Lazy as it uses existing packages where it can. This includes:
//! - cozy chess
mod evaluation;
mod search;
mod tests;
use cozy_chess::*;
use search::AlphaBetaSearcher;
/// Main entry point - currently handles UCI.
fn main() {
    let mut board: Board = Board::default();
    let mut input: String = String::new();
    let mut searcher: AlphaBetaSearcher = AlphaBetaSearcher::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.starts_with("ucinewgame") {
            board = Board::default();
        } else if input.starts_with("uci") {
            println!("id name sloth");
            println!("id author Phil Brooks");
            println!("uciok");
        } else if input.starts_with("isready") {
            println!("readyok");
        } else if input.starts_with("position startpos moves") {
            board = Board::default();
            let moves = input.split_whitespace().skip(3);
            for m in moves {
                match util::parse_uci_move(&board, m) {
                    Ok(ucimove) => {
                        board.play(ucimove);
                        //searcher.add_to_threefold_repetition(board.hash());
                    }
                    Err(e) => {
                        eprintln!("Failed to parse move: {}. Error: {:?}", m, e);
                        break;
                    }
                }
            }
        } else if input.starts_with("position startpos") {
            board = Board::default();
        } else if input.starts_with("position fen") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            let fen_end = parts
                .iter()
                .position(|&x| x == "moves")
                .unwrap_or(parts.len());
            let fen = parts[2..fen_end].join(" ");

            match Board::from_fen(&fen, false) {
                Ok(new_board) => board = new_board,
                Err(e) => {
                    eprintln!("Failed to parse FEN: {}. Error: {:?}", fen, e);
                    continue;
                }
            }

            if let Some(moves_index) = parts.iter().position(|&x| x == "moves") {
                for m in parts.iter().skip(moves_index + 1) {
                    match util::parse_uci_move(&board, m) {
                        Ok(ucimove) => {
                            board.play(ucimove);
                            //searcher.add_to_threefold_repetition(board.hash());
                        }
                        Err(e) => {
                            eprintln!("Failed to parse move: {}. Error: {:?}", m, e);
                            break;
                        }
                    }
                }
            }
        } else if input.starts_with("go depth") {
            let words: Vec<&str> = input.split_whitespace().collect();
            if words.len() < 3 {
                eprintln!("Missing depth value");
                continue;
            }
            if let Ok(depth) = words[2].parse::<i32>() {
                let best_move: String = searcher.get_best_move(&board, 9999999999, depth);
                println!("bestmove {}", best_move);
                searcher = AlphaBetaSearcher::new();
            } else {
                eprintln!("Invalid depth value: {}", words[2]);
            }
        } else if input.starts_with("go movetime") {
            let words: Vec<&str> = input.split_whitespace().collect();
            if words.len() < 3 {
                eprintln!("Missing movetime value");
                continue;
            }
            if let Ok(movetime) = words[2].parse::<u64>() {
                let best_move: String = searcher.get_best_move(&board, movetime, 999);
                println!("bestmove {}", best_move);
                searcher = AlphaBetaSearcher::new();
            } else {
                eprintln!("Invalid depth value: {}", words[2]);
            }
        } else if input.starts_with("go wtime") {
            let words: Vec<&str> = input.split_whitespace().collect();
            if words.len() < 5 {
                eprintln!("Missing time values");
                continue;
            }
            let wtime = match words[2].parse::<u64>() {
                Ok(wtime) => wtime,
                Err(_) => {
                    eprintln!("Invalid wtime value: {}", words[2]);
                    continue;
                }
            };
            let btime = match words[4].parse::<u64>() {
                Ok(btime) => btime,
                Err(_) => {
                    eprintln!("Invalid btime value: {}", words[4]);
                    continue;
                }
            };
            let movetime = if board.side_to_move() == Color::White {
                wtime
            } else {
                btime
            };
            let mut divisor = 50;
            if input.contains("movestogo") {
                let words: Vec<&str> = input.split_whitespace().collect();
                if let Ok(movestogo) = words[words.len()-1].parse::<u64>() {
                    if movestogo > 0 {
                        divisor = movestogo;
                    } else {
                        println!("Invalid movestogo value: {}", words[words.len()-1]);
                    }
                } else {
                    println!("Invalid movestogo value: {}", words[words.len()-1]);
                }
            }
            let best_move: String = searcher.get_best_move(&board, movetime/divisor, 999);
            println!("bestmove {}", best_move);
            searcher = AlphaBetaSearcher::new();
        } else if input.starts_with("eval") {
            println!("eval: {}cp", evaluation::eval_from_scratch(&board));
        } else if input.starts_with("quit") {
            break;
        } else {
            eprintln!("Unknown command: {}", input);
        }
    }
}
