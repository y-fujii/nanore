# nanore

nanore is a tiny (~0.1K SLOC), O(MN) regular expression matcher for arbitrary data types written in Rust.

## Example

```
use nanore::*;

let mut re = RegExRoot::new(
    atom(|e| *e % 2 != 0) * atom(|e| *e % 2 == 0) + rep(atom(|e| *e > 0))
);

assert!(re.is_match());
re.feed(&1);
assert!(re.is_match());
re.feed(&2);
assert!(re.is_match());
re.feed(&3);
assert!(re.is_match());
re.feed(&0);
assert!(!re.is_match());
```
