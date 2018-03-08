use data::*;
use parse::*;
use bigdecimal::BigDecimal;
use std::io;
use num::ToPrimitive;

pub struct Executor {
	stack: QuarkStack,
	code: Vec<CodePiece>,
	curr_block: Vec<Vec<CodePiece>>,
	curr_block_pos: Vec<usize>,
	on_block_end: Vec<ExecutorLambda>,
	state: Vec<CMDState>,
}


type ExecutorLambda = fn(&mut Executor, bool) -> bool;

#[derive(Debug)]
enum CMDState {
	None,
	WhileLoop(/*to_execute:*/Vec<CodePiece>)
}

impl Executor {
	pub fn new(code: Vec<CodePiece>) -> Executor {
		Executor {
			code,
			stack: QuarkStack::new(),
			curr_block: vec![],
			curr_block_pos: vec![],
			on_block_end: vec![],
			state: vec![],
		}
	}

	pub fn execute(&mut self) {
		// Lets get down to business.
		let codeiter = self.code.clone();
		let mut codeiter = codeiter.iter();
		loop {
			let to_exec;
			let mut i_hate_you_too_rust = false;
			let blen = self.curr_block.len();
			if blen == 0 {
				to_exec = match codeiter.next() {
					Some(e) => Some(e.clone()),
					None => None,
				};
			} else {
				if self.curr_block[blen-1].len() < self.curr_block_pos[blen-1] {
						to_exec = Some(self.curr_block[blen-1][self.curr_block_pos[blen-1]].clone());
						self.curr_block_pos[blen-1] += 1;
				} else {
						if match self.on_block_end.pop() {
							Some(e) => {
								e(self, true)
							},
							_ => false,
						} {
							i_hate_you_too_rust = false;
							to_exec = Some(self.curr_block[blen-1][self.curr_block_pos[blen-1]].clone());
							self.curr_block_pos[blen-1] += 1;
						} else {
							i_hate_you_too_rust = true;
							to_exec = match codeiter.next() {
								Some(e) => Some(e.clone()),
								None => None,
							}

					}
				};
			}
			if i_hate_you_too_rust {
				self.curr_block.pop();
				self.curr_block_pos.pop();
			}

			match to_exec {
				Some(e) => {
					match e {
						CodePiece::Number(e) => {
							self.stack.push(QuarkType::Number(BigDecimal::new(e,0)));
						}
						CodePiece::Block(ref e) => {
							self.stack.push(QuarkType::Block(e.clone()));
						}
						CodePiece::Point(e) => {
							match e {
								0 | 1=>{/*Newlines exist purely for readability. Spaces are just seperators*/},
								12=>add_cmd(self),
								13=>subtract_cmd(self),
								14=>multiply_cmd(self),
								15=>divide_cmd(self),
								16=>exec_block(self),
								18=>to_range(self),
								28=>get_input(self),
								34=>{while_loop(self, false);},
								37=>{let val = self.stack.pop(); self.stack.push(val.clone()); self.stack.push(val);}
								41=>print_cmd(self),
								53=>self.stack.arraymark(),
								54=>collect_array(self),
								69=>get_length(self),
								74=>{self.stack.pin()},
								_=> {println!("WARNING: Unknown {}", e)}
							}
						}
						_=> {unimplemented!()}
					}
				}
				None => { break; }
			}
		}

		println!("{}", self.stack);

	}
}

fn print_cmd(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Array(ref e) => {
			println!("{:?}", e);
		},
		QuarkType::Number(ref e) => {
			println!("{}", e);
		}
		QuarkType::Block(ref e) => {
			println!("{:?}", e);
		}
	}
}

fn divide_cmd(e: &mut Executor) {
	let x = e.stack.pop();
	let y = e.stack.pop();
	match x {
		QuarkType::Number(x) => {
			match y {
				QuarkType::Number(y) => {
					e.stack.push(QuarkType::Number(x/y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn multiply_cmd(e: &mut Executor) {
	let x = e.stack.pop();
	let y = e.stack.pop();
	match x {
		QuarkType::Number(x) => {
			match y {
				QuarkType::Number(y) => {
					e.stack.push(QuarkType::Number(x*y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn add_cmd(e: &mut Executor) {
	let x = e.stack.pop();
	let y = e.stack.pop();
	match x {
		QuarkType::Number(x) => {
			match y {
				QuarkType::Number(y) => {
					e.stack.push(QuarkType::Number(x+y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn subtract_cmd(e: &mut Executor) {
	let x = e.stack.pop();
	let y = e.stack.pop();
	match x {
		QuarkType::Number(x) => {
			match y {
				QuarkType::Number(y) => {
					e.stack.push(QuarkType::Number(x-y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn exec_block(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Block(to_exec) => {
			e.curr_block_pos.push(0);
			e.curr_block.push(to_exec); //Yay!
		}
		_ => {/*no pls*/}
	}

}

fn collect_array(e: &mut Executor) {
	let mut newarr = vec![];
	loop {
		if e.stack.flags().contains(QuarkMarker::ARRAY_BEGIN) {
			let val = e.stack.pop();
			newarr.push(val);
			e.stack.push(QuarkType::Array(newarr));
			break;
		}
		if e.stack.is_end() {
			e.stack.push(QuarkType::Array(newarr));
			break;
		}
		let val = e.stack.pop();
		newarr.push(val);
	}
}

fn while_loop(e: &mut Executor, islambdad: bool) -> bool {
	if !islambdad {
		let block = match e.stack.pop() {
			QuarkType::Block(e) => {
				e
			}
			_=>{return false; /*RIP*/}
		};
		e.curr_block_pos.push(0);
		e.curr_block.push(block.clone());
		e.on_block_end.push(while_loop);
		e.state.push(CMDState::WhileLoop(block));
		return false;
	}
	if match e.stack.pop() {
		QuarkType::Number(e) => {
			if e == BigDecimal::from(0) {
				false
			} else {
				true
			}
		},
		_ => {true},
	}/*HINT: If block begins here*/{
		e.curr_block_pos.pop();
		e.curr_block.pop();

		e.curr_block.push(match e.state.pop() {
			Some(CMDState::WhileLoop(v)) => {
				e.curr_block_pos.push(0);
				e.state.push(CMDState::WhileLoop(v.clone()));
				v
			}
			v => {
				println!("{:?}\n{:?}\n{:?}", v, e.stack, e.state);
				panic!("Got a non WhileLoop state!") }
		});
		e.on_block_end.push(while_loop);
	};
	return true;
}

fn get_input(e: &mut Executor) {
	println!("Program requested input.");
	let reader = io::stdin();
    let mut input_text = String::new();
    reader.read_line(&mut input_text).expect("failed to read line");
	e.stack.push(QuarkType::Number(match input_text.trim().parse::<BigDecimal>() {
		Ok(e) => {
			e
		}
		_=>{
			panic!("Not a valid number.");
		}
	}));
}

fn get_length(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Array(v) => {
			e.stack.push(QuarkType::Number(BigDecimal::from(v.len() as u64)));
		}
		_=>{},
	}
}

fn to_range(e: &mut Executor) {
	let backup = e.stack.flags();
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					let range = (y.to_u64().unwrap())..=(x.to_u64().unwrap());
					let mut new = vec![];
					println!("{:?}",range);

					for i in range {
						new.push(QuarkType::Number(BigDecimal::from(i)));
					}
					e.stack.push(QuarkType::Array(new));
				}
				_=>{e.stack.push(QuarkType::Number(x));e.stack.flag(backup);},
			}
		},
		_=>{},
	}
}
