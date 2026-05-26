

use std::rc::Rc;

// locals and globals are all references
// dest, src, [src]

#[derive(Debug)]
pub enum Op<ID> {

    Jump(ID),
    BranchNotZero(ID, ID),

    // leaves address on ret
    AllocateData(usize),
    // leaves address on ret
    Coroutine(ID, Vec<ID>),
    Resume(ID),
    Yield(ID),

    // usize here is offset
    SetData(ID, usize, Vec<u8>),
    // ref, offset, ref, offset, length
    CopyData(ID, usize, ID, usize, usize),

    ReturnLocal(ID), 
    SetLocalFromReturn(ID),
    SetLocalFromLocal(ID, ID),
    SetLocalFromGlobal(ID, ID),
    SetLocalFromProc(ID, ID),
    SetGlobalFromLocal(ID, ID),

    Call(ID, Vec<ID>),
    DynCall(ID, Vec<ID>),

    FAdd(ID, ID, ID),
    FSub(ID, ID, ID),
    FMul(ID, ID, ID),
    FDiv(ID, ID, ID),
    FExp(ID, ID, ID),
    FNeg(ID, ID),

    FEq(ID, ID, ID),
    FGt(ID, ID, ID),
    FLt(ID, ID, ID),

    IAdd(ID, ID, ID),
    ISub(ID, ID, ID),
    IMul(ID, ID, ID),
    IDiv(ID, ID, ID),
    IMod(ID, ID, ID),
    IExp(ID, ID, ID),
    INeg(ID, ID),

    IEq(ID, ID, ID),
    IGt(ID, ID, ID),
    ILt(ID, ID, ID),

    LNot(ID, ID),
    LAnd(ID, ID, ID),
    LOr(ID, ID, ID),
    LXor(ID, ID, ID),

    BNot(ID, ID),
    BAnd(ID, ID, ID),
    BOr(ID, ID, ID),
    BXor(ID, ID, ID),

    Nop,
}

#[derive(Debug)]
pub struct Proc { 
    pub name : Rc<str>,
    pub instrs : Vec<Op<Rc<str>>>,
}

#[derive(Debug)]
pub struct CompiledProc { 
    pub name : Rc<str>,
    pub (crate) instrs : Vec<Op<usize>>,
    pub (crate) slot_names : Vec<Rc<str>>,
    pub (crate) frame_size : usize,
}


pub type StackTrace = Vec<(Rc<str>, usize)>;

#[derive(Debug)]
pub enum VmError {
    UnknownProcId(usize, StackTrace),
    InstrPointerOutOfRange(usize, StackTrace),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f : &mut std::fmt::Formatter) -> std::fmt::Result {
        fn d(x : &StackTrace) -> String {
            x.into_iter().map(|(n, i)| format!("    {} at index {}\n", n, i)).collect()
        }
        match self { 
            VmError::UnknownProcId(id, st) => write!(f, "encountered unknown proc id: {}\n{}", id, d(st)),
            VmError::InstrPointerOutOfRange(ip, st) => write!(f, "encountered instruction pointer past proc length: {}\n{}", ip, d(st)),
        }
    }
}

impl std::error::Error for VmError { }


