// (c) Yasuhiro Fujii <y-fujii at mimosa-pudica.net>, under MIT License.
use std::ops;
use std::cell::Cell;


pub enum RegEx<T> {
	Atom( Box<Fn( &T ) -> bool> ),
	Alt( Box<RegEx<T>>, Box<RegEx<T>> ),
	Seq( Box<RegEx<T>>, Box<RegEx<T>>, Cell<bool> ),
	Repeat( Box<RegEx<T>>, Cell<bool> ),
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
		Box::new( RegEx::Seq( self, other, Cell::new( false ) ) )
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
	Box::new( RegEx::Repeat( e0, Cell::new( false ) ) )
}

pub fn opt<T>( e0: Box<RegEx<T>> ) -> Box<RegEx<T>> {
	Box::new( RegEx::Option( e0 ) )
}

// handle epsilon transition.
fn propagate<T>( e: &RegEx<T>, s0: bool ) -> bool {
	match *e {
		RegEx::Atom( _ ) => {
			false
		}
		RegEx::Alt( ref e0, ref e1 ) => {
			propagate( e0, s0 ) | propagate( e1, s0 )
		}
		RegEx::Seq( ref e0, ref e1, ref s ) => {
			s.set( s.get() | propagate( e0, s0 ) );
			propagate( e1, s.get() )
		}
		RegEx::Repeat( ref e0, ref s ) => {
			s.set( s.get() | s0 );
			s.set( s.get() | propagate( e0, s.get() ) );
			s.get()
		}
		RegEx::Option( ref e0 ) => {
			s0 | propagate( e0, s0 )
		}
	}
}

// handle normal transition.
fn shift<T>( e: &RegEx<T>, v: &T, s0: bool ) -> bool {
	match *e {
		RegEx::Atom( ref f ) => {
			s0 && f( v )
		}
		RegEx::Alt( ref e0, ref e1 ) => {
			shift( e0, v, s0 ) | shift( e1, v, s0 )
		}
		RegEx::Seq( ref e0, ref e1, ref s ) => {
			shift( e1, v, s.replace( shift( e0, v, s0 ) ) )
		}
		RegEx::Repeat( ref e0, ref s ) => {
			s.set( shift( e0, v, s.get() ) );
			false
		}
		RegEx::Option( ref e0 ) => {
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
	pub fn new( e: Box<RegEx<T>> ) -> RegExRoot<T> {
		let s1 = propagate( &e, true );
		RegExRoot{
			regex: e,
			s0: true,
			s1: s1,
		}
	}

	pub fn feed( &mut self, v: &T ) {
		self.s1 = shift( &self.regex, v, self.s0 ) | propagate( &self.regex, self.s0 );
		self.s0 = false;
	}

	pub fn is_match( &self ) -> bool {
		self.s1
	}
}
