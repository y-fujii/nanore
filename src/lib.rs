// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;


struct Path( usize, usize, Option<rc::Rc<Path>> );

#[derive( Clone )]
pub struct State( isize, Option<rc::Rc<Path>> );

pub enum RegEx<T> {
	Atom( Box<Fn( &T ) -> bool> ),
	Alt( Box<RegEx<T>>, Box<RegEx<T>> ),
	Seq( Box<RegEx<T>>, Box<RegEx<T>>, State ),
	Repeat( Box<RegEx<T>>, State ),
	Option( Box<RegEx<T>> ),
	Weight( isize ),
	Mark( usize ),
}

impl<T> ops::Add for Box<RegEx<T>> {
	type Output = Box<RegEx<T>>;

	fn add( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Alt( self, other ) )
	}
}

impl<T> ops::Mul for Box<RegEx<T>> {
	type Output = Box<RegEx<T>>;

	fn mul( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Seq( self, other, State( isize::MAX, None ) ) )
	}
}

pub fn atom<T, F: 'static + Fn( &T ) -> bool>( f: F ) -> Box<RegEx<T>> {
	Box::new( RegEx::Atom( Box::new( f ) ) )
}

pub fn val<T: 'static + PartialEq>( v0: T ) -> Box<RegEx<T>> {
	atom( move |v| *v == v0 )
}

pub fn any<T>() -> Box<RegEx<T>> {
	atom( move |_| true )
}

pub fn rep<T>( e0: Box<RegEx<T>> ) -> Box<RegEx<T>> {
	Box::new( RegEx::Repeat( e0, State( isize::MAX, None ) ) )
}

pub fn opt<T>( e0: Box<RegEx<T>> ) -> Box<RegEx<T>> {
	Box::new( RegEx::Option( e0 ) )
}

pub fn weight<T>( i: isize ) -> Box<RegEx<T>> {
	Box::new( RegEx::Weight( i ) )
}

pub fn mark<T>( i: usize ) -> Box<RegEx<T>> {
	Box::new( RegEx::Mark( i ) )
}

fn choice( s0: &State, s1: &State ) -> State {
	(if s0.0 < s1.0 { s0 } else { s1 }).clone()
}

// handle epsilon transition.
fn propagate<T>( e: &mut RegEx<T>, s0: &State, index: usize ) -> State {
	match *e {
		RegEx::Atom( _ ) => {
			State( isize::MAX, None )
		}
		RegEx::Alt( ref mut e0, ref mut e1 ) => {
			choice( &propagate( e0, s0, index ), &propagate( e1, s0, index ) )
		}
		RegEx::Seq( ref mut e0, ref mut e1, ref mut s ) => {
			*s = choice( s, &propagate( e0, s0, index ) );
			propagate( e1, s, index )
		}
		RegEx::Repeat( ref mut e0, ref mut s ) => {
			*s = choice( s, s0 );
			*s = choice( s, &propagate( e0, s, index ) );
			s.clone()
		}
		RegEx::Option( ref mut e0 ) => {
			choice( s0, &propagate( e0, s0, index ) )
		}
		RegEx::Weight( i ) => {
			State( s0.0 + i, s0.1.clone() )
		}
		RegEx::Mark( i ) => {
			State( s0.0, Some( rc::Rc::new( Path( index, i, s0.1.clone() ) ) ) )
		}
	}
}

// handle normal transition.
fn shift<T>( e: &mut RegEx<T>, v: &T, s0: &State ) -> State {
	match *e {
		RegEx::Atom( ref f ) => {
			if s0.0 != isize::MAX && f( v ) { s0.clone() } else { State( isize::MAX, None ) }
		}
		RegEx::Alt( ref mut e0, ref mut e1 ) => {
			choice( &shift( e0, v, s0 ), &shift( e1, v, s0 ) )
		}
		RegEx::Seq( ref mut e0, ref mut e1, ref mut s ) => {
			shift( e1, v, &mem::replace( s, shift( e0, v, s0 ) ) )
		}
		RegEx::Repeat( ref mut e0, ref mut s ) => {
			*s = shift( e0, v, s );
			State( isize::MAX, None )
		}
		RegEx::Option( ref mut e0 ) => {
			shift( e0, v, s0 )
		}
		RegEx::Weight( _ ) => {
			State( isize::MAX, None )
		}
		RegEx::Mark( _ ) => {
			State( isize::MAX, None )
		}
	}
}

pub struct RegExRoot<T> {
	regex: Box<RegEx<T>>,
	s0: State,
	s1: State,
	index: usize,
}

impl<T> RegExRoot<T> {
	pub fn new( mut e: Box<RegEx<T>> ) -> RegExRoot<T> {
		let s1 = propagate( &mut e, &State( 0, None ), 0 );
		RegExRoot{
			regex: e,
			s0: State( 0, None ),
			s1: s1,
			index: 0,
		}
	}

	pub fn feed( &mut self, v: &T ) {
		self.index += 1;
		self.s1 = shift( &mut self.regex, v, &self.s0 );
		self.s0 = State( isize::MAX, None );
		self.s1 = choice( &self.s1, &propagate( &mut self.regex, &self.s0, self.index ) );
	}

	pub fn is_match( &self ) -> bool {
		self.s1.0 != isize::MAX
	}

	pub fn path( &self ) -> Vec<(usize, usize)> {
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
