use std::usize;

use cozy_chess::*;

pub const SEE_VALS: [i32; 6] = [100, 450, 450, 650, 1250, 0];

const HIDDEN: usize = 1024;
const SCALE: i32 = 400;
const QA: i32 = 255;
const QB: i32 = 64;
const QAB: i32 = QA * QB;

#[macro_export]
macro_rules! bitloop {
    (| $bb:expr, $sq:ident | $func:expr) => {
        while $bb > 0 {
            let $sq = $bb.trailing_zeros() as u8;
            $bb &= $bb - 1;
            $func;
        }
    };
}

pub unsafe fn boxed_and_zeroed<T>() -> Box<T> {
    let layout = std::alloc::Layout::new::<T>();
    let ptr = unsafe { std::alloc::alloc_zeroed(layout) };
    if ptr.is_null() {
        std::alloc::handle_alloc_error(layout);
    }
    unsafe { Box::from_raw(ptr.cast()) }
}

#[repr(C)]
pub struct Network {
    feature_weights: [Accumulator; 768 * NUM_BUCKETS],
    feature_bias: Accumulator,
    output_weights: [Accumulator; 2],
    output_bias: i16,
}

static NNUE: Network =
    unsafe { std::mem::transmute(*include_bytes!(concat!("../resources/net.bin"))) };

const NUM_BUCKETS: usize = 4;

#[rustfmt::skip]
static BUCKETS: [usize; 64] = [
    0, 0, 1, 1, 5, 5, 4, 4,
    2, 2, 2, 2, 6, 6, 6, 6,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
    3, 3, 3, 3, 7, 7, 7, 7,
];

impl Network {
    pub fn out(boys: &Accumulator, opps: &Accumulator) -> i32 {
        let weights = &NNUE.output_weights;
        let sum = flatten(boys, &weights[0]) + flatten(opps, &weights[1]);
        (sum / QA + i32::from(NNUE.output_bias)) * SCALE / QAB
    }

    pub fn get_bucket<const COLOR: usize>(mut ksq: usize) -> usize {
        if COLOR == 1 {
            ksq ^= 56;
        }

        BUCKETS[ksq]
    }

    pub fn get_base_index<const COLOR: usize>(side: usize, pc: usize, mut ksq: usize) -> usize {
        if ksq % 8 > 3 {
            ksq ^= 7;
        }

        if COLOR == 0 {
            768 * Self::get_bucket::<0>(ksq) + [0, 384][side] + 64 * pc
        } else {
            768 * Self::get_bucket::<1>(ksq) + [384, 0][side] + 64 * pc
        }
    }
}

#[derive(Clone, Copy)]
#[repr(C, align(64))]
pub struct Accumulator {
    vals: [i16; HIDDEN],
}

impl Accumulator {
    pub fn update_multi(&mut self, adds: &[u16], subs: &[u16]) {
        const REGS: usize = 8;
        const PER: usize = REGS * 16;

        let mut regs = [0i16; PER];

        for i in 0..HIDDEN / PER {
            let offset = PER * i;

            for (j, reg) in regs.iter_mut().enumerate() {
                *reg = self.vals[offset + j];
            }

            for &add in adds {
                let weights = &NNUE.feature_weights[usize::from(add)];

                for (j, reg) in regs.iter_mut().enumerate() {
                    *reg += weights.vals[offset + j];
                }
            }

            for &sub in subs {
                let weights = &NNUE.feature_weights[usize::from(sub)];

                for (j, reg) in regs.iter_mut().enumerate() {
                    *reg -= weights.vals[offset + j];
                }
            }

            for (j, reg) in regs.iter().enumerate() {
                self.vals[offset + j] = *reg;
            }
        }
    }
}

impl Default for Accumulator {
    fn default() -> Self {
        NNUE.feature_bias
    }
}

pub struct EvalEntry {
    pub bbs: [u64; 8],
    pub white: Accumulator,
    pub black: Accumulator,
}

pub struct EvalTable {
    pub table: Box<[[EvalEntry; 2 * NUM_BUCKETS]; 2 * NUM_BUCKETS]>,
}

impl Default for EvalTable {
    fn default() -> Self {
        let mut table: Box<[[EvalEntry; 2 * NUM_BUCKETS]; 2 * NUM_BUCKETS]> =
            unsafe { boxed_and_zeroed() };

        for row in table.iter_mut() {
            for entry in row.iter_mut() {
                entry.white = Accumulator::default();
                entry.black = Accumulator::default();
            }
        }

        Self { table }
    }
}

fn flatten(acc: &Accumulator, weights: &Accumulator) -> i32 {
    #[cfg(not(target_feature = "avx2"))]
    {
        fallback::flatten(acc, weights)
    }
    #[cfg(target_feature = "avx2")]
    unsafe {
        avx2::flatten(acc, weights)
    }
}

#[cfg(not(target_feature = "avx2"))]
mod fallback {
    use super::{Accumulator, QA};

    #[inline]
    pub fn screlu(x: i16) -> i32 {
        i32::from(x.clamp(0, QA as i16)).pow(2)
    }

    #[inline]
    pub fn flatten(acc: &Accumulator, weights: &Accumulator) -> i32 {
        let mut sum = 0;

        for (&x, &w) in acc.vals.iter().zip(&weights.vals) {
            sum += screlu(x) * i32::from(w);
        }

        sum
    }
}

#[cfg(target_feature = "avx2")]
mod avx2 {
    use super::{Accumulator, HIDDEN, QA};
    use std::arch::x86_64::*;

