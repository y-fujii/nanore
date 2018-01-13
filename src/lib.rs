// (c) Yasuhiro Fujii <http://mimosa-pudica.net>, under MIT License.
use std::{ mem, ops };


pub enum RegEx<T> {
	Atom( Box<Fn( &T ) -> bool> ),
	Alt( Box<RegEx<T>>, Box<RegEx<T>> ),
	Seq( Box<RegEx<T>>, Box<RegEx<T>>, bool ),
	Repeat( Box<RegEx<T>>, bool ),
	Option( Box<RegEx<T>> ),
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
		Box::new( RegEx::Seq( self, other, false ) )
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
	Box::new( RegEx::Repeat( e0, false ) )
}

pub fn opt<T>( e0: Box<RegEx<T>> ) -> Box<RegEx<T>> {
	Box::new( RegEx::Option( e0 ) )
}

// handle epsilon transition.
fn propagate<T>( e: &mut RegEx<T>, s0: bool ) -> bool {
	match *e {
		RegEx::Atom( _ ) => {
			false
		}
		RegEx::Alt( ref mut e0, ref mut e1 ) => {
			propagate( e0, s0 ) | propagate( e1, s0 )
		}
		RegEx::Seq( ref mut e0, ref mut e1, ref mut s ) => {
			*s |= propagate( e0, s0 );
			propagate( e1, *s )
		}
		RegEx::Repeat( ref mut e0, ref mut s ) => {
			*s |= s0;
			*s |= propagate( e0, *s );
			*s
		}
		RegEx::Option( ref mut e0 ) => {
			s0 | propagate( e0, s0 )
		}
	}
}

// handle normal transition.
fn shift<T>( e: &mut RegEx<T>, v: &T, s0: bool ) -> bool {
	match *e {
		RegEx::Atom( ref f ) => {
			s0 && f( v )
		}
		RegEx::Alt( ref mut e0, ref mut e1 ) => {
			shift( e0, v, s0 ) | shift( e1, v, s0 )
		}
		RegEx::Seq( ref mut e0, ref mut e1, ref mut s ) => {
			shift( e1, v, mem::replace( s, shift( e0, v, s0 ) ) )
		}
		RegEx::Repeat( ref mut e0, ref mut s ) => {
			*s = shift( e0, v, *s );
			false
		}
		RegEx::Option( ref mut e0 ) => {
			shift( e0, v, s0 )
		}
	}
}

pub struct RegExRoot<T> {
	regex: Box<RegEx<T>>,
	s0: bool,
	s1: bool,
}

impl<T> RegExRoot<T> {
	pub fn new( mut e: Box<RegEx<T>> ) -> RegExRoot<T> {
		let s1 = propagate( &mut e, true );
		RegExRoot{
			regex: e,
			s0: true,
			s1: s1,
		}
	}

	pub fn feed( &mut self, v: &T ) {
		self.s1 = shift( &mut self.regex, v, self.s0 ) | propagate( &mut self.regex, self.s0 );
		self.s0 = false;
	}

	pub fn is_match( &self ) -> bool {
		self.s1
	}
}
