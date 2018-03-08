#![feature(inclusive_range_syntax)]

extern crate bit_vec;
extern crate bigdecimal;
extern crate num;
#[macro_use] extern crate bitflags;

use std::fs::*;
use std::io::prelude::*;

pub mod data;
pub mod parse;
pub mod exec;

fn main() {
	println!("NOTICE: This only works on Little Endian machines. Careful!");
	let mut file = match std::env::args().nth(1) {
		Some(e) => {
			match File::open(e) {
				Ok(f) => {
					f
				}
				Err(e) => {
					println!("Could not open file.\n{}",e);
					return;
				}
			}

		}
		None => {
			println!("Please input a file name to execute from.");
			return;
		}
	};


	let mut contents = String::new();
	file.read_to_string(&mut contents);
	let execdata = parse::parse_to_executable(parse::unencoded_to_id(contents));
	let mut exec = exec::Executor::new(execdata);
	exec.execute();
}
