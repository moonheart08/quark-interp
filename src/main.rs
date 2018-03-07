extern crate bit_vec;
use bit_vec::BitVec;

fn main() {
	println!("NOTICE: This only works on Little Endian machines. Careful!");
	println!("{:?}", unencoded_to_id(include_str!("testfile").into()));
}

fn decode(data: &[u8]) -> String {
	let cp = include!("codepage"); //FIXME: This should be a constant or something.

	let mut data = BitVec::from_bytes(data); // Converted data
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

fn decode_id(data: &[u8]) -> Vec<usize> {
	let cp = include!("codepage"); //FIXME: This should be a constant or something.

	let mut data = BitVec::from_bytes(data); // Converted data
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
fn unencoded_to_id(data: String) -> Vec<usize> {
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
