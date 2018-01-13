// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate nanore;


fn test_re<T>( ast: Box<nanore::RegEx<T>>, fst: bool, seq: &[(T, bool)] ) {
	let mut re = nanore::RegExRoot::new( ast );
	assert!( re.is_match() == fst );
	for &(ref v, r) in seq.iter() {
		re.feed( v );
		assert!( re.is_match() == r );
	}
}

#[test]
fn it_works() {
	use nanore::*;

	test_re(
		val('a') * opt(val('b')) * rep(val('c')),
		false, &[ ('a', true), ('b', true), ('c', true), ('c', true), ('c', true), ('b', false) ],
	);
	test_re(
		any() * opt(val('b')) * rep(val('c')) * opt(val('x') + val('y') + val('z')),
		false, &[ ('a', true), ('b', true), ('c', true), ('z', true), ('c', false), ('y', false) ],
	);
	test_re(
		atom(|e| *e > 0) * rep(atom(|e| *e == 0)) * atom(|e| *e < 0),
		false, &[ ( 1, false), (-1, true) ],
	);
	test_re(
		atom(|e| *e % 2 != 0) * atom(|e| *e % 2 == 0) + rep(atom(|e| *e > 0)),
		true, &[ (1, true), (2, true), (3, true), (0, false) ],
	);
	test_re(
		rep(rep(val('a'))),
		true, &[ ('a', true), ('a', true), ('b', false) ],
	);
	test_re(
		opt(rep(val('a'))),
		true, &[ ('a', true), ('a', true), ('b', false) ],
	);
	test_re(
		rep(opt(rep(val('a')))) * opt(val('b')) * rep(val('b')),
		true, &[ ('a', true), ('a', true), ('b', true), ('b', true), ('c', false) ],
	);
}
