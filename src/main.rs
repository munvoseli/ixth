use std::fs::File;
use std::io::Read;

fn split_on_ws(s: Vec<u8>) -> (Vec<Vec<u8>>, Vec<usize>) {
	let mut g = Vec::<Vec<u8>>::new();
	let mut lines = Vec::<usize>::new();
	let mut i = 0;
	let mut linen = 1;
	loop {
		let mut h = Vec::<u8>::new();
		while i < s.len() && s[i] > 0x20 {
			h.push(s[i]);
			i += 1;
		}
		g.push(h);
		lines.push(linen);
		while i < s.len() && s[i] <= 0x20 {
			if s[i] == 10 {
				linen += 1;
			}
			i += 1;
		}
		if i == s.len() { break; }
	}
	(g, lines)
}

fn find_ifterm(mut i: usize, c: &Vec<Vec<u8>>) -> usize {
	// if can terminate on fi or else, but not if the fi or else is in nest
	let mut fict = 0;
	assert!(c[i] == b"if");
	i += 1;
	loop {
		if c[i] == b"if" {
			fict += 1;
		}
		else if c[i] == b"fi" {
			if fict == 0 { break; }
			fict -= 1;
		}
		else if c[i] == b"else" {
			if fict == 0 { break; }
		}
		i += 1;
	}
	return i;
}
fn find_fi(mut i: usize, c: &Vec<Vec<u8>>) -> usize {
	// move past ifs and elses to fi
	let mut fict = 0;
	loop {
		if c[i] == b"if" {
			fict += 1;
		}
		else if c[i] == b"fi" {
			fict -= 1;
			if fict == 0 { break; }
		}
		else if c[i] == b"else" {
			fict -= 1;
		}
		i += 1;
	}
	return i;
}

fn checks(c: &Vec<Vec<u8>>, fdecs: &mut Vec<usize>) {
	let mut elct = 0;
	let mut ifct = 0;
	let mut fict = 0;
	for i in 0..c.len() {
		if c[i] == b"if" { ifct += 1; }
		else if c[i] == b"else" { elct += 1; }
		else if c[i] == b"fi" { fict += 1; }
		else if c[i] == b"func" { fdecs.push(i + 1); }
	}
	assert!(ifct - elct - fict == 0, "Every else needs an if.");
}

fn operate_the_stack(mut i: usize, c: &Vec<Vec<u8>>, stack: &mut Vec<u64>) -> usize {
	assert!(c[i] == b"(");
	i += 1;
	let ia = i;
	while c[i] != b"--" { i += 1 };
	let pops = i - ia;
	let mut rems = Vec::new();
	for _ in 0..pops {
		let k = stack.pop().unwrap();
		rems.push(k);
	}
	assert!(c[i] == b"--");
	loop {
		i += 1;
		if c[i] == b")" { break; }
		let mut j = ia;
		while c[j] != b"--" && c[j] != c[i] {
			j += 1;
		}
		assert!(c[j] != b"--", "re-order is bad");
		let ri = (pops - 1) - (j - ia);
		stack.push(rems[ri]);
	}
	assert!(c[i] == b")");
	return i;
}

fn things_eq(a: &[u8], b: &[u8]) -> bool {
	if a.len() != b.len() {
//		println!("lengths not equal: {} and {}", a.len(), b.len());
		return false;
	}
	for i in 0..a.len() {
		if a[i] != b[i] {
//			println!("not equal at {}: {} and {}", i, a[i], b[i]);
			return false;
		}
	}
	return true;
}

fn main() {
	let mut f = File::open("h.txt").unwrap();
	let mut s = Vec::<u8>::new();
	f.read_to_end(&mut s).unwrap();
	println!("Read {} bytes", s.len());
	let (c, cline) = split_on_ws(s);
	let mut fndecs = Vec::<usize>::new();
	checks(&c, &mut fndecs);
	let mut stack = Vec::<u64>::new();
	let mut posstack = Vec::<usize>::new();
	let mut i = 0;
	while i < c.len() {
//		println!("{:x?} {:?}", c[i], stack);
		println!("    stack: {:?}", stack);
		println!("    op: {} {:x?} on line {}", i, c[i], cline[i]);
//		println!("{:x?}", c[i]);
		if c[i] == b"(" {
			i = operate_the_stack(i, &c, &mut stack);
			assert!(c[i] == b")");
		} else if c[i] == b"if" {
			if stack.pop().unwrap() == 0 {
//				let bi = i;
				// move past else/fi label
				i = find_ifterm(i, &c) + 1;
//				println!("if going from line {} to {}",cline[bi],cline[i]);
				continue;
			}
		} else if c[i] == b"else" {
			// move past fi label
			i = find_fi(i + 1, &c) + 1;
			continue;
		} else if c[i] == b"fi" {
		} else if c[i] == b"{" {
//			println!("      passing into block");
		} else if c[i] == b"}" {
			println!("tried to exit block without gof");
		} else if c[i] == b"add" {
			if stack.len() >= 2 {
				let a = stack.pop().unwrap();
				let b = stack.pop().unwrap();
				stack.push(a + b);
			}
		} else if c[i] == b"sub" {
			let a = stack.pop().unwrap();
			let b = stack.pop().unwrap();
			stack.push(b - a);
		} else if c[i] == b"print" {
			println!("{}", stack.pop().unwrap());
		} else if c[i] == b"ret" {
			i = posstack.pop().unwrap();
		} else if c[i] == b"func" {
			while c[i] != b"ret" { i += 1; }
		} else if c[i] == b"gob" {
			let mut j = stack.pop().unwrap();
			let bi = i;
			while j > 0 {
				i -= 1;
				if c[i] == b"{" { j -= 1; }
			}
			println!("gob to line {} from line {}", cline[i], cline[bi]);
		} else if c[i] == b"gof" {
			let mut j = stack.pop().unwrap();
			while j > 0 {
				i += 1;
				if c[i] == b"}" { j -= 1; }
			}
			println!("gof to symbol on line {}", cline[i]);
		} else {
			if c[i][0] >= 0x30 && c[i][0] <= 0x39 {
				stack.push((c[i][0] - 0x30).into());
			} else {
				let mut found = false;
				for j in 0..fndecs.len() {
					let fni = fndecs[j];
//					println!("cmp {:x?} {:x?}", c[fni], c[i]);
					if things_eq(&c[fni], &c[i]) {
						posstack.push(i);
						i = fni;
						found = true;
//						println!("cmp good");
						break;
					}
				}
				assert!(found, "undefined function {:x?}", c[i]);
			}
		}
		i += 1;
	}
	println!("Hello, world!");
}