    pub unsafe fn flatten(acc: &Accumulator, weights: &Accumulator) -> i32 {
        use std::arch::x86_64::*;

        const CHUNK: usize = 16;

        let mut sum = _mm256_setzero_si256();
        let min = _mm256_setzero_si256();
        let max = _mm256_set1_epi16(QA as i16);

        for i in 0..HIDDEN / CHUNK {
            let mut v = load_i16s(acc, i * CHUNK);
            v = _mm256_min_epi16(_mm256_max_epi16(v, min), max);
            let w = load_i16s(weights, i * CHUNK);
            let product = _mm256_madd_epi16(v, _mm256_mullo_epi16(v, w));
            sum = _mm256_add_epi32(sum, product);
        }

        horizontal_sum_i32(sum)
    }

    #[inline]
    unsafe fn load_i16s(acc: &Accumulator, start_idx: usize) -> __m256i {
        _mm256_load_si256(acc.vals.as_ptr().add(start_idx).cast())
    }

    #[inline]
    unsafe fn horizontal_sum_i32(sum: __m256i) -> i32 {
        let upper_128 = _mm256_extracti128_si256::<1>(sum);
        let lower_128 = _mm256_castsi256_si128(sum);
        let sum_128 = _mm_add_epi32(upper_128, lower_128);
        let upper_64 = _mm_unpackhi_epi64(sum_128, sum_128);
        let sum_64 = _mm_add_epi32(upper_64, sum_128);
        let upper_32 = _mm_shuffle_epi32::<0b00_00_00_01>(sum_64);
        let sum_32 = _mm_add_epi32(upper_32, sum_64);

        _mm_cvtsi128_si32(sum_32)
    }
}

pub fn eval(board: &Board, cache: &mut EvalTable) -> i32 {
    let wksq = board.king(Color::White) as usize;
    let bksq = board.king(Color::Black) as usize;

    let wbucket = Network::get_bucket::<0>(wksq);
    let bbucket = Network::get_bucket::<1>(bksq);

    let entry = &mut cache.table[wbucket][bbucket];

    let mut addf = [[0; 32]; 2];
    let mut subf = [[0; 32]; 2];

    let (adds, subs) = fill_diff(board, &entry.bbs, &mut addf, &mut subf);

    entry.white.update_multi(&addf[0][..adds], &subf[0][..subs]);
    entry.black.update_multi(&addf[1][..adds], &subf[1][..subs]);

    entry.bbs = [
        board.colors(Color::White).0,
        board.colors(Color::Black).0,
        board.pieces(Piece::Pawn).0,
        board.pieces(Piece::Knight).0,
        board.pieces(Piece::Bishop).0,
        board.pieces(Piece::Rook).0,
        board.pieces(Piece::Queen).0,
        board.pieces(Piece::King).0,
    ];

    eval_from_accs(board, &entry.white, &entry.black)
}

fn eval_from_accs(board: &Board, white: &Accumulator, black: &Accumulator) -> i32 {
    let eval = if board.side_to_move() == Color::White {
        Network::out(white, black)
    } else {
        Network::out(black, white)
    };

    scale(board, eval)
}

pub fn eval_from_scratch(board: &Board) -> i32 {
    let mut table = EvalTable::default();
    eval(board, &mut table)
}
fn scale(board: &Board, eval: i32) -> i32 {
    let mut mat = board.pieces(Piece::Knight).len() as i32 * SEE_VALS[Piece::Knight as usize]
        + board.pieces(Piece::Bishop).len() as i32 * SEE_VALS[Piece::Bishop as usize]
        + board.pieces(Piece::Rook).len() as i32 * SEE_VALS[Piece::Rook as usize]
        + board.pieces(Piece::Queen).len() as i32 * SEE_VALS[Piece::Queen as usize];

    mat = 700 + mat / 32;

    eval * mat / 1024
}
fn fill_diff(
    board: &Board,
    bbs: &[u64; 8],
    add_feats: &mut [[u16; 32]; 2],
    sub_feats: &mut [[u16; 32]; 2],
) -> (usize, usize) {
    let mut adds = 0;
    let mut subs = 0;

    let wksq = board.king(Color::White) as usize;
    let bksq = board.king(Color::Black) as usize;

    let wflip = if wksq % 8 > 3 { 7 } else { 0 };
    let bflip = if bksq % 8 > 3 { 7 } else { 0 } ^ 56;

    for side in [Color::White, Color::Black] {
        let old_boys = bbs[side as usize];
        let new_boys = board.colors(side).0;

        for (piece, &(mut old_bb)) in bbs[Piece::Pawn as usize..=Piece::King as usize].iter().enumerate() {
            old_bb &= old_boys;
            let new_bb = board.pieces(Piece::index(piece)).0 & new_boys;

            let wbase = Network::get_base_index::<0>(side as usize, piece, wksq) as u16;
            let bbase = Network::get_base_index::<1>(side as usize, piece, bksq) as u16;

            let mut add_diff = new_bb & !old_bb;
            bitloop!(|add_diff, sq| {
                let sq = u16::from(sq);
                add_feats[0][adds] = wbase + (sq ^ wflip);
                add_feats[1][adds] = bbase + (sq ^ bflip);
                adds += 1;
            });

            let mut sub_diff = old_bb & !new_bb;
            bitloop!(|sub_diff, sq| {
                let sq = u16::from(sq);
                sub_feats[0][subs] = wbase + (sq ^ wflip);
                sub_feats[1][subs] = bbase + (sq ^ bflip);
                subs += 1;
            });
        }
    }

    (adds, subs)
}
