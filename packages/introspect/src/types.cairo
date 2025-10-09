/// This file contains the implementation of the `Introspect` trait.
///
/// The introspection is used to get the size and layout of a type.
/// It is important to note that signed integers in Cairo are always using `252` bits.

/// Note that for `Array` and `FixedArray` we can't directly use `Ty` as it will cause infinite
/// recursion, so we decided to use a Span with one item only.
/// Note also that, now, Torii uses this `Span` for specific processing on its side, so it cannot be
/// changed directly by a Box<Ty>.
#[derive(Copy, Drop, Serde, Debug, PartialEq)]
pub enum Ty {
    Primitive: felt252,
    Struct: Struct,
    Enum: Enum,
    Tuple: Span<Ty>,
    Array: Span<Ty>,
    ByteArray,
    FixedArray: (Span<Ty>, u32),
}

#[derive(Copy, Drop, Serde, Debug, PartialEq)]
pub struct Struct {
    pub name: felt252,
    pub attrs: Span<felt252>,
    pub children: Span<Member>,
}

#[derive(Copy, Drop, Serde, Debug, PartialEq)]
pub struct Enum {
    pub name: felt252,
    pub attrs: Span<felt252>,
    pub children: Span<(felt252, Ty)>,
}

#[derive(Copy, Drop, Serde, Debug, PartialEq)]
pub struct Member {
    pub name: felt252,
    pub attrs: Span<felt252>,
    pub ty: Ty,
}
