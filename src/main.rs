// nock(a)    ~> *a
//
// +0 ~> 1
// +a ~> +a
//
// ?{a b} ~> 0
// ?a     ~> 1
//
// ={a a} ~> 0
// ={a b} ~> 1
//
// /{1 a}           ~> a
// /{2 {a b}}       ~> a
// /{3 {a b}}       ~> b
// /{(a + a) b}     ~> /{2 /{a b}}
// /{(a + a + 1) b} ~> /{3 /{a b}}
// /a               ~> /a
//
// *{a {b c} d}    ~> {*{a b c} *{a d}}
// *{a 0 b}        ~> /{b a}
// *{a 1 b}        ~> b
// *{a 2 b c}      ~> *{*{a b} *{a c}}
// *{a 3 b}        ~> ?*{a b}
// *{a 4 b}        ~> +*{a b}
// *{a 5 b c}      ~> ={*{a b} *{a c}}
// *{a 6 b c d}    ~> *{a *{{c d} 0 *{{2 3} 0 *{a 4 4 b}}}}
// *{a 7 b c}      ~> *{*{a b} c}
// *{a 8 b c}      ~> *{{*{a b} a} c}
// *{a 9 b c}      ~> *{*{a c} 2 {0 1} 0 b}
// *{a 10 {b c} d} ~> #{b *{a c} *{a d}}
// *a              ~> *a

