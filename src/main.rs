// nock(a)    ~> *a
//
// +0 ~> 1
// +a ~> +a
//
// ?{a b} ~> 1
// ?a     ~> 0
//
// /{1 a}           ~> a
// /{2 {a b}}       ~> a
// /{3 {a b}}       ~> b
// /{(a + a) b}     ~> /{2 /{a b}}
// /{(a + a + 1) b} ~> /{3 /{a b}}
// /a               ~> a
//
// *{a 0 b}   ~> /{b a}
// *{a 1 b}   ~> b
// *{a 2 b c} ~> *{*{a b} *{a c}}
// *{a 3 b}   ~> ?*{a b}
// *{a 4 b}   ~> +*{a b}
// *{a 5 b c} ~> ={*{a b} *{a c}}
// *a         ~> *a

use std::{collections::VecDeque, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
struct Atom(u64);

impl Atom {
  const fn incr(Atom(atom): Self) -> Atom {
    Atom(1 + atom)
  }
}

const ATOM_ADDR: Atom = Atom(0);
const ATOM_IDTY: Atom = Atom(1);
const ATOM_EVAL: Atom = Atom(2);
const ATOM_CELL: Atom = Atom(3);
const ATOM_INCR: Atom = Atom(4);
const ATOM_EQAL: Atom = Atom(5);

thread_local! {
  pub static NOUN_ADDR: Noun = Noun::atom(ATOM_ADDR);
  pub static NOUN_IDTY: Noun = Noun::atom(ATOM_IDTY);
  pub static NOUN_EVAL: Noun = Noun::atom(ATOM_EVAL);
  pub static NOUN_CELL: Noun = Noun::atom(ATOM_CELL);
  pub static NOUN_INCR: Noun = Noun::atom(ATOM_INCR);
  pub static NOUN_EQAL: Noun = Noun::atom(ATOM_EQAL);
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

fn noun_equal(a: Noun, b: Noun) -> bool {
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
    NounInner::Cell(Cell(a, b)) => {
      if let NounInner::Atom(a) = &*a.0 {
        (a, b)
      } else {
        panic!("expected an atom but found {a:?}")
      }
    }
    a => panic!("expected a cell but found {a:?}"),
  };

  match &*inst {
    &ATOM_ADDR => addr(subj, b.clone()),
    &ATOM_IDTY => idty(b.clone()),
    &ATOM_EVAL => eval(subj.clone(), b.clone()),
    &ATOM_INCR => incr(subj.clone(), b.clone()),
    &ATOM_EQAL => eqal(subj.clone(), b.clone()),
    &ATOM_CELL => cell(subj.clone(), b.clone()),
    atom => todo!("atom = {atom:?}"),
  }
}

#[inline(always)]
fn addr(subj: &Noun, addr: Noun) -> Noun {
  let NounInner::Atom(atom) = &*addr.0 else {
    panic!()
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
    _ => todo!(),
  };

  let evaled_b = nock(Noun::cell(subj.clone(), b));
  let evaled_c = nock(Noun::cell(subj, c));

  nock(Noun::cell(evaled_b, evaled_c))
}

#[inline(always)]
fn incr(subj: Noun, form: Noun) -> Noun {
  let prod = nock(Noun::cell(subj, form));
  if let NounInner::Atom(atom) = &*prod.0 {
    Noun::atom(Atom::incr(atom.clone()))
  } else {
    panic!()
  }
}

#[inline(always)]
fn eqal(subj: Noun, form: Noun) -> Noun {
  let (b, c) = match &*form.0 {
    NounInner::Cell(Cell(b, c)) => (b.clone(), c.clone()),
    _ => todo!(),
  };

  let evaled_b = nock(Noun::cell(subj.clone(), b));
  let evaled_c = nock(Noun::cell(subj, c));

  Noun::atom(Atom(if noun_equal(evaled_b, evaled_c) { 1 } else { 0 }))
}

#[inline(always)]
fn cell(subj: Noun, form: Noun) -> Noun {
  let prod = nock(Noun::cell(subj, form));
  Noun::atom(Atom(if prod.is_cell() { 1 } else { 0 }))
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
    Noun::cell(syn!($a), syn!($b))
  };
  (addr) => {
    NOUN_ADDR.with(Clone::clone)
  };
  (idty) => {
    NOUN_IDTY.with(Clone::clone)
  };
  (eval) => {
    NOUN_EVAL.with(Clone::clone)
  };
  (cell) => {
    NOUN_CELL.with(Clone::clone)
  };
  (incr) => {
    NOUN_INCR.with(Clone::clone)
  };
  (eqal) => {
    NOUN_EQAL.with(Clone::clone)
  };
  ($e:tt) => {
    Noun::atom(Atom($e))
  };
}

fn main() {
  let a = syn![
    {{{4, {41, 42}}, 3}, {incr, {addr, 11}}}
  ];
  println!("a = {a:?}");
  let p = nock(a);
  println!("p = {p:?}");
}
