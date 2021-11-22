// pub type Trace<const N: usize> = Vec<[bool; N]>;
pub type Trace<'a, const N: usize> = &'a [[bool; N]];
