
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::sync::Arc;

use cranelift::codegen::ir::FuncRef;
use cranelift::prelude::*;
use swc_ecma_ast::*;

use parking_lot::RwLock;

use crate::runtime::Runtime;
use crate::error::Error;
use crate::value::JValue;

pub struct LoopExit{
    label:Option<String>,
    exit_block:Block,
    continue_block:Block,
}

pub struct BuilderContext{

    runtime:Arc<&'static mut Runtime>,
    parent:Option<&'static mut BuilderContext>,

    builder:&'static mut FunctionBuilder<'static>,
    variables:HashSet<u64>,

    loop_exits:Arc<RefCell<Vec<LoopExit>>>,

    is_in_try:bool,
    try_exits:Arc<RefCell<Vec<Block>>>,

    need_capture:Arc<RefCell<Vec<u64>>>,

    vmctx:Variable,
    this:Variable,

    to_bool:FuncRef,
    /// fn(value:JValue) -> !
    throw:FuncRef,

    /// fn(self:JValue, key:JValue) -> JValue
    member:FuncRef,
    /// fn(self:JValue, key:JValue, value:JValue)
    set_memebr:FuncRef,
    /// fn(self:JValue, key:JValue, value:JValue, op:i8)
    assign_member:FuncRef,

    /// fn(callee:JValue, *mut VmContext, this:JValue, argv:*mut JValue, argc:i64, spread:bool) -> (JValue, ok:bool)
    call:FuncRef,

    /// fn(self:JValue, member:Jvalue, *mut VmContext, argv:*mut JValue, argc:i64, spread:bool) -> (JValue, ok:bool)
    memberCall:FuncRef,
    /// fn(self:JValue, member:JValue, *mut VmContext, argv:*mut JValue, argc:i64, spread:bool) -> (Jvalue, ok:bool)
    superMemberCall:FuncRef,

    /// fn(argv:*mut JValue, argc:i64, spread:bool) -> Arc<dyn JObjectInner>
    array_new:FuncRef,

    /// fn(Arc<dyn JObjectInner>) -> JValue
    object_from_inner:FuncRef,
    
    add:FuncRef,
    bitAnd:FuncRef,
    bitOr:FuncRef,
    bitXor:FuncRef,
    div:FuncRef,
    eqeq:FuncRef,
    eqeqeq:FuncRef,
    exp:FuncRef,
    gt:FuncRef,
    gteq:FuncRef,
    In:FuncRef,
    instanceof:FuncRef,
    lshift:FuncRef,
    and:FuncRef,
    or:FuncRef,
    lt:FuncRef,
    lteq:FuncRef,
    Mod:FuncRef,
    mul:FuncRef,
    noteq:FuncRef,
    noteqeq:FuncRef,
    nullishCoalscing:FuncRef,
    rshift:FuncRef,
    sub:FuncRef,
    unsignedRShift:FuncRef,
}

impl BuilderContext{

    pub fn new_context(&mut self) -> Self{
        unsafe{
            return Self{
                runtime:self.runtime.clone(),
                parent:Some(std::mem::transmute_copy(&self)),
                builder:std::mem::transmute_copy(&self.builder),

                variables:Default::default(),
                loop_exits:self.loop_exits.clone(),

                is_in_try:self.is_in_try,
                try_exits:self.try_exits.clone(),

                need_capture:self.need_capture.clone(),

                ..*self
            }
        }
        
    }

    pub fn close(&mut self){

    }

    /// return a B8 value
    pub fn to_bool(&mut self, v:Value) -> Value{
        let ins = self.builder.ins().call(self.to_bool, &[v]);
        self.builder.inst_results(ins)[0]
    }

    pub fn eqeqeq(&mut self, v:Value, v1:Value) -> Value{
        let ins = self.builder.ins().call(self.eqeqeq, &[v, v1]);
        self.builder.inst_results(ins)[0]
    }

