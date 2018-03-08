use bigdecimal::BigDecimal;
use parse::*;
use std;

bitflags! {
	pub struct QuarkMarker: u32 {
		const NONE        = 0b00000000;
		const ARRAY_BEGIN = 0b10000000;
		const PINNED      = 0b01000000;
		const INPUT_VAR   = 0b00100000;
	}
}

#[derive(Clone, Debug)]
pub enum QuarkType {
	Number(BigDecimal),
	Array(Vec<QuarkType>),
	/// Contains the decoded program.
	Block(Vec<CodePiece>),
}

#[derive(Debug)]
pub struct QuarkStack {
	container: Vec<(QuarkType, QuarkMarker)>
}

impl QuarkStack {
	pub fn new() -> QuarkStack {
		QuarkStack {
			container: vec![],
		}
	}

	pub fn pop(&mut self) -> QuarkType {
		let val = match self.container.pop() {
			Some(e) => e,
			None => (QuarkType::Number(BigDecimal::from(0)), QuarkMarker::NONE),
		};
		if val.1.contains(QuarkMarker::PINNED) {
			let newv = val.0.clone();
			self.container.push(val);
			return newv;
		} else if val.1.contains(QuarkMarker::INPUT_VAR) {
			unimplemented!();
		}
		val.0
	}

	pub fn get(&mut self, i: usize) -> Option<(QuarkType, QuarkMarker)> {
		match self.container.get(i) {
			Some(e) => Some(e.clone()),
			None => None,
		}
	}

	pub fn push(&mut self, v: QuarkType) {
		self.container.push((v, QuarkMarker::NONE));
	}

	pub fn flags(&mut self) -> QuarkMarker {
		if self.container.len() == 0 {
			self.container.push((QuarkType::Number(BigDecimal::from(0)), QuarkMarker::NONE))
		}
		let val = self.container.pop();
		let val2 = match val.clone() {
			Some(e) => e.1,
			None => unreachable!(),
		};
		self.container.push(val.unwrap());
		return val2;
	}

	pub fn flag(&mut self, e: QuarkMarker) {
		if self.container.len() == 0 {
			self.container.push((QuarkType::Number(BigDecimal::from(0)), QuarkMarker::NONE))
		}
		let len = self.container.len()-1;
		self.container[len].1.set(e, true);
	}

	pub fn pin(&mut self) {
		if self.container.len() == 0 {
			self.container.push((QuarkType::Number(BigDecimal::from(0)), QuarkMarker::NONE))
		}
		let len = self.container.len()-1;
		self.container[len].1.toggle(QuarkMarker::PINNED);
	}

	pub fn arraymark(&mut self) {
		if self.container.len() == 0 {
			self.container.push((QuarkType::Number(BigDecimal::from(0)), QuarkMarker::NONE))
		}
		let len = self.container.len()-1;
		self.container[len].1.toggle(QuarkMarker::ARRAY_BEGIN);
	}

	pub fn is_end(&mut self) -> bool{
		if self.container.len() == 0 {
			return true;
		} else {
			return false;
		}
	}
}

impl std::fmt::Display for QuarkStack {
	// magic!
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "STACK");
		for i in self.container.iter() {
			writeln!(f, "{}", format_qsen(i));
		}
		writeln!(f, "END STACK")
	}
}

fn format_qsen(v: &(QuarkType, QuarkMarker)) -> String {
	match &v.0 {
		&QuarkType::Array(ref e) => {
			let mut out = String::new();
			out += "[\n".into();
			for i in e {
				out += &format_arr(i);
			}
			out += "]".into();
			return out;
		},
		&QuarkType::Number(ref e) => {
			format!("{}:{:?}", e, v.1)
		}
		&QuarkType::Block(ref e) => {
			format!("{:?}:{:?}", e, v.1)
		}
	}
}

fn format_arr(v: &QuarkType) -> String {
	match v {
		&QuarkType::Array(ref e) => {
			let mut out = String::new();
			out += "[\n".into();
			for i in e {
				out += &format_arr(i);
			}
			out += "]".into();
			return out;
		},
		&QuarkType::Number(ref e) => {
			format!("{},\n", e)
		}
		&QuarkType::Block(ref e) => {
			format!("{:?},\n", e)
		}
	}
}
