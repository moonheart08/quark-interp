use bit_vec::BitVec;
use num::bigint::BigInt;
use std;

const OPEN_BLOCK: usize  = 32;
const CLOSE_BLOCK: usize = 33;
const NUM_START: usize = 2;
const NUM_END: usize = 11;

#[derive(Debug, Clone)]
pub enum CodePiece {
	Point(usize),
	Block(Vec<CodePiece>),
	Number(BigInt),
}

pub fn parse_to_executable(v: Vec<usize>) -> Vec<CodePiece> {
	let mut block_depth = 0;
	let mut in_num_mode = false;
	let mut fullprog = vec![];
	let mut num = BigInt::from(0);
	let mut inprog_blocks: Vec<Vec<CodePiece>> = vec![]; // Stores blocks the parser is busy constructing.

	for i in v.iter() {
	 	match *i {
			OPEN_BLOCK => {
				if in_num_mode == true {
					in_num_mode = false;
					if block_depth == 0 {
						fullprog.push(CodePiece::Number(num));
					} else {
						let top = inprog_blocks.len()-1;
						inprog_blocks[top].push(CodePiece::Number(num));
					}
					num = BigInt::from(0);
				}
				block_depth += 1;
				inprog_blocks.push(vec![]);
			}

			CLOSE_BLOCK => {
				if in_num_mode == true {
					in_num_mode = false;
					if block_depth == 0 {
						fullprog.push(CodePiece::Number(num));
					} else {
						let top = inprog_blocks.len()-1;
						inprog_blocks[top].push(CodePiece::Number(num));
					}
					num = BigInt::from(0);
				}
				block_depth -= 1;
				let newblock = inprog_blocks.pop().unwrap();
				fullprog.push(CodePiece::Block(newblock));
			}

			NUM_START...NUM_END => {
				let realval = *i-NUM_START; // Will be 0 through 9
				in_num_mode = true;
				num = BigInt::from(10) * num;
				num = realval + num;
			}
			_ => {
				if in_num_mode == true {
					in_num_mode = false;
					if block_depth == 0 {
						fullprog.push(CodePiece::Number(num));
					} else {
						let top = inprog_blocks.len()-1;
						inprog_blocks[top].push(CodePiece::Number(num));
					}
					num = BigInt::from(0);
				}
				if block_depth == 0 {
					fullprog.push(CodePiece::Point(*i));
				} else {
					let top = inprog_blocks.len()-1;
					inprog_blocks[top].push(CodePiece::Point(*i));
				}
			}
		}
	}

	return fullprog;
}

pub fn decode(data: &[u8]) -> String {
	let cp = include!("codepage"); //FIXME: This should be a constant or something.

	let data = BitVec::from_bytes(data); // Converted data
	let mut chunk = BitVec::with_capacity(9); // Quick buffer for bits
	let mut decoded = String::new();

	for i in data.iter() {
		chunk.push(i);
		if chunk.len() == 9 { // We got the bits we need to decode!
			let value = chunk.blocks().next().unwrap();
			decoded.push(cp[value as usize]);
			chunk.clear();
		}
	}

	return decoded; //Hurray!
}

pub fn decode_id(data: &[u8]) -> Vec<usize> {
	let data = BitVec::from_bytes(data); // Converted data
	let mut chunk = BitVec::with_capacity(9); // Quick buffer for bits, to store things.
	let mut decoded = Vec::new();

	for i in data.iter() {
		chunk.push(i);
		if chunk.len() == 9 { // We got the bits we need to decode!
			let value = chunk.blocks().next().unwrap();
			decoded.push(value as usize);
			chunk.clear();
		}
	}

	return decoded; //Hurray!
}

//FIXME: SLOW!!!
pub fn unencoded_to_id(data: String) -> Vec<usize> {
	let cp = include!("codepage"); //FIXME: This should be a constant or something.
	let mut res = Vec::new();

	for i in data.chars() {
		match cp.iter().position(|x| x == &i) {
			Some(e) => res.push(e),
			_ => {/*ignore unknown characters*/},
		};
	}

	return res;
}
