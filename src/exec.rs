use data::*;
use parse::*;
use bigdecimal::BigDecimal;
use std::io;
use num::*;

macro_rules! qNumber {
	($e:expr) => {
		QuarkType::Number(BigDecimal::from($e))
	}
}

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
	WhileLoop(/*to_execute:*/Vec<CodePiece>),
	RepeatLoop(Vec<CodePiece>),
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
				if self.curr_block[blen-1].len() > self.curr_block_pos[blen-1] {
					to_exec = Some(self.curr_block[blen-1][self.curr_block_pos[blen-1]].clone());
					self.curr_block_pos[blen-1] += 1;
				} else {
						if match self.on_block_end.pop() {
							Some(e) => {
								e(self, true)
							},
							_ => false,
						} {
							let blen = self.curr_block.len();
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
								19=>{self.stack.push(qNumber!(360))}
								20=>{self.stack.push(qNumber!(180))}
								21=>{self.stack.push(qNumber!(90))}
								22=>{self.stack.push(qNumber!(270))}
								25=>ceil(self),
								26=>floor(self),
								28=>get_input(self),
								31=>{repeat_loop(self, false);}
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
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					e.stack.push(qNumber!(x/y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn multiply_cmd(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					e.stack.push(qNumber!(x*y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn add_cmd(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					e.stack.push(qNumber!(x+y));
				}
				_ => {},
			}
		}
		_ => {},
	}
}

fn subtract_cmd(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					e.stack.push(qNumber!(x-y));
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
		_ => {}
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
			_=>{return false;}
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
		_ => {false},
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
	} else {
		e.curr_block_pos.pop();
		e.curr_block.pop();
		e.state.pop();
		return false;
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
	match e.stack.pop() {
		QuarkType::Number(x) => {
			match e.stack.pop() {
				QuarkType::Number(y) => {
					let range = (y.to_u64().unwrap())..=(x.to_u64().unwrap());
					let mut new = vec![];

					for i in range {
						new.push(QuarkType::Number(BigDecimal::from(i)));
					}
					e.stack.push(QuarkType::Array(new));
				}
				_=>{},
			}
		},
		_=>{},
	}
}

//FIXME: This is causes a loss of precision. Remove these when implemented in BigDecimal.
fn floor_bd(v: BigDecimal) -> BigDecimal {
	let mut v = v.to_f32().unwrap();
	return BigDecimal::from(v.floor());
}

fn ceil_bd(v: BigDecimal) -> BigDecimal {
	let mut v = v.to_f32().unwrap();
	return BigDecimal::from(v.ceil());
}

fn floor(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Number(x) => {
			e.stack.push(qNumber!(floor_bd(x)));
		}
		v=>{e.stack.push(v)}
	}
}

fn ceil(e: &mut Executor) {
	match e.stack.pop() {
		QuarkType::Number(x) => {
			e.stack.push(qNumber!(ceil_bd(x)));
		}
		v=>{e.stack.push(v)}
	}
}

fn repeat_loop(e: &mut Executor, lambdad: bool) -> bool {
	if !lambdad {
		let block = match e.stack.pop() {
			QuarkType::Block(e) => {
				e
			}
			_=>{return false;}
		};
		e.curr_block_pos.push(0);
		e.curr_block.push(block.clone());
		e.on_block_end.push(repeat_loop);
		e.state.push(CMDState::RepeatLoop(block));
		return false;
	}
	e.curr_block_pos.pop();
	e.curr_block.pop();

	e.curr_block.push(match e.state.pop() {
		Some(CMDState::RepeatLoop(v)) => {
			e.curr_block_pos.push(0);
			e.state.push(CMDState::RepeatLoop(v.clone()));
			v
		}
		v => {
			println!("{:?}\n{:?}\n{:?}", v, e.stack, e.state);
			panic!("Got a non RepeatLoop state!") }
	});
	e.on_block_end.push(repeat_loop);
	return true;
}
