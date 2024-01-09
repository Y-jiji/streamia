use std::{sync::Arc, collections::HashMap};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Expr {
    id: u32,
    ty: Ty,
    inner: Primitive,
}

pub enum TyError {
    Mismatch { expr: Box<Expr>, declared: Ty, synthesized: Ty },
    OverflowVariableList { expr: Box<Expr>, declared: Ty },
}

impl Expr {
    pub fn check(&self, ctx: &Ctx, local: &mut Vec<Ty>) -> Result<(), TyError> {
        use Primitive::*;
        match &self.inner {
            FAdd { l, r } |
            FSub { l, r } |
            FDiv { l, r } |
            FMul { l, r } |
            FPow { l, r } => {
                l.check(ctx, local)?; r.check(ctx, local)?;
                if self.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                if l.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: l.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                if r.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: r.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                Ok(())
            },
            FVal { .. } => {
                if self.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                Ok(())
            },
            Pair { l, r } => {
                l.check(ctx, local)?; r.check(ctx, local)?;
                let synthesized = Ty::Prod(Box::new((l.ty.clone(), r.ty.clone())));
                if self.ty != synthesized {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized
                    })?
                }
                Ok(())
            }
            LProj { x } => {
                x.check(ctx, local)?;
                let Ty::Prod(internal) = &x.ty else {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: x.ty.clone(), 
                        synthesized: Ty::Prod(Box::new((self.ty.clone(), Ty::Auto)))
                    })?
                };
                if internal.as_ref().0 != self.ty {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: internal.as_ref().0.clone(), 
                    })?
                }
                Ok(())
            }
            RProj { x } => {
                x.check(ctx, local)?;
                let Ty::Prod(internal) = &x.ty else {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: x.ty.clone(), 
                        synthesized: Ty::Prod(Box::new((Ty::Auto, self.ty.clone())))
                    })?
                };
                if internal.as_ref().1 != self.ty {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: internal.as_ref().1.clone(), 
                    })?
                }
                Ok(())
            }
            FLtr { l, r } |
            FGtr { l, r } |
            FLte { l, r } |
            FGte { l, r } |
            FEq { l, r } => {
                l.check(ctx, local)?; r.check(ctx, local)?;
                if self.ty != Ty::Bool {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: Ty::Bool,
                    })?
                }
                if l.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: l.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                if r.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: r.ty.clone(), 
                        synthesized: Ty::Fp,
                    })?
                }
                Ok(())
            }
            Br { c, l, r } => {
                c.check(ctx, local)?; l.check(ctx, local)?; r.check(ctx, local)?;
                if l.ty != self.ty {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: l.ty.clone(), 
                        synthesized: self.ty.clone(),
                    })?
                }
                if r.ty != self.ty {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: r.ty.clone(), 
                        synthesized: self.ty.clone(),
                    })?
                }
                if c.ty != Ty::Bool {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: c.ty.clone(), 
                        synthesized: Ty::Bool,
                    })?
                }
                Ok(())
            }
            ReferExternal { slot } => {
                ctx.cls[*slot].check(ctx, local)?;
                if self.ty != ctx.cls[*slot].ty {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: ctx.cls[*slot].ty.clone(),
                    })?
                }
                Ok(())
            }
            ReferInternal { debrujin } => {
                if local.len() <= *debrujin {
                    Err(TyError::OverflowVariableList {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                    })?
                }
                if self.ty != local[local.len() - 1 - debrujin] {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: local[local.len() - 1 - debrujin].clone(),
                    })?
                }
                Ok(())
            }
            Function { x } => {
                match &self.ty {
                    Ty::Fn(ref my) => {
                        local.push(my.0.clone());
                        x.check(ctx, local)?;
                        local.pop();
                        if x.ty != my.1 {
                            Err(TyError::Mismatch {
                                expr: Box::new(self.clone()), 
                                declared: self.ty.clone(), 
                                synthesized: Ty::Fn(Box::new((Ty::Auto, x.ty.clone()))) 
                            })?
                        }
                        else {
                            Ok(())
                        }
                    }
                    _ => {
                        Err(TyError::Mismatch {
                            expr: Box::new(self.clone()), 
                            declared: self.ty.clone(), 
                            synthesized: Ty::Fn(Box::new((Ty::Auto, Ty::Auto)))
                        })
                    }
                }
            }
            Apply { x, f } => {
                x.check(ctx, local)?; 
                f.check(ctx, local)?;
                match &f.ty {
                    Ty::Fn(ref ft) => {
                        if x.ty != ft.0 {
                            Err(TyError::Mismatch {
                                expr: x.clone(), 
                                declared: x.ty.clone(), 
                                synthesized: ft.0.clone() 
                            })?
                        }
                    }
                    _ => {
                        Err(TyError::Mismatch {
                            expr: Box::new(self.clone()), 
                            declared: self.ty.clone(), 
                            synthesized: Ty::Fn(Box::new((x.ty.clone(), Ty::Auto))) 
                        })?
                    }
                }
                Ok(())
            }
            TimeSec { x } => {
                x.check(ctx, local)?;
                if !matches!(x.ty, Ty::Time | Ty::Part) {
                    Err(TyError::Mismatch {
                        expr: x.clone(), 
                        declared: x.ty.clone(), 
                        synthesized: Ty::Time 
                    })?
                }
                if self.ty != Ty::Fp {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: Ty::Fp 
                    })?
                }
                Ok(())
            }
            PartRel { x } |
            PartAbs { x } => {
                x.check(ctx, local)?;
                if !matches!(x.ty, Ty::Part { .. }) {
                    Err(TyError::Mismatch {
                        expr: x.clone(), 
                        declared: x.ty.clone(), 
                        synthesized: Ty::Part
                    })?
                }
                if self.ty != Ty::Time {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized: Ty::Time
                    })?
                }
                Ok(())
            }
            TimeODE { transform, init } => {
                let args = init.len();
                local.push(Ty::Time);
                local.extend([Ty::Fp].into_iter().cycle().take(args));
                for x in transform { x.check(ctx, local)?; }
                local.resize_with(local.len() - args, || panic!());
                local.pop();
                for x in transform {
                    if x.ty != Ty::Fp {
                        Err(TyError::Mismatch { expr: Box::new(x.clone()), declared: x.ty.clone(), synthesized: Ty::Fp })?  
                    }
                }
                let synthesized = Ty::Fn(Box::new((Ty::Time, Ty::dup(Ty::Fp, args))));
                if self.ty != synthesized {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized
                    })?
                }
                Ok(())
            }
            TimeZip { x, secs, default } => {
                x.check(ctx, local)?;
                default.check(ctx, local)?;
                let synthesized = Ty::Fn(Box::new((Ty::Time, Ty::dup(Ty::Fp, secs.len()))));
                if self.ty != synthesized {
                    Err(TyError::Mismatch {
                        expr: Box::new(self.clone()), 
                        declared: self.ty.clone(), 
                        synthesized,
                    })?
                }
                if x.ty != Ty::Fn(Box::new((Ty::Time, Ty::Fp))) {
                    Err(TyError::Mismatch {
                        expr: x.clone(), 
                        declared: x.ty.clone(), 
                        synthesized: Ty::Fn(Box::new((Ty::Time, Ty::Fp)))
                    })?
                }
                Ok(())
            }
            // in case we want to add more variants
            #[allow(unreachable_patterns)] _ => { todo!() }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Primitive {
    // timezip: (Time -> Fp) -> (Time -> (Fp, (Fp, ...)))
    TimeZip { x: Box<Expr>, secs: Vec<f64>, default: Box<Expr> },
    // timeode: (Time -> (Fp, (Fp, ...)) -> (Fp, (Fp, ...))) -> (Time -> (Fp, (Fp, ...)))
    TimeODE { transform: Vec<Expr>, init: Vec<f64> },
    // partrel: Part -> Time
    PartRel { x: Box<Expr> },
    // partabs: Part -> Time
    PartAbs { x: Box<Expr> },
    // timesec: Time -> Fp
    TimeSec { x: Box<Expr> },
    // apply function (can also be used as let in function)
    Apply { x: Box<Expr>, f: Box<Expr> },
    // function
    Function { x: Box<Expr> },
    // refer to a variable with debrujin index
    ReferInternal { debrujin: usize },
    // refer to an external index
    ReferExternal { slot: usize },
    // comparision and control flow
    Br { c: Box<Expr>, l: Box<Expr>, r: Box<Expr> },
    FEq { l: Box<Expr>, r: Box<Expr> },
    FLtr { l: Box<Expr>, r: Box<Expr> },
    FGtr { l: Box<Expr>, r: Box<Expr> },
    FLte { l: Box<Expr>, r: Box<Expr> },
    FGte { l: Box<Expr>, r: Box<Expr> },
    // product type and projection
    Pair { l: Box<Expr>, r: Box<Expr> },
    LProj { x: Box<Expr> },
    RProj { x: Box<Expr> },
    // float operation
    FAdd { l: Box<Expr>, r: Box<Expr> },
    FSub { l: Box<Expr>, r: Box<Expr> },
    FMul { l: Box<Expr>, r: Box<Expr> },
    FDiv { l: Box<Expr>, r: Box<Expr> },
    FPow { l: Box<Expr>, r: Box<Expr> },
    // constant controlled externally
    FVal { muted: bool, value: f64 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Ty {
    Fn(Box<(Ty, Ty)>),
    Fp,
    Time,
    Part, // Time's subtype
    Bool,
    Prod(Box<(Ty, Ty)>),
    Enum(Box<(Ty, Ty)>),
    Auto, // automatically inferred
}

impl Ty {
    pub fn dup(ty: Ty, n: usize) -> Ty {
        assert!(n >= 1);
        if n == 1 { ty }
        else {
            Ty::Prod(Box::new((ty.clone(), Ty::dup(ty, n-1))))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ctx {
    // closed expressions to be evaluated with time inputs
    cls: Vec<Arc<Expr>>,
    // a tag of each expression
    tag: Vec<String>,
    // current id
    idcnt: u64,
    // from id to expression index
    idmap: HashMap<u64, usize>,
}

impl Ctx {
}