    pub fn const_value(&mut self, value:JValue) -> Value{
        todo!()
    }

    pub fn translate_stmt(&mut self, stmt:&Stmt, label:Option<String>) -> Result<(), Error>{
        match stmt{
            Stmt::Block(b) => {
                let mut ctx = self.new_context();
                for i in &b.stmts{
                    ctx.translate_stmt(i, None)?;
                }
                ctx.close();
            },

            Stmt::Break(b) => {
                
                if self.loop_exits.borrow().is_empty(){
                    return Err(Error::IllegalBreakStatement)
                }

                if let Some(l) = &b.label{

                    // lable exist
                    for i in self.loop_exits.borrow().iter(){
                        if let Some(s) = &i.label{
                            if s.as_str() == &l.sym{
                                self.builder.ins().jump(i.exit_block, &[]);
                                return Ok(())
                            }
                        }
                    }

                } else{
                    let e = self.loop_exits.borrow();
                    let exit = e.last().unwrap();
                    self.builder.ins().jump(exit.exit_block, &[]);
                }
            },

            Stmt::Continue(c) => {
                if self.loop_exits.borrow().is_empty(){
                    return Err(Error::IllegalContinueStatment)
                }

                if let Some(l) = &c.label{

                    // lable exist
                    for i in self.loop_exits.borrow().iter(){
                        if let Some(s) = &i.label{
                            if s.as_str() == &l.sym{
                                self.builder.ins().jump(i.continue_block, &[]);
                                return Ok(())
                            }
                        }
                    }

                    // lable does not exist
                    return Err(Error::UndefinedLabel(l.sym.to_string()))

                } else{
                    let e = self.loop_exits.borrow();
                    let exit = e.last().unwrap();
                    self.builder.ins().jump(exit.continue_block, &[]);
                }
            },

            Stmt::Debugger(d) => {
                todo!()
            },

            Stmt::Decl(d) => {
                todo!()
            },

            Stmt::DoWhile(d) => {
                let entry_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.loop_exits.borrow_mut().push(LoopExit { 
                    label: label, 
                    exit_block, 
                    continue_block: entry_block 
                });

                self.builder.ins().jump(entry_block, &[]);
                
                self.builder.switch_to_block(entry_block);

                let mut ctx = self.new_context();
                ctx.translate_stmt(d.body.as_ref(), None)?;
                

                let test_re = ctx.translate_expr(&d.test)?;
                let b = ctx.to_bool(test_re);
                
                ctx.close();
                self.loop_exits.borrow_mut().pop();

                self.builder.ins().brz(b, exit_block, &[]);
                self.builder.ins().jump(entry_block, &[]);

                self.builder.seal_block(entry_block);
                self.builder.seal_block(exit_block);

                self.builder.switch_to_block(exit_block);
            },

            Stmt::Empty(e) => {},
            Stmt::Expr(e) => {
                self.translate_expr(&e.expr)?;
            },
            Stmt::For(f) => {
                let entry_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.loop_exits.borrow_mut().push(LoopExit { 
                    label: label, 
                    exit_block, 
                    continue_block: entry_block 
                });
                
                // init
                if let Some(d) = &f.init{
                    match d{
                        VarDeclOrExpr::Expr(e) => {
                            self.translate_expr(&e)?;
                        },
                        VarDeclOrExpr::VarDecl(d) => {
                            self.translate_var_decl(d)?;
                        },
                    }
                }

                self.builder.ins().jump(entry_block, &[]);
                
                self.builder.switch_to_block(entry_block);

                let mut ctx = self.new_context();

                // test
                if let Some(e) = &f.test{
                    let v = ctx.translate_expr(&e)?;
                    let b = ctx.to_bool(v);
                    ctx.builder.ins().brnz(b, exit_block, &[]);
                }

                ctx.translate_stmt(f.body.as_ref(), None)?;

                if let Some(e) = &f.update{
                    ctx.translate_expr(&e)?;
                }

                ctx.close();
                self.loop_exits.borrow_mut().pop();

                self.builder.ins().jump(entry_block, &[]);

                self.builder.seal_block(entry_block);
                self.builder.seal_block(exit_block);

                self.builder.switch_to_block(exit_block);

            },
            Stmt::ForIn(f) => {
                todo!()
            },
            Stmt::ForOf(f) => {
                todo!()
            },
            Stmt::If(i) => {
                let entry_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                let test_re = self.translate_expr(&i.test)?;
                let b = self.to_bool(test_re);

                self.builder.ins().brnz(b, entry_block, &[]);

                // fall through
                if let Some(alt) = &i.alt{
                    // if there is alternative (else)
                    let mut ctx = self.new_context();
                    ctx.translate_stmt(&alt, None)?;
                    ctx.close();

                } else{
                    // jump to exit directly
                    self.builder.ins().jump(exit_block, &[]);
                }

                self.builder.seal_block(entry_block);
                self.builder.switch_to_block(entry_block);

                let mut ctx = self.new_context();
                ctx.translate_stmt(&i.cons, None)?;
                ctx.close();

                self.builder.ins().jump(exit_block, &[]);

                self.builder.seal_block(exit_block);
                self.builder.switch_to_block(exit_block);
            },

            Stmt::Labeled(l) => {
                return self.translate_stmt(&l.body, Some(l.label.sym.to_string()));
            },

            Stmt::Return(r) => {
                
                let re = if let Some(e) = &r.arg{
                    self.translate_expr(&e)?
                } else{
                    self.builder.ins().iconst(types::I128, 0)
                };
                self.close();
                self.builder.ins().return_(&[re]);
            },

            Stmt::Switch(s) => {
                let discrim = self.translate_expr(&s.discriminant)?;

                let exit_block = self.builder.create_block();

                let mut ctx = self.new_context();

                for cas in &s.cases{
                    if let Some(d) = &cas.test{

                        let val = self.translate_expr(&d)?;
                        let b = self.eqeqeq(discrim, val);
                        let b = self.to_bool(b);

                        let current = self.builder.current_block().unwrap();
                        let case_block = self.builder.create_block();

                        self.builder.ins().brnz(b, case_block, &[]);

                        self.builder.seal_block(case_block);
                        self.builder.switch_to_block(case_block);

                        for s in &cas.cons{
                            ctx.translate_stmt(s, None)?;
                        }
                        self.builder.ins().jump(exit_block, &[]);

                        self.builder.switch_to_block(current);

                    } else{
                        let default_block = self.builder.create_block();

                        self.builder.ins().jump(default_block, &[]);
                        self.builder.seal_block(default_block);

                        self.builder.switch_to_block(default_block);
                        for s in &cas.cons{
                            ctx.translate_stmt(s, None)?;
                        }
                        self.builder.ins().jump(exit_block, &[]);
                    }
                }

                ctx.close();

                self.builder.ins().jump(exit_block, &[]);

                self.builder.seal_block(exit_block);
                self.builder.switch_to_block(exit_block);
            },
            
            Stmt::Throw(t) => {

                let arg = self.translate_expr(&t.arg)?;
                self.close();

                if self.is_in_try{
                    self.builder.ins().jump(*self.try_exits.borrow().last().unwrap(), &[arg]);
                } else{
                    self.builder.ins().call(self.throw, &[arg]);
                }
                
            },

            Stmt::Try(t) => {
                let mut ctx = self.new_context();

                let catch_block = self.builder.create_block();
                let final_block = self.builder.create_block();

                self.builder.append_block_param(catch_block, JValue::TYPE);

                // register the exit
                ctx.is_in_try = true;
                ctx.try_exits.borrow_mut().push(catch_block);

                for i in &t.block.stmts{
                    ctx.translate_stmt(i, None)?;
                }

                // remove the exit
                ctx.try_exits.borrow_mut().pop();
                ctx.close();
                
                // jump to the final block if fallthrough
                self.builder.ins().jump(final_block, &[]);

                // the catch block
                self.builder.seal_block(catch_block);
                self.builder.switch_to_block(catch_block);

                
                if let Some(clause) = &t.handler{
                    ctx = self.new_context();
                    
                    if let Some(p) = &clause.param{
                        let err = self.builder.block_params(catch_block)[0];
                        self.translate_pat(p, err, Some(VarDeclKind::Var), AssignOp::Assign)?;
                    }

                    for i in &clause.body.stmts{
                        ctx.translate_stmt(i, None)?;
                    }
                    ctx.close();
                }
                self.builder.ins().jump(final_block, &[]);


                // the final block
                self.builder.seal_block(final_block);
                self.builder.switch_to_block(final_block);

                if let Some(c) = &t.finalizer{
                    ctx = self.new_context();

                    for i in &c.stmts{
                        ctx.translate_stmt(i, None)?;
                    }
                    ctx.close();
                }
            },

            Stmt::While(w) => {
                let entry_block = self.builder.create_block();
                let exit_block = self.builder.create_block();

                self.loop_exits.borrow_mut().push(LoopExit { 
                    label: label, 
                    exit_block, 
                    continue_block: entry_block 
                });

                self.builder.ins().jump(entry_block, &[]);
                
                self.builder.switch_to_block(entry_block);
                
                let mut ctx = self.new_context();

                // break if false
                let test_re = ctx.translate_expr(&w.test)?;
                let b = ctx.to_bool(test_re);
                self.builder.ins().brz(b, exit_block, &[]);

                // body
                ctx.translate_stmt(w.body.as_ref(), None)?;
                
                ctx.close();
                self.loop_exits.borrow_mut().pop();

                // loop
                self.builder.ins().jump(entry_block, &[]);

                self.builder.seal_block(entry_block);
                self.builder.seal_block(exit_block);

                self.builder.switch_to_block(exit_block);
            },

            Stmt::With(w) => {
                return Err(Error::Deprecated("`with` statment is deprecated and not supported."))
            }
        }
        Ok(())
    }