use std::{collections::VecDeque, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct Atom(u64);

impl Atom {
  const fn incr(Atom(atom): Self) -> Atom {
    Atom(1 + atom)
  }
}

pub const YES: u64 = 0;
pub const NAH: u64 = 1;

const ATOM_ADDR: Atom = Atom(0);
const ATOM_IDTY: Atom = Atom(1);
const ATOM_EVAL: Atom = Atom(2);
const ATOM_CELL: Atom = Atom(3);
const ATOM_INCR: Atom = Atom(4);
const ATOM_EQAL: Atom = Atom(5);
const ATOM_BRCH: Atom = Atom(6);
const ATOM_CMPS: Atom = Atom(7);
const ATOM_EXTN: Atom = Atom(8);
const ATOM_INVK: Atom = Atom(9);
const ATOM_RPLC: Atom = Atom(10);
const ATOM_HINT: Atom = Atom(11);

thread_local! {
  pub static NOUN_ADDR: Noun = Noun::atom(ATOM_ADDR);
  pub static NOUN_IDTY: Noun = Noun::atom(ATOM_IDTY);
  pub static NOUN_EVAL: Noun = Noun::atom(ATOM_EVAL);
  pub static NOUN_CELL: Noun = Noun::atom(ATOM_CELL);
  pub static NOUN_INCR: Noun = Noun::atom(ATOM_INCR);
  pub static NOUN_EQAL: Noun = Noun::atom(ATOM_EQAL);
  pub static NOUN_BRCH: Noun = Noun::atom(ATOM_BRCH);
  pub static NOUN_CMPS: Noun = Noun::atom(ATOM_CMPS);
  pub static NOUN_EXTN: Noun = Noun::atom(ATOM_EXTN);
  pub static NOUN_INVK: Noun = Noun::atom(ATOM_INVK);
  pub static NOUN_RPLC: Noun = Noun::atom(ATOM_RPLC);
  pub static NOUN_HINT: Noun = Noun::atom(ATOM_HINT);
}

#[derive(Clone, Debug)]
struct Cell(Noun, Noun);

#[derive(Clone, Debug)]
enum NounInner {
  Atom(Atom),
  Cell(Cell),
}

#[derive(Clone, Debug)]
struct Noun(Rc<NounInner>);

impl Noun {
  pub fn atom(atom: Atom) -> Self {
    Self(Rc::new(NounInner::Atom(atom)))
  }

  pub fn cell(car: Noun, cdr: Noun) -> Self {
    Self(Rc::new(NounInner::Cell(Cell(car, cdr))))
  }

  pub fn is_cell(&self) -> bool {
    matches!(&*self.0, NounInner::Cell(..))
  }
}

fn noun_eq(a: Noun, b: Noun) -> bool {
  if Rc::ptr_eq(&a.0, &b.0) {
    return true;
  }

  let mut deque = VecDeque::new();
  deque.push_back((&*a.0, &*b.0));

  while let Some((a, b)) = deque.pop_front() {
    match (a, b) {
      (NounInner::Atom(a), NounInner::Atom(b)) if a == b => {}
      (NounInner::Cell(a), NounInner::Cell(b)) => {
        deque.push_back((&*a.0.0, &*b.0.0));
        deque.push_back((&*a.1.0, &*b.1.0));
      }
      _ => return false,
    }
  }

  true
}

fn nock(noun: Noun) -> Noun {
  let (subj, form) = match &*noun.0 {
    NounInner::Cell(Cell(a, b)) => (a, b),
    _ => todo!(), // return?
  };
  let (inst, b) = match &*form.0 {
    NounInner::Cell(Cell(inst, b)) => match &*inst.0 {
      NounInner::Atom(inst) => (inst, b),
      NounInner::Cell(Cell(b_, c)) => {
        let d = b;
        let a = Noun::cell(subj.clone(), Noun::cell(b_.clone(), c.clone()));
        let d = Noun::cell(subj.clone(), d.clone());
        return Noun::cell(nock(a), nock(d));
      }
    },
    a => panic!("expected a cell but found {a:?}"),
  };

  match inst {
    &ATOM_ADDR => addr(subj, b.clone()),
    &ATOM_IDTY => idty(b.clone()),
    &ATOM_EVAL => eval(subj.clone(), b.clone()),
    &ATOM_CELL => cell(subj.clone(), b.clone()),
    &ATOM_INCR => incr(subj.clone(), b.clone()),
    &ATOM_EQAL => eqal(subj.clone(), b.clone()),
    &ATOM_BRCH => brch(subj.clone(), b.clone()),
    &ATOM_CMPS => cmps(subj.clone(), b.clone()),
    &ATOM_EXTN => extn(subj.clone(), b.clone()),
    &ATOM_INVK => invk(subj.clone(), b.clone()),
    &ATOM_RPLC => rplc(subj.clone(), b.clone()),
    &ATOM_HINT => todo!("hint"),
    atom => todo!("atom = {atom:?}"),
  }
}

#[inline(always)]
fn addr(subj: &Noun, addr: Noun) -> Noun {
  let NounInner::Atom(atom) = &*addr.0 else {
    panic!("address is not an atom")
  };

  if atom.0 == 0 {
    panic!("address can't be zero")
  }

  // ignore the leading '1' bit
  //
  // 0b100 = go left
  //    ^
  // 0b101 = go right
  //     ^
  fn aux(path: u64, mut subj: &Noun) -> Noun {
    let mut cursor = 64 - path.leading_zeros() - 1;

    loop {
      if cursor == 0 {
        break;
      }

      let NounInner::Cell(Cell(car, cdr)) = &*subj.0 else {
        panic!("expected a cell")
      };

      cursor -= 1;

      let bit = (path & (1 << cursor)) >> cursor;

      if bit == 0 {
        subj = car;
      } else {
        subj = cdr;
      }
    }

    subj.clone()
  }

  aux(atom.0, subj)
}

#[inline(always)]
const fn idty(noun: Noun) -> Noun {
  noun
}

#[inline(always)]
fn eval(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => panic!(),
  };

  let evaled_b = nock(Noun::cell(subj.clone(), b));
  let evaled_c = nock(Noun::cell(subj, c));

  nock(Noun::cell(evaled_b, evaled_c))
}

#[inline(always)]
fn incr(subj: Noun, form: Noun) -> Noun {
  let prod = nock(Noun::cell(subj, form));
  if let NounInner::Atom(atom) = &*prod.0 {
    Noun::atom(Atom::incr(*atom))
  } else {
    panic!()
  }
}

#[inline(always)]
fn eqal(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => panic!(),
  };

  let evaled_b = nock(Noun::cell(subj.clone(), b));
  let evaled_c = nock(Noun::cell(subj, c));

  Noun::atom(Atom(if noun_eq(evaled_b, evaled_c) { 0 } else { 1 }))
}

#[inline(always)]
fn cell(subj: Noun, form: Noun) -> Noun {
  let prod = nock(Noun::cell(subj, form));
  Noun::atom(Atom(if prod.is_cell() { 0 } else { 1 }))
}

