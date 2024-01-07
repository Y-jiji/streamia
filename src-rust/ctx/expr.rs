pub struct Lambda(Expr);



pub enum Expr {
    // odestream<Part>: ((Time -> Fp) -> (Time -> Fp)) -> (Time -> Fp)
    // odestream<Time>: ((Time -> Fp) -> (Time -> Fp)) -> (Time -> Fp)
    ODEStream { equation: Box<Expr>, init: f64 },
    // partsec: Part -> Fp
    PartSec { x: Box<Expr> },
    // timesec: Time -> Fp
    TimeSec { x: Box<Expr> },
    // apply function (can also be used as let in function)
    Apply { x: Box<Expr>, f: Box<Lambda> },
    // refer to a variable with debrujin index
    ReferInternal { debrujin: usize },
    // refer to an external index
    ReferExternal { slot: usize },
    // comparision and control flow
    Br { c: Box<Expr>, l: Box<Expr>, r: Box<Expr> },
    Eq { l: Box<Expr>, r: Box<Expr> },
    Ltr { l: Box<Expr>, r: Box<Expr> },
    Gtr { l: Box<Expr>, r: Box<Expr> },
    Lte { l: Box<Expr>, r: Box<Expr> },
    Gte { l: Box<Expr>, r: Box<Expr> },
    // enum type and match
    LInto { x: Box<Expr> },
    RInto { x: Box<Expr> },
    Case { x: Box<Expr>, l: Box<Lambda>, r: Box<Lambda> },
    // product type and projection
    Pair { l: Box<Expr>, r: Box<Expr> },
    LProj { x: Box<Expr> },
    RProj { y: Box<Expr> },
    // float operation
    FAdd { l: Box<Expr>, r: Box<Expr> },
    FSub { l: Box<Expr>, r: Box<Expr> },
    FMul { l: Box<Expr>, r: Box<Expr> },
    FDiv { l: Box<Expr>, r: Box<Expr> },
    FPow { l: Box<Expr>, r: Box<Expr> },
    // constant knob controlled externally
    Knob { value: f64 },
    // part time
    Part { start: f64, end: f64 },
}

pub enum Ty {
    Fn(Box<(Ty, Ty)>),
    Fp,
    Time,
    Part { start: f64, end: f64 }, // Time's subtype
    Null,
    Bool,
    Prod(Box<(Ty, Ty)>),
    Enum(Box<(Ty, Ty)>),
}