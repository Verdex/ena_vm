

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

    FAdd(usize, usize),
    FSub(usize, usize),
    FMul(usize, usize),
    FDiv(usize, usize),
    FExp(usize, usize),
    FNeg(usize),

    FEq(usize, usize),
    FGt(usize, usize),
    FLt(usize, usize),

    IAdd(usize, usize),
    ISub(usize, usize),
    IMul(usize, usize),
    IDiv(usize, usize),
    IMod(usize, usize),
    IExp(usize, usize),
    INeg(usize),

    IEq(usize, usize),
    IGt(usize, usize),
    ILt(usize, usize),

    LNot(usize),
    LAnd(usize, usize),
    LOr(usize, usize),
    LXor(usize, usize),

    BNot(usize),
    BAnd(usize, usize),
    BOr(usize, usize),
    BXor(usize, usize),

    Nop,
}

#[derive(Debug)]
pub struct Proc { 
    pub name : Rc<str>,
    pub instrs : Vec<Op>,
}