#[inline(always)]
fn brch(subj: Noun, form: Noun) -> Noun {
  let NounInner::Cell(Cell(b, cd)) = &*form.0 else {
    panic!()
  };
  let NounInner::Cell(Cell(c, d)) = &*cd.0 else {
    panic!()
  };

  let brch_addr = Noun::cell(Noun::atom(Atom(2)), Noun::atom(Atom(3)));
  let cond = Noun::cell(
    subj.clone(),
    Noun::cell(
      NOUN_INCR.with(Clone::clone),
      Noun::cell(NOUN_INCR.with(Clone::clone), b.clone()),
    ),
  );
  let evaled_cond = nock(cond);
  let addr_ = nock(Noun::cell(
    brch_addr,
    Noun::cell(NOUN_ADDR.with(Clone::clone), evaled_cond),
  ));

  let then_else = Noun::cell(c.clone(), d.clone());
  let form = Noun::cell(then_else, Noun::cell(NOUN_ADDR.with(Clone::clone), addr_));
  let form = nock(form);

  nock(Noun::cell(subj, form))
}

#[inline(always)]
fn cmps(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => panic!(),
  };

  let evaled_b = nock(Noun::cell(subj, b));

  nock(Noun::cell(evaled_b, c))
}

#[inline(always)]
fn extn(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => panic!(),
  };

  let evaled_b = nock(Noun::cell(subj.clone(), b));
  let new_subj = Noun::cell(evaled_b, subj);

  nock(Noun::cell(new_subj, c))
}

#[inline(always)]
fn invk(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => panic!(),
  };

  let core = nock(Noun::cell(subj, c));
  let eval = Noun::cell(
    NOUN_EVAL.with(Clone::clone),
    Noun::cell(
      Noun::cell(NOUN_ADDR.with(Clone::clone), Noun::atom(Atom(1))),
      Noun::cell(NOUN_ADDR.with(Clone::clone), b),
    ),
  );
  nock(Noun::cell(core, eval))
}

#[inline(always)]
fn rplc(subj: Noun, form: Noun) -> Noun {
  let (bc, d) = match &*form.0 {
    NounInner::Cell(Cell(b, d)) => (b, d.clone()),
    _ => panic!(),
  };
  let (b, c, d) = match &*bc.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone(), d),
    _ => panic!(),
  };
  let NounInner::Atom(b) = *b.0 else { panic!() };

  let evaled_c = nock(Noun::cell(subj.clone(), c));
  let evaled_d = nock(Noun::cell(subj, d));

  rplc_at(b.0, evaled_c, &evaled_d)
}

fn rplc_at(path: u64, new_val: Noun, target: &Noun) -> Noun {
  let mut cursor = 64 - path.leading_zeros() - 1;

  let mut stack = vec![];
  let mut current = target;

  loop {
    if cursor == 0 {
      break;
    }

    let NounInner::Cell(Cell(car, cdr)) = &*current.0 else {
      panic!("expected a cell");
    };

    cursor -= 1;

    let bit = (path & (1 << cursor)) >> cursor;

    stack.push((bit, car.clone(), cdr.clone()));

    if bit == 0 {
      current = car;
    } else {
      current = cdr;
    }
  }

  let mut result = new_val;

  while let Some((bit, car, cdr)) = stack.pop() {
    result = if bit == 0 {
      Noun::cell(result, cdr)
    } else {
      Noun::cell(car, result)
    }
  }

  result
}

impl std::fmt::Display for Atom {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl std::fmt::Display for Cell {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{")?;

    let mut first = true;
    let mut current = Some(self);

    while let Some(Cell(car, cdr)) = current {
      if !first {
        write!(f, " ")?;
      }
      write!(f, "{car}")?;

      match &*cdr.0 {
        NounInner::Cell(cell) => current = Some(cell),
        _ => {
          write!(f, " {cdr}}}")?;
          return Ok(());
        }
      }

      first = false;
    }

    write!(f, "}}")
  }
}

impl std::fmt::Display for Noun {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &*self.0 {
      NounInner::Atom(atom) => write!(f, "{atom}"),
      NounInner::Cell(cell) => write!(f, "{cell}"),
    }
  }
}

