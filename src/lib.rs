// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;


pub enum RegEx<'a, T, U: Copy = ()> {
	Atom( Box<'a + Fn( &T ) -> bool> ),
	Alt( Box<RegEx<'a, T, U>>, Box<RegEx<'a, T, U>> ),
	Seq( Box<RegEx<'a, T, U>>, Box<RegEx<'a, T, U>>, usize ),
	Repeat( Box<RegEx<'a, T, U>>, usize ),
	Option( Box<RegEx<'a, T, U>> ),
	Weight( isize ),
	Mark( U ),
}

impl<'a, T, U: Copy> ops::Add for Box<RegEx<'a, T, U>> {
	type Output = Box<RegEx<'a, T, U>>;

	fn add( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Alt( self, other ) )
	}
}

impl<'a, T, U: Copy> ops::Mul for Box<RegEx<'a, T, U>> {
	type Output = Box<RegEx<'a, T, U>>;

	fn mul( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Seq( self, other, usize::MAX ) )
	}
}

pub fn atom<'a, T, U: Copy, F: 'a + Fn( &T ) -> bool>( f: F ) -> Box<RegEx<'a, T, U>> {
	Box::new( RegEx::Atom( Box::new( f ) ) )
}

pub fn val<'a, T: 'a + PartialEq, U: Copy>( v0: T ) -> Box<RegEx<'a, T, U>> {
	atom( move |v| *v == v0 )
}

pub fn any<'a, T, U: Copy>() -> Box<RegEx<'a, T, U>> {
	atom( move |_| true )
}

pub fn rep<'a, T, U: Copy>( e0: Box<RegEx<'a, T, U>> ) -> Box<RegEx<'a, T, U>> {
	Box::new( RegEx::Repeat( e0, usize::MAX ) )
}

pub fn opt<'a, T, U: Copy>( e0: Box<RegEx<'a, T, U>> ) -> Box<RegEx<'a, T, U>> {
	Box::new( RegEx::Option( e0 ) )
}

pub fn weight<'a, T, U: Copy>( w: isize ) -> Box<RegEx<'a, T, U>> {
	Box::new( RegEx::Weight( w ) )
}

pub fn mark<'a, T, U: Copy>( m: U ) -> Box<RegEx<'a, T, U>> {
	Box::new( RegEx::Mark( m ) )
}

pub struct RegExRoot<'a, T, U: Copy = ()> {
	regex: Box<RegEx<'a, T, U>>,
	nstate: usize,
}

impl<'a, T, U: Copy> RegExRoot<'a, T, U> {
	pub fn new( mut e: Box<RegEx<'a, T, U>> ) -> RegExRoot<'a, T, U> {
		let n = Self::renumber( &mut e, 0 );
		RegExRoot{
			regex: e,
			nstate: n,
		}
	}

	fn renumber( e: &mut RegEx<T, U>, i: usize ) -> usize {
		match *e {
			RegEx::Atom( _ ) => i,
			RegEx::Alt( ref mut e0, ref mut e1 ) => {
				Self::renumber( e1, Self::renumber( e0, i ) )
			}
			RegEx::Seq( ref mut e0, ref mut e1, ref mut s ) => {
				*s = Self::renumber( e0, i );
				Self::renumber( e1, *s + 1 )
			}
			RegEx::Repeat( ref mut e0, ref mut s ) => {
				*s = i;
				Self::renumber( e0, i + 1 )
			}
			RegEx::Option( ref mut e0 ) => {
				Self::renumber( e0, i )
			}
			RegEx::Weight( _ ) => i,
			RegEx::Mark( _ )   => i,
		}
	}
}

struct Path<T: Copy>( usize, T, Option<rc::Rc<Path<T>>> );

#[derive( Clone )]
struct State<T: Copy>( isize, Option<rc::Rc<Path<T>>> );

pub struct Matcher<'a, T: 'a, U: 'a + Copy = ()> {
	root: &'a RegExRoot<'a, T, U>,
	index: usize,
	s0: State<U>,
	states: Vec<State<U>>,
	s1: State<U>,
}

