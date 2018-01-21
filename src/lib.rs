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

#[derive( Clone )]
pub struct Matcher<'a, T: 'a, U: 'a + Copy = ()> {
	root: &'a RegExRoot<'a, T, U>,
	index: usize,
	s0: State<U>,
	states: Vec<State<U>>,
	s1: State<U>,
}

impl<'a, T, U: Copy> Matcher<'a, T, U> {
	pub fn new( root: &'a RegExRoot<'a, T, U> ) -> Matcher<'a, T, U> {
		let mut this = Matcher{
			root: root,
			index: 0,
			s0: State( 0, None ),
			states: vec![ State( isize::MAX, None ); root.nstate ],
			s1: State( isize::MAX, None ),
		};
		this.s1 = this.propagate( &root.regex, State( 0, None ) );
		this
	}

	pub fn feed( &mut self, v: &T ) {
		self.index += 1;
		let s0 = mem::replace( &mut self.s0, State( isize::MAX, None ) );
		let s1 = self.shift( &self.root.regex, v, s0 );
		let s2 = self.propagate( &self.root.regex, State( isize::MAX, None ) );
		self.s1 = Self::choice( s1, s2 );
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

	fn choice( s0: State<U>, s1: State<U> ) -> State<U> {
		if s0.0 < s1.0 { s0 } else { s1 }
	}

	fn choice_inplace( s0: &mut State<U>, s1: State<U> ) {
		if s0.0 > s1.0 {
			*s0 = s1;
		}
	}

	// handle epsilon transition.
	fn propagate( &mut self, e: &RegEx<T, U>, s0: State<U> ) -> State<U> {
		match *e {
			RegEx::Atom( _ ) => {
				State( isize::MAX, None )
			}
			RegEx::Alt( ref e0, ref e1 ) => {
				let s1 = self.propagate( e0, s0.clone() );
				let s2 = self.propagate( e1, s0 );
				Self::choice( s1, s2 )
			}
			RegEx::Seq( ref e0, ref e1, s ) => {
				let s1 = self.propagate( e0, s0 );
				Self::choice_inplace( &mut self.states[s], s1 );
				let s2 = self.states[s].clone();
				self.propagate( e1, s2 )
			}
			RegEx::Repeat( ref e0, s ) => {
				Self::choice_inplace( &mut self.states[s], s0 );
				let s1 = self.states[s].clone();
				let s2 = self.propagate( e0, s1 );
				Self::choice_inplace( &mut self.states[s], s2 );
				self.states[s].clone()
			}
			RegEx::Option( ref e0 ) => {
				let s1 = self.propagate( e0, s0.clone() );
				Self::choice( s0, s1 )
			}
			RegEx::Weight( w ) => {
				let dw = if s0.0 != isize::MAX { w } else { 0 };
				State( s0.0 + dw, s0.1 )
			}
			RegEx::Mark( m ) => {
				State( s0.0, Some( rc::Rc::new( Path( self.index, m, s0.1 ) ) ) )
			}
		}
	}

	// handle normal transition.
	fn shift( &mut self, e: &RegEx<T, U>, v: &T, s0: State<U> ) -> State<U> {
		match *e {
			RegEx::Atom( ref f ) => {
				if s0.0 != isize::MAX && f( v ) { s0 } else { State( isize::MAX, None ) }
			}
			RegEx::Alt( ref e0, ref e1 ) => {
				let s1 = self.shift( e0, v, s0.clone() );
				let s2 = self.shift( e1, v, s0 );
				Self::choice( s1, s2 )
			}
			RegEx::Seq( ref e0, ref e1, s ) => {
				let s1 = self.shift( e0, v, s0 );
				let s2 = mem::replace( &mut self.states[s], s1 );
				self.shift( e1, v, s2 )
			}
			RegEx::Repeat( ref e0, s ) => {
				let s1 = mem::replace( &mut self.states[s], State( isize::MAX, None ) );
				self.states[s] = self.shift( e0, v, s1 );
				State( isize::MAX, None )
			}
			RegEx::Option( ref e0 ) => {
				self.shift( e0, v, s0 )
			}
			RegEx::Weight( _ ) => State( isize::MAX, None ),
			RegEx::Mark( _ )   => State( isize::MAX, None ),
		}
	}
}