macro_rules! syn {
  ({ $a:tt, $b:tt }) => {
    crate::Noun::cell(syn!($a), syn!($b))
  };
  (addr) => {
    crate::NOUN_ADDR.with(Clone::clone)
  };
  (idty) => {
    crate::NOUN_IDTY.with(Clone::clone)
  };
  (eval) => {
    crate::NOUN_EVAL.with(Clone::clone)
  };
  (cell) => {
    crate::NOUN_CELL.with(Clone::clone)
  };
  (incr) => {
    crate::NOUN_INCR.with(Clone::clone)
  };
  (eqal) => {
    crate::NOUN_EQAL.with(Clone::clone)
  };
  (brch) => {
    crate::NOUN_BRCH.with(Clone::clone)
  };
  (cmps) => {
    crate::NOUN_CMPS.with(Clone::clone)
  };
  (extn) => {
    crate::NOUN_EXTN.with(Clone::clone)
  };
  (invk) => {
    crate::NOUN_INVK.with(Clone::clone)
  };
  (rplc) => {
    crate::NOUN_RPLC.with(Clone::clone)
  };
  (hint) => {
    crate::NOUN_HINT.with(Clone::clone)
  };
  ($e:expr) => {
    crate::Noun::atom(crate::Atom($e))
  };
}

#[cfg(test)]
mod test {
  use crate::{Atom, Noun, nock, noun_eq, rplc_at};
  use crate::{NAH, YES};

  #[test]
  fn test_addr() {
    let a = syn!({{{{8, 42}, 5}, 2}, {addr, 9}});

    let p = nock(a);
    let e = Noun::atom(Atom(42));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_incr() {
    let a = syn!({40, {incr, {incr, {addr, 1}}}});

    let p = nock(a);
    let e = Noun::atom(Atom(42));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_eval() {
    let a = syn!({41, {eval, {{incr, {addr, 1}}, {idty, {addr, 1}}}}});

    let p = nock(a);
    let e = Noun::atom(Atom(42));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_brch_yes() {
    let a = syn!({YES, {brch, {{addr, 1}, {{idty, 99}, {idty, 42}}}}});

    let p = nock(a);
    let e = Noun::atom(Atom(99));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_brch_nah() {
    let a = syn!({NAH, {brch, {{addr, 1}, {{idty, 99}, {idty, 42}}}}});

    let p = nock(a);
    let e = Noun::atom(Atom(42));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_cmps() {
    // compose is like eval when quoting 'c'
    let a = syn!({41, {cmps, {{incr, {addr, 1}}, {addr, 1}}}});

    let p = nock(a);
    let e = Noun::atom(Atom(42));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_extn() {
    let a = syn!({42, {extn, {{incr, {addr, 1}}, {addr, 1}}}});

    let p = nock(a);
    let e = Noun::cell(Noun::atom(Atom(43)), Noun::atom(Atom(42)));

    assert!(noun_eq(p, e));
  }

  #[test]
  fn test_rplc() {
    let t = syn!({{22, {89, 78}}, 44});
    let r = rplc_at(10, Noun::atom(Atom(55)), &t);
    let e = syn!({{22, {55, 78}}, 44});

    assert!(noun_eq(r, e));
  }

  #[test]
  fn test_decr() {
    // fn(a) {
    //   let mut b = 0;
    //   'trap: loop {
    //     if +b = a {
    //       return b;
    //     } else {
    //       b = +b;
    //       continue 'trap;
    //     }
    //   }
    // }
    //
    // core = [bat pay]
    // where pay = [b a]
    // and bat = loop

    let s = syn!(43);

    let test = syn!({eqal, {{addr, 7}, {incr, {addr, 6}}}});
    let yes = syn!({addr, 6});
    let new_core = syn!({{addr, 2}, {{incr, {addr, 6}}, {addr, 7}}});
    let nah = Noun::cell(syn!(invk), Noun::cell(syn!(2), new_core));
    let r#loop = Noun::cell(syn!(brch), Noun::cell(test, Noun::cell(yes, nah)));
    let r#loop = Noun::cell(syn!(idty), r#loop);
    let g = Noun::cell(
      syn!(extn),
      Noun::cell(
        Noun::cell(syn!(idty), syn!(0)),
        Noun::cell(syn!(extn), Noun::cell(r#loop, syn!({invk, {2, {addr, 1}}}))),
      ),
    );
    let p = nock(Noun::cell(s, g));
    let e = syn!(42);

    assert!(noun_eq(p, e));
  }
}

fn main() {
  todo!()
}
