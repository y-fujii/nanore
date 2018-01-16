// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::*;


struct Path<T: Copy>( usize, T, Option<rc::Rc<Path<T>>> );

#[derive( Clone )]
pub struct State<T: Copy>( isize, Option<rc::Rc<Path<T>>> );

pub enum RegEx<T, U: Copy = ()> {
	Atom( Box<Fn( &T ) -> bool> ),
	Alt( Box<RegEx<T, U>>, Box<RegEx<T, U>> ),
	Seq( Box<RegEx<T, U>>, Box<RegEx<T, U>>, State<U> ),
	Repeat( Box<RegEx<T, U>>, State<U> ),
	Option( Box<RegEx<T, U>> ),
	Weight( isize ),
	Mark( U ),
}

impl<T, U: Copy> ops::Add for Box<RegEx<T, U>> {
	type Output = Box<RegEx<T, U>>;

	fn add( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Alt( self, other ) )
	}
}

impl<T, U: Copy> ops::Mul for Box<RegEx<T, U>> {
	type Output = Box<RegEx<T, U>>;

	fn mul( self, other: Self ) -> Self::Output {
		Box::new( RegEx::Seq( self, other, State( isize::MAX, None ) ) )
	}
}

pub fn atom<T, U: Copy, F: 'static + Fn( &T ) -> bool>( f: F ) -> Box<RegEx<T, U>> {
	Box::new( RegEx::Atom( Box::new( f ) ) )
}

pub fn val<T: 'static + PartialEq, U: Copy>( v0: T ) -> Box<RegEx<T, U>> {
	atom( move |v| *v == v0 )
}

pub fn any<T, U: Copy>() -> Box<RegEx<T, U>> {
	atom( move |_| true )
}

pub fn rep<T, U: Copy>( e0: Box<RegEx<T, U>> ) -> Box<RegEx<T, U>> {
	Box::new( RegEx::Repeat( e0, State( isize::MAX, None ) ) )
}

pub fn opt<T, U: Copy>( e0: Box<RegEx<T, U>> ) -> Box<RegEx<T, U>> {
	Box::new( RegEx::Option( e0 ) )
}

pub fn weight<T, U: Copy>( w: isize ) -> Box<RegEx<T, U>> {
	Box::new( RegEx::Weight( w ) )
}

pub fn mark<T, U: Copy>( m: U ) -> Box<RegEx<T, U>> {
	Box::new( RegEx::Mark( m ) )
}

fn choice<T: Copy>( s0: &State<T>, s1: &State<T> ) -> State<T> {
	(if s0.0 < s1.0 { s0 } else { s1 }).clone()
}

// handle epsilon transition.
fn propagate<T, U: Copy>( e: &mut RegEx<T, U>, s0: &State<U>, index: usize ) -> State<U> {
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
		RegEx::Weight( w ) => {
			State( s0.0 + w, s0.1.clone() )
		}
		RegEx::Mark( m ) => {
			State( s0.0, Some( rc::Rc::new( Path( index, m, s0.1.clone() ) ) ) )
		}
	}
}

// handle normal transition.
fn shift<T, U: Copy>( e: &mut RegEx<T, U>, v: &T, s0: &State<U> ) -> State<U> {
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

pub struct RegExRoot<T, U: Copy = ()> {
	regex: Box<RegEx<T, U>>,
	s0: State<U>,
	s1: State<U>,
	index: usize,
}

impl<T, U: Copy> RegExRoot<T, U> {
	pub fn new( mut e: Box<RegEx<T, U>> ) -> RegExRoot<T, U> {
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
