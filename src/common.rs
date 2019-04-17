/// Flag indicating whether to store data in Big-endian or Little-endian format.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Endian {
    Big,
    Little,
}
