use ::std::{cell::Cell, fmt::Display};

pub struct FmtOneshot<F>(Cell<Option<F>>);

impl<F> FmtOneshot<F>
where
    F: for<'a> FnMut(&mut ::std::fmt::Formatter<'a>) -> ::std::fmt::Result,
{
    pub fn new(f: F) -> Self {
        Self(Cell::new(Option::Some(f)))
    }
}

impl<F> Display for FmtOneshot<F>
where
    F: for<'a> FnMut(&mut ::std::fmt::Formatter<'a>) -> ::std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut fmt = self.0.take().unwrap();
        let res = fmt(f);
        self.0.set(Some(fmt));
        res
    }
}

impl<F> ::std::fmt::Debug for FmtOneshot<F>
where
    F: for<'a> FnMut(&mut ::std::fmt::Formatter<'a>) -> ::std::fmt::Result,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
