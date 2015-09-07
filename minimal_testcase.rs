use std::cell::RefCell;

struct Inner;

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

fn test(refb: &B)
{
}

fn main()
{
    let i = Inner;
    let mut b = B {b : RefCell::new(Buf{inner: &i})};
    test(&b);
}