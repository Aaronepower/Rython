use std::collections::{HashMap, HashSet};

pub enum Object<'a> {
    None,
    NotImplemented,
    Ellipsis,
    Number(Number),
    Sequence(Sequence<'a>),
    Set(Set<'a>),
    Map(Map<'a>),
    Callable(Callable<'a>),
}

pub enum Number {
    Integral(Integral),
    Real(f64),
}

pub enum Integral {
    Integer(i64),
    Bool(bool),
}

pub enum Sequence<'a> {
    Immutable(ImmutableSequence<'a>),
    Mutable(MutableSequence<'a>),
}

pub enum ImmutableSequence<'a> {
    String(String),
    Tuple(usize, Vec<Object<'a>>),
    Bytes(Vec<u8>),
}

pub enum MutableSequence<'a> {
    List(Vec<Object<'a>>),
    ByteArray(Vec<u8>),
}

pub enum Set<'a> {
    Set(HashSet<Object<'a>>),
    Frozen(HashSet<Object<'a>>),
}

pub enum Map<'a> {
    Dict(HashMap<Object<'a>, Object<'a>>)
}

pub enum Callable<'a> {
    User(&'a str, HashMap<&'a str, Object<'a>>),
}
