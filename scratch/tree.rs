use std::cmp::Ord;
use std::comm::Chan;
use std::num::Zero;

enum Tree<T> {
	Node(T, ~Tree<T>, ~Tree<T>, int),
	Nil
}

fn insert<T: Ord>(t: ~Tree<T>, v: T) -> ~Tree<T> {
	match *t {
		Nil => ~Node(v, ~Nil, ~Nil, 1),
		Node(v2, l, r, n) =>
			if v < v2 {
				~Node(v2, insert(l, v), r, n+1)
			} else if v > v2 {
				~Node(v2, l, insert(r, v), n+1)
			} else {
				~Node(v2, l, r, n)	
			}
	}
}

fn sum<T: Num+Send>(t: ~Tree<T>, max: int) -> T {
	match *t {
		Nil => Zero::zero(),
		Node(v, l, r, n) =>
			if n < max {
				v + sum(l, max) + sum(r, max)
			} else {
				task_sum(v, l, r, max)
			}
	}
}

fn task_sum<T: Num+Send>(v: T, l: ~Tree<T>, r: ~Tree<T>, max: int) -> T {
	let (lport, lchan): (Port<T>, Chan<T>) = Chan::new();
	let (rport, rchan): (Port<T>, Chan<T>) = Chan::new();

	spawn(proc() {
		let sum = sum(l, max);
		lchan.send(sum);
	});
	spawn(proc() {
		let sum = sum(r, max);
		rchan.send(sum);
	});

	v + lport.recv() + rport.recv()
}

fn main() {
	let mut t = ~Nil;
	t = insert(t, 0);
	t = insert(t, -2);
	t = insert(t, 2);
	t = insert(t, -1);
	t = insert(t, 1);
	println!("tree: {:?}", t);
	let sum = sum(t, 1);
	println!("sum: {}", sum);
}