impl<'a, T, U: Copy> Matcher<'a, T, U> {
	pub fn new( root: &'a RegExRoot<'a, T, U> ) -> Matcher<'a, T, U> {
		let s0 = State( 0, None );
		let mut states = vec![ State( isize::MAX, None ); root.nstate ];
		let s1 = propagate( &mut states, &root.regex, s0.clone(), 0 );
		Matcher{
			root: root,
			index: 0,
			s0: s0,
			states: states,
			s1: s1,
		}
	}

	pub fn feed( &mut self, v: &T ) {
		self.index += 1;
		let s0 = mem::replace( &mut self.s0, State( isize::MAX, None ) );
		let s1 = shift( &mut self.states, &self.root.regex, v, s0 );
		let s2 = propagate( &mut self.states, &self.root.regex, self.s0.clone(), self.index );
		self.s1 = choice( s1, s2 );
	}

	pub fn feed_iter<'b, Iter: IntoIterator<Item = &'b T>>( &mut self, iter: Iter ) where 'a: 'b {
		for v in iter {
			self.feed( v );
		}
	}

	pub fn is_match( &self ) -> bool {
		self.s1.0 != isize::MAX
	}

	pub fn alive( &self ) -> bool {
		self.s0.0 != isize::MAX ||
		self.s1.0 != isize::MAX ||
		self.states.iter().any( |s| s.0 != isize::MAX )
	}

	pub fn path( &self ) -> Vec<(usize, U)> {
		let mut result = Vec::new();
		let mut it = self.s1.1.clone();
		while let Some( e ) = it {
			result.push( (e.0, e.1) );
			it = e.2.clone();
		}
		result.reverse();
		result
	}
}

fn choice<T: Copy>( s0: State<T>, s1: State<T> ) -> State<T> {
	if s0.0 < s1.0 { s0 } else { s1 }
}

fn choice_inplace<T: Copy>( s0: &mut State<T>, s1: State<T> ) {
	if s0.0 > s1.0 {
		*s0 = s1;
	}
}

// handle epsilon transition.
fn propagate<T, U: Copy>( states: &mut [State<U>], e: &RegEx<T, U>, mut s0: State<U>, index: usize ) -> State<U> {
	match *e {
		RegEx::Atom( _ ) => {
			State( isize::MAX, None )
		}
		RegEx::Alt( ref e0, ref e1 ) => {
			let s1 = propagate( states, e0, s0.clone(), index );
			let s2 = propagate( states, e1, s0, index );
			choice( s1, s2 )
		}
		RegEx::Seq( ref e0, ref e1, s ) => {
			let s1 = propagate( states, e0, s0, index );
			choice_inplace( &mut states[s], s1 );
			let s2 = states[s].clone();
			propagate( states, e1, s2, index )
		}
		RegEx::Repeat( ref e0, s ) => {
			choice_inplace( &mut states[s], s0 );
			let s1 = states[s].clone();
			let s2 = propagate( states, e0, s1, index );
			choice_inplace( &mut states[s], s2 );
			states[s].clone()
		}
		RegEx::Option( ref e0 ) => {
			let s1 = propagate( states, e0, s0.clone(), index );
			choice( s0, s1 )
		}
		RegEx::Weight( w ) => {
			if s0.0 != isize::MAX {
				s0.0 += w;
			}
			s0
		}
		RegEx::Mark( m ) => {
			State( s0.0, Some( rc::Rc::new( Path( index, m, s0.1.clone() ) ) ) )
		}
	}
}

// handle normal transition.
fn shift<T, U: Copy>( states: &mut [State<U>], e: &RegEx<T, U>, v: &T, s0: State<U> ) -> State<U> {
	match *e {
		RegEx::Atom( ref f ) => {
			if s0.0 != isize::MAX && f( v ) { s0 } else { State( isize::MAX, None ) }
		}
		RegEx::Alt( ref e0, ref e1 ) => {
			let s1 = shift( states, e0, v, s0.clone() );
			let s2 = shift( states, e1, v, s0 );
			choice( s1, s2 )
		}
		RegEx::Seq( ref e0, ref e1, s ) => {
			let s1 = shift( states, e0, v, s0 );
			let s2 = mem::replace( &mut states[s], s1 );
			shift( states, e1, v, s2 )
		}
		RegEx::Repeat( ref e0, s ) => {
			let s1 = mem::replace( &mut states[s], unsafe { mem::uninitialized() } );
			let s2 = shift( states, e0, v, s1 );
			mem::forget( mem::replace( &mut states[s], s2 ) );
			State( isize::MAX, None )
		}
		RegEx::Option( ref e0 ) => {
			shift( states, e0, v, s0 )
		}
		RegEx::Weight( _ ) => State( isize::MAX, None ),
		RegEx::Mark( _ )   => State( isize::MAX, None ),
	}
}
