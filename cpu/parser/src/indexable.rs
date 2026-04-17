pub trait Indexable {
    /// since ids in xml start at 1, we have to sub 1 to get an array index
    fn to_index(self) -> usize;
    /// since ids in xml start at 1, we have to add 1 to id from an index
    fn from_index(idx: usize) -> Self;
}
