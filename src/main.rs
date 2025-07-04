use cozy_chess::*;

fn main() {
    let mut board: Board = Board::default();
    let mut input: String = String::new();
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
        } else if input.starts_with("go") {
            let words: Vec<&str> = input.split_whitespace().collect();
            let mut i: usize = 0;
            let mut wtime: u64 = 0;
            let mut btime: u64 = 0;
            let mut movetime: u64 = 0;
            while i < words.len() {
                match words[i] {
                    "wtime" | "btime" | "winc" | "binc" | "movetime" => {
                        if i + 1 < words.len() {
                            if let Ok(value) = words[i + 1].parse::<u64>() {
                                match words[i] {
                                    "wtime" => wtime = value,
                                    "btime" => btime = value,
                                    "movetime" => movetime = value,
                                    _ => (),
                                }
                            } else {
                                eprintln!("Error parsing {}: Invalid number", words[i]);
                            }
                            i += 2;
                        } else {
                            eprintln!("Missing value for {}", words[i]);
                            i += 1;
                        }
                    }
                    _ => i += 1,
                }
            }
            //if the input uses wtime/btime, use that, otherwise use movetime
            let time_remaining = if board.side_to_move() == Color::White {
                if wtime > 0 { wtime } else { movetime }
            } else {
                if btime > 0 { btime } else { movetime }
            };
            //let best_move: String = searcher.get_best_move(&board, time_remaining);
            //println!("bestmove {}", best_move);
        } else if input.starts_with("quit") {
            break;
        } else {
            eprintln!("Unknown command: {}", input);
        }
    }
}
