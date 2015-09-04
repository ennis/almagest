use std::cell::RefCell;

struct Inner;
struct A
{
    i: Inner
}

struct Buf<'a> {
    inner: &'a Inner
}

impl<'a> Drop for Buf<'a>
{
	fn drop(&mut self) 
	{
	}
}

struct B<'a>
{
    b: RefCell<Buf<'a>>
}

impl A {
 fn new() -> A
 {
    A { i: Inner }
 }
 
 fn make_b<'a>(&'a self) -> B<'a>
 {
    B::new(&self.i)
 }
}

impl<'a> B<'a>
{
    pub fn new(inner: &'a Inner) -> B<'a>
	{
		B { b: RefCell::new(Buf{inner: inner}) }
	}
}

fn test<'a>(refb: &'a B<'a>)
{
}

fn main()
{
    let a = A::new();
    let mut b = a.make_b();
    test(&b);
}