    pub fn translate_expr(&mut self, expr:&Expr) -> Result<Value, Error>{
        match expr{
            Expr::Array(a) => {
                let mut spread = false;

                let slot = self.builder.create_stack_slot(StackSlotData { 
                    kind: StackSlotKind::ExplicitSlot, 
                    size: (a.elems.len() * size_of::<JValue>()) as u32,
                });

                let mut i = 0;
                for e in &a.elems{
                    if let Some(e) = e{
                        if e.spread.is_some(){
                            spread = true;
                        }

                        let v = self.translate_expr(&e.expr)?;
                        self.builder.ins().stack_store(v, slot, (i*size_of::<JValue>()) as i32); 
                    } else{
                        let v = self.builder.ins().iconst(types::I128, 0);
                        self.builder.ins().stack_store(v, slot, (i*size_of::<JValue>()) as i32);
                    };
                    i += 1;
                }
                
                let addr = self.builder.ins().stack_addr(types::I64, slot, 0);
                let len = self.builder.ins().iconst(types::I64, a.elems.len() as i64);
                let spread = self.builder.ins().bconst(types::B8, spread);
                
                let inner = self.builder.ins().call(self.array_new, &[addr, len, spread]);
                let inner = self.builder.inst_results(inner)[0];

                let object = self.builder.ins().call(self.object_from_inner, &[inner]);
                let object = self.builder.inst_results(object)[0];
                Ok(object)
            },

            Expr::Arrow(a) => {
                todo!()
            },

            Expr::Assign(a) => {
                let value = self.translate_expr(&a.right)?;

                match &a.left{
                    PatOrExpr::Expr(e) => {
                        match e.as_ref(){
                            Expr::Member(m) => {

                                let prop = self.translate_prop(&m.prop, m.computed)?;
                                let obj = match &m.obj{
                                    ExprOrSuper::Expr(e) => self.translate_expr(&e)?,
                                    ExprOrSuper::Super(s) => todo!()
                                };

                                match a.op{
                                    AssignOp::Assign => {
                                        self.builder.ins().call(self.set_memebr, &[obj, prop, value]);
                                    },
                                    op => {
                                        let op = self.builder.ins().iconst(types::I8, op as i8 as i64);
                                        let ins = self.builder.ins().call(self.assign_member, &[obj, prop, value, op]);
                                        return Ok(self.builder.inst_results(ins)[0])
                                    }
                                }

                            },
                            _ => return Err(Error::Unimplemented("unimplemented assign expr."))
                        }
                    },
                    PatOrExpr::Pat(p) => {
                        self.translate_pat(&p, value, None, a.op);
                    }
                }
                Ok(value)
            },

            Expr::Await(a) => {
                todo!()
            },

            Expr::Bin(b) => {
                let right = self.translate_expr(&b.right)?;
                let left = self.translate_expr(&b.left)?;

                let ins = match b.op{
                    BinaryOp::Add => self.builder.ins().call(self.add, &[left, right]),
                    BinaryOp::BitAnd => self.builder.ins().call(self.bitAnd, &[left, right]),
                    BinaryOp::BitOr => self.builder.ins().call(self.bitOr, &[left, right]),
                    BinaryOp::BitXor => self.builder.ins().call(self.bitXor, &[left, right]),
                    BinaryOp::Div => self.builder.ins().call(self.div, &[left, right]),
                    BinaryOp::EqEq => self.builder.ins().call(self.eqeq, &[left, right]),
                    BinaryOp::EqEqEq => self.builder.ins().call(self.eqeqeq, &[left, right]),
                    BinaryOp::Exp => self.builder.ins().call(self.exp, &[left, right]),
                    BinaryOp::Gt => self.builder.ins().call(self.gt, &[left, right]),
                    BinaryOp::GtEq => self.builder.ins().call(self.gteq, &[left, right]),
                    BinaryOp::In => self.builder.ins().call(self.In, &[left, right]),
                    BinaryOp::InstanceOf => self.builder.ins().call(self.instanceof, &[left, right]),
                    BinaryOp::LShift => self.builder.ins().call(self.lshift, &[left, right]),
                    BinaryOp::LogicalAnd => self.builder.ins().call(self.and, &[left, right]),
                    BinaryOp::LogicalOr => self.builder.ins().call(self.or, &[left, right]),
                    BinaryOp::Lt => self.builder.ins().call(self.lt, &[left, right]),
                    BinaryOp::LtEq => self.builder.ins().call(self.lteq, &[left, right]),
                    BinaryOp::Mod => self.builder.ins().call(self.Mod, &[left, right]),
                    BinaryOp::Mul => self.builder.ins().call(self.mul, &[left, right]),
                    BinaryOp::NotEq => self.builder.ins().call(self.noteq, &[left, right]),
                    BinaryOp::NotEqEq => self.builder.ins().call(self.noteqeq, &[left, right]),
                    BinaryOp::NullishCoalescing => self.builder.ins().call(self.nullishCoalscing, &[left, right]),
                    BinaryOp::RShift => self.builder.ins().call(self.rshift, &[left, right]),
                    BinaryOp::Sub => self.builder.ins().call(self.sub, &[left, right]),
                    BinaryOp::ZeroFillRShift => self.builder.ins().call(self.unsignedRShift, &[left, right]),
                };
                Ok(self.builder.inst_results(ins)[0])
            }

            Expr::Call(c) => {

                let (argv, argc, spread) = self.translate_args(&c.args)?;

                match &c.callee{
                    ExprOrSuper::Expr(e) => {

                        // determine the call
                        let (re, ok) = match e.as_ref(){

                            // this is a member call
                            Expr::Member(m) => {
                                match &m.obj{
                                    ExprOrSuper::Expr(e) => {
                                        let callee = self.translate_expr(&e)?;
                                        let prop = self.translate_expr(&m.prop)?;
                                        let vmctx = self.builder.use_var(self.vmctx);
                                        let ins = self.builder.ins().call(self.memberCall, &[callee, prop, vmctx, argv, argc, spread]);
                                        let v = self.builder.inst_results(ins);
                                        (v[0], v[1])
                                    },

                                    ExprOrSuper::Super(s) => {
                                        let prop = self.translate_expr(&m.prop)?;
                                        let this = self.builder.use_var(self.this);
                                        let vmctx = self.builder.use_var(self.vmctx);

                                        let ins = self.builder.ins().call(self.superMemberCall, &[
                                            this, 
                                            prop,
                                            vmctx, 
                                            argv, argc, spread
                                        ]);
                                        let v = self.builder.inst_results(ins);
                                        (v[0], v[1])
                                    }

                                }
                            },

                            // this is a function call
                            callee => {
                                let callee = self.translate_expr(callee)?;
                                let vmctx = self.builder.use_var(self.vmctx);
                                let this = self.builder.use_var(self.this);
                                let ins = self.builder.ins().call(self.call, &[
                                    callee, 
                                    vmctx, 
                                    this, 
                                    argv, argc, spread
                                ]);
                                let v = self.builder.inst_results(ins);
                                (v[0], v[1])
                            }
                        };

                        if self.is_in_try{

                            self.builder.ins().brz(ok, *self.try_exits.borrow().last().unwrap(), &[re]);

                        } else{
                            let throw_block = self.builder.create_block();
                            let exit_block = self.builder.create_block();

                            self.builder.append_block_param(throw_block, JValue::TYPE);

                            self.builder.ins().brz(ok, throw_block, &[re]);
                            self.builder.ins().jump(exit_block, &[]);

                            self.builder.seal_block(throw_block);
                            self.builder.switch_to_block(throw_block);

                            let throw_value = self.builder.block_params(throw_block)[0];

                            self.builder.ins().call(self.throw, &[throw_value]);
                            self.builder.ins().jump(exit_block, &[]);

                            self.builder.seal_block(exit_block);
                            self.builder.switch_to_block(exit_block);

                            
                        };
                        Ok(re)
                    },

                    _ => todo!("todo: super call")
                }
                
            }
            _ => todo!()
        }
    }

