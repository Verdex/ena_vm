

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
    AllocateCopyFrame, // TODO: also need to grab ip and which proc this even is (also how big is this thing anyway?).  Might want coroutine directly.

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

    IAdd(usize, usize),
    ISub(usize, usize),
    IMul(usize, usize),
    IDiv(usize, usize),
    IMod(usize, usize),
    INeg(usize),

    IEq(usize, usize),
    IGt(usize, usize),
    ILt(usize, usize),

    Not(usize),
    And(usize, usize),
    Or(usize, usize),
    Xor(usize, usize),
    Nop,
}

#[derive(Debug)]
pub struct Proc { 
    pub name : Rc<str>,
    pub instrs : Vec<Op>,
}

