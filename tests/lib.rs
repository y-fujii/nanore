// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
extern crate nanore;


#[test]
fn it_works() {
	use nanore::*;

	let mut re = RegExRoot::new( val('a') * opt(val('b')) * rep(val('c')) );
	assert!( !re.is_match() );
	re.feed( &'a' ); assert!(  re.is_match() );
	re.feed( &'b' ); assert!(  re.is_match() );
	re.feed( &'c' ); assert!(  re.is_match() );
	re.feed( &'c' ); assert!(  re.is_match() );
	re.feed( &'c' ); assert!(  re.is_match() );
	re.feed( &'b' ); assert!( !re.is_match() );

	let mut re = RegExRoot::new( any() * opt(val('b')) * rep(val('c')) * opt(val('x') + val('y') + val('z')) );
	assert!( !re.is_match() );
	re.feed( &'a' ); assert!(  re.is_match() );
	re.feed( &'b' ); assert!(  re.is_match() );
	re.feed( &'c' ); assert!(  re.is_match() );
	re.feed( &'z' ); assert!(  re.is_match() );
	re.feed( &'c' ); assert!( !re.is_match() );
	re.feed( &'y' ); assert!( !re.is_match() );

	let mut re = RegExRoot::new( atom(|e| *e > 0) * rep(atom(|e| *e == 0)) * atom(|e| *e < 0) );
	assert!( !re.is_match() );
	re.feed( & 1 ); assert!( !re.is_match() );
	re.feed( &-1 ); assert!(  re.is_match() );

	let mut re = RegExRoot::new( atom(|e| *e % 2 != 0) * atom(|e| *e % 2 == 0) + rep(atom(|e| *e > 0)) );
	assert!( re.is_match() );
	re.feed( &1 );
	assert!( re.is_match() );
	re.feed( &2 );
	assert!( re.is_match() );
	re.feed( &3 );
	assert!( re.is_match() );
	re.feed( &0 );
	assert!( !re.is_match() );

}