    pub fn translate_var_decl(&mut self, decl:&VarDecl) -> Result<(), Error>{
        todo!()
    }

    pub fn translate_pat(&mut self, p:&Pat, val:Value, kind:Option<VarDeclKind>, op:AssignOp) -> Result<(), Error>{
        todo!()
    }

    pub fn translate_prop(&mut self, expr:&Expr, computed:bool) -> Result<Value, Error>{
        if computed{
            self.translate_expr(expr)
        } else{
            match expr{
                Expr::Ident(i) => Ok(self.const_value(i.sym.as_ref().into())),
                _ => return Err(Error::Unimplemented("non computed property expression."))
            }
        }
    }

    /// return *mut u8, i64, bool
    pub fn translate_args(&mut self, v:&[ExprOrSpread]) -> Result<(Value, Value, Value), Error>{

        let mut spread = false;

        let slot = self.builder.create_stack_slot(StackSlotData { 
            kind: StackSlotKind::ExplicitSlot, 
            size: (v.len() * size_of::<JValue>()) as u32,
        });

        let mut i = 0;
        for e in v{
            if e.spread.is_some(){
                spread = true;
            }

            let v = self.translate_expr(&e.expr)?;
            self.builder.ins().stack_store(v, slot, (i*size_of::<JValue>()) as i32);

            i += 1;
        }
        Ok((
            self.builder.ins().stack_addr(types::I64, slot, 0), 
            self.builder.ins().iconst(types::I64, v.len() as i64),
            self.builder.ins().bconst(types::B8, spread),
        ))
    }
}