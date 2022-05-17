pub trait AssertFrom<From>: Sized {
    fn assert_from(value: From) -> Self;
}

impl<T, From> AssertFrom<From> for T
where T: TryFrom<From>,
      <T as TryFrom<From>>::Error: std::fmt::Debug
{
    fn assert_from(value: From) -> Self {
        Self::try_from(value).expect("try from")
    }
}
