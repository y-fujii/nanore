// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
extern crate nanore;
use nanore::*;


fn test_re<T>( ast: Box<RegEx<T>>, fst: bool, seq: &[(T, bool)] ) {
	let re = RegExRoot::new( ast );
	let mut m = Matcher::new( &re );
	assert_eq!( m.is_match(), fst );
	for &(ref v, r) in seq.iter() {
		m.feed( v );
		assert_eq!( m.is_match(), r );
	}
}

fn test_path<T, U: Copy + PartialEq + std::fmt::Debug>( ast: Box<RegEx<T, U>>, seq: &[T], path: &[(usize, U)] ) {
	let re = RegExRoot::new( ast );
	let mut m = Matcher::new( &re );
	for v in seq.iter() {
		m.feed( v );
	}
	assert_eq!( m.path(), path );
}

#[test]
fn test00() {
	test_re(
		val('a') * opt(val('b')) * rep(val('c')),
		false, &[ ('a', true), ('b', true), ('c', true), ('c', true), ('c', true), ('b', false) ],
	);
}

#[test]
fn test01() {
	test_re(
		any() * opt(val('b')) * rep(val('c')) * opt(val('x') + val('y') + val('z')),
		false, &[ ('a', true), ('b', true), ('c', true), ('z', true), ('c', false), ('y', false) ],
	);
}

#[test]
fn test02() {
	test_re(
		atom(|e| *e > 0) * rep(atom(|e| *e == 0)) * atom(|e| *e < 0),
		false, &[ ( 1, false), (-1, true) ],
	);
}

#[test]
fn test03() {
	test_re(
		atom(|e| *e % 2 != 0) * atom(|e| *e % 2 == 0) + rep(atom(|e| *e > 0)),
		true, &[ (1, true), (2, true), (3, true), (0, false) ],
	);
}

#[test]
fn test04() {
	test_re(
		rep(rep(val('a'))),
		true, &[ ('a', true), ('a', true), ('b', false) ],
	);
}

#[test]
fn test05() {
	test_re(
		opt(rep(val('a'))),
		true, &[ ('a', true), ('a', true), ('b', false) ],
	);
}

#[test]
fn test06() {
	test_re(
		rep(opt(rep(val('a')))) * opt(val('b')) * rep(val('b')),
		true, &[ ('a', true), ('a', true), ('b', true), ('b', true), ('c', false) ],
	);
}

#[test]
fn test07() {
	test_re(
		opt(val('a') + val('b')),
		true, &[ ('a', true), ('a', false) ],
	);
}

#[test]
fn test08() {
	test_re(
		opt(opt(val('a')) * val('a') + val('b')),
		true, &[ ('a', true), ('a', true), ('a', false) ],
	);
}

#[test]
fn test09() {
	test_re(
		opt(val('x')) * val('a'),
		false, &[ ('a', true), ('a', false) ],
	);
}

#[test]
fn test10() {
	test_path(
		mark(0) * val('a') * mark(1) * val('b') * mark(2),
		&[ 'a', 'b' ],
		&[ (0, 0), (1, 1), (2, 2) ],
	);
}

#[test]
fn test11() {
	test_path(
		(mark(0) * val('a') + mark(1) * val('b')) * val('c') * mark(2),
		&[ 'b', 'c' ],
		&[ (0, 1), (2, 2) ],
	);
}

#[test]
fn test12() {
	test_path(
		(mark(0) * val('a') + mark(1) * opt(weight(-1) * val('a'))) * val('c') * mark(2),
		&[ 'a', 'c' ],
		&[ (0, 1), (2, 2) ],
	);
}

#[test]
fn test13() {
	test_path(
		rep(weight(1) * mark(0) * val('a')) * rep(weight(-1) * mark(1) * val('a')),
		&[ 'a', 'a', 'a' ],
		&[ (0, 1), (1, 1), (2, 1) ],
	);
}
