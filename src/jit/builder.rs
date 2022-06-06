use std::alloc::Layout;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::mem::size_of;
use std::sync::Arc;

use cranelift::codegen::Context;
use cranelift::codegen::ir::FuncRef;
use cranelift::prelude::*;
use cranelift_jit::JITModule;
use cranelift_module::Module;
use string_interner::Symbol as _;
use swc_ecma_ast::*;

use parking_lot::RwLock;

use num_traits::ToPrimitive;

use crate::builtins::Symbol;
use crate::builtins::regexp::RegExp;
use crate::runtime::Runtime;
use crate::error::Error;
use crate::value::JValue;

pub struct LoopExit{
    label:Option<String>,
    exit_block:Block,
    continue_block:Block,
}

pub struct BuilderContext<'a>{

    runtime:Arc<Runtime>,
    parent:Option<&'a mut BuilderContext<'a>>,
    is_func:bool,

    builder:&'static mut FunctionBuilder<'static>,
    /// for drop only
    func_ctx:*mut FunctionBuilderContext,

    jit_module:Arc<JITModule>,
    variables:HashSet<usize>,

    loop_exits:Arc<RefCell<Vec<LoopExit>>>,

    is_in_try:bool,
    try_exits:Arc<RefCell<Vec<Block>>>,

    need_capture:Arc<RefCell<Vec<u64>>>,

    pointer_type:Type,

    /// if this is a function:
    /// 
    /// params: (*mut VmContext, this:JValue, argv:*mut JValue, argc:*mut JValue)
    entry_block:Block,

    vmctx:Variable,
    this:Variable,

    /// fn(*mut VmContext, id:i64) -> JValue
    resolve_var:FuncRef,
    /// fn(*mut VmContext, id:i64, value:JValue)
    set_var:FuncRef,
    /// fn(argv:*mut JValue, argc:i64, idx:i64) -> JValue
    resolve_argument:FuncRef,

    /// fn(value:JValue) -> bool
    to_bool:FuncRef,
    /// fn(value:JValue) -> !
    throw:FuncRef,

    /// fn(self:JValue, key:JValue) -> JValue
    member:FuncRef,
    superMember:FuncRef,
    /// fn(self:JValue, key:JValue, value:JValue)
    set_memebr:FuncRef,
    /// fn(self:JValue, key:JValue, value:JValue, op:i8)
    assign_member:FuncRef,
    /// use for object construction
    /// fn(self:JValue, spreadable:JValue)
    set_member_spread:FuncRef,

    /// fn(callee:JValue, *mut VmContext, this:JValue, argv:*mut JValue, argc:i64, spread:bool) -> (JValue, ok:bool)
    call:FuncRef,

    /// fn(callee:JValue, *mut VmContext, argv:*mut JValue, argc:i64, spread:bool) -> (JValue, ok:bool)
    construct:FuncRef,

    /// fn(self:JValue, member:Jvalue, *mut VmContext, argv:*mut JValue, argc:i64, spread:bool) -> (JValue, ok:bool)
    memberCall:FuncRef,
    /// fn(self:JValue, member:JValue, *mut VmContext, argv:*mut JValue, argc:i64, spread:bool) -> (Jvalue, ok:bool)
    superMemberCall:FuncRef,
    
    /// fn(argv:*mut JValue, argc:i64) -> JValue
    tpl_new:FuncRef,

    /// fn(argv:*mut JValue, argc:i64, spread:bool) -> Arc<dyn JObjectInner>
    array_new:FuncRef,
    /// fn(vmctx:*mut VmContext, mem:*mut u8, async:bool, generator:bool) -> JValue
    function_new:FuncRef,

    new_object:FuncRef,

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

impl<'a> BuilderContext<'a>{

    pub fn new(runtime:Arc<Runtime>, module:Arc<cranelift_jit::JITModule>, ctx:&mut Context) -> Self{
        
        let func = &mut ctx.func;
        

        let resolve_var= module.declare_func_in_func(runtime.builtin_functions["resolve_var"], func);
        let set_var= module.declare_func_in_func(runtime.builtin_functions["set_var"], func);
        let resolve_argument= module.declare_func_in_func(runtime.builtin_functions["resolve_argument"], func);
        let to_bool= module.declare_func_in_func(runtime.builtin_functions["to_bool"], func);
        let throw =  module.declare_func_in_func(runtime.builtin_functions["throw"], func);
        let member= module.declare_func_in_func(runtime.builtin_functions["member"], func);
        let superMember= module.declare_func_in_func(runtime.builtin_functions["superMember"], func);
        let set_memebr= module.declare_func_in_func(runtime.builtin_functions["set_member"], func);
        let assign_member = module.declare_func_in_func(runtime.builtin_functions["assign_member"], func);
        let set_member_spread= module.declare_func_in_func(runtime.builtin_functions["set_member_spread"], func);
        let call= module.declare_func_in_func(runtime.builtin_functions["call"], func);
        let construct= module.declare_func_in_func(runtime.builtin_functions["construct"], func);
        let memberCall= module.declare_func_in_func(runtime.builtin_functions["memberCall"], func);
        let superMemberCall= module.declare_func_in_func(runtime.builtin_functions["superMemberCall"], func);
        let tpl_new= module.declare_func_in_func(runtime.builtin_functions["tpl_new"], func);
        let array_new= module.declare_func_in_func(runtime.builtin_functions["array_new"], func);
        let function_new= module.declare_func_in_func(runtime.builtin_functions["function_new"], func);
        let new_object= module.declare_func_in_func(runtime.builtin_functions["new_object"], func);
        let object_from_inner = module.declare_func_in_func(runtime.builtin_functions["object_from_inner"], func);
        let add = module.declare_func_in_func(runtime.builtin_functions["add"], func);
        let bitAnd= module.declare_func_in_func(runtime.builtin_functions["bitAnd"], func);
        let bitOr = module.declare_func_in_func(runtime.builtin_functions["bitOr"], func);
        let bitXor= module.declare_func_in_func(runtime.builtin_functions["bitXor"], func);
        let div= module.declare_func_in_func(runtime.builtin_functions["div"], func);
        let eqeq= module.declare_func_in_func(runtime.builtin_functions["eqeq"], func);
        let eqeqeq= module.declare_func_in_func(runtime.builtin_functions["eqeqeq"], func);
        let exp = module.declare_func_in_func(runtime.builtin_functions["exp"], func);
        let gt= module.declare_func_in_func(runtime.builtin_functions["gt"], func);
        let gteq= module.declare_func_in_func(runtime.builtin_functions["gteq"], func);
        let In= module.declare_func_in_func(runtime.builtin_functions["in"], func);
        let instanceof= module.declare_func_in_func(runtime.builtin_functions["instanceof"], func);
        let lshift= module.declare_func_in_func(runtime.builtin_functions["lshift"], func);
        let and= module.declare_func_in_func(runtime.builtin_functions["and"], func);
        let or= module.declare_func_in_func(runtime.builtin_functions["or"], func);
        let lt= module.declare_func_in_func(runtime.builtin_functions["lt"], func);
        let lteq= module.declare_func_in_func(runtime.builtin_functions["lteq"], func);
        let Mod= module.declare_func_in_func(runtime.builtin_functions["mod"], func);
        let mul= module.declare_func_in_func(runtime.builtin_functions["mul"], func);
        let noteq= module.declare_func_in_func(runtime.builtin_functions["noteq"], func);
        let noteqeq= module.declare_func_in_func(runtime.builtin_functions["noteqeq"], func);
        let nullishCoalscing= module.declare_func_in_func(runtime.builtin_functions["nullishCoalscing"], func);
        let rshift= module.declare_func_in_func(runtime.builtin_functions["rshift"], func);
        let sub= module.declare_func_in_func(runtime.builtin_functions["sub"], func);
        let unsignedRShift= module.declare_func_in_func(runtime.builtin_functions["unsignedRShift"], func);


        let mut func_ctx = Box::leak(Box::new(FunctionBuilderContext::new()));
        let func_ctx_ptr = func_ctx as *mut FunctionBuilderContext;
        let builder = Box::leak(Box::new(FunctionBuilder::new(unsafe{std::mem::transmute_copy(&func)}, func_ctx)));

        builder.declare_var(Variable::with_u32(0), module.isa().pointer_type());
        builder.declare_var(Variable::with_u32(1), JValue::TYPE);
        let entry_block = builder.create_block();

        return Self { 
            runtime:runtime.clone(), 
            parent: None, 

            is_func:true,
            builder: builder, 
            func_ctx:func_ctx_ptr,
            jit_module:module.clone(),

            variables: Default::default(), 
            loop_exits: Arc::new(RefCell::new(Vec::new())), 
            is_in_try: false, 
            try_exits: Arc::new(RefCell::new(Vec::new())), 
            need_capture: Arc::new(RefCell::new(Vec::new())), 

            entry_block,

            vmctx: Variable::with_u32(0), 
            this: Variable::with_u32(1), 

            pointer_type:module.isa().pointer_type(),

            resolve_var,
            set_var,
            resolve_argument,
            to_bool,
            throw,
            member,
            superMember,
            set_memebr,
            assign_member,
            set_member_spread,
            call,
            construct,
            memberCall,
            superMemberCall,
            tpl_new,
            function_new,
            array_new,
            new_object,
            object_from_inner,
            add,
            bitAnd,
            bitOr,
            bitXor,
            div,
            eqeq,
            eqeqeq,
            exp,
            gt,
            gteq,
            In,
            instanceof,
            lshift,
            and,
            or,
            lt,
            lteq,
            Mod,
            mul,
            noteq,
            noteqeq,
            nullishCoalscing,
            rshift,
            sub,
            unsignedRShift,
        }
    }

    pub fn new_context(&mut self) -> Self{
        unsafe{
            return Self{
                runtime:self.runtime.clone(),
                parent:Some(std::mem::transmute_copy(&self)),
                is_func:false,

                builder:std::mem::transmute_copy(&self.builder),
                jit_module:self.jit_module.clone(),

                variables:Default::default(),
                loop_exits:self.loop_exits.clone(),

                is_in_try:self.is_in_try,
                try_exits:self.try_exits.clone(),

                need_capture:self.need_capture.clone(),

                ..*self
            }
        }
        
    }

    pub fn new_function(&mut self) -> (Self, &'static mut Context){
        let mut context = Box::leak(Box::new(self.jit_module.make_context()));
        let mut f = Self::new(self.runtime.clone(), self.jit_module.clone(), context);
        f.parent = Some(unsafe{std::mem::transmute_copy(&self)});
        (f, context)
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

    pub fn has_variable(&self, v:usize) -> bool{
        if self.variables.contains(&v){
            true
        } else{
            if let Some(p) = &self.parent{
                p.has_variable(v)
            } else{
                false
            }
        }
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

                self.loop_exits.as_ref().borrow_mut().push(LoopExit { 
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
                self.loop_exits.as_ref().borrow_mut().pop();

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

                self.loop_exits.as_ref().borrow_mut().push(LoopExit { 
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
                self.loop_exits.as_ref().borrow_mut().pop();

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
                ctx.try_exits.as_ref().borrow_mut().push(catch_block);

                for i in &t.block.stmts{
                    ctx.translate_stmt(i, None)?;
                }

                // remove the exit
                ctx.try_exits.as_ref().borrow_mut().pop();
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

                self.loop_exits.as_ref().borrow_mut().push(LoopExit { 
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
                self.loop_exits.as_ref().borrow_mut().pop();

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
                let (mut builder, ctx) = self.new_function();

                let argv = builder.builder.block_params(builder.entry_block)[1];
                let argc = builder.builder.block_params(builder.entry_block)[2];
                let mut i = 0;
                for p in &a.params{
                    let c = builder.builder.ins().iconst(types::I64, i);
                    let v = builder.builder.ins().call(builder.resolve_argument, &[argv, argc, c]);
                    let v = builder.builder.inst_results(v)[0];
                    builder.translate_pat(p, v, Some(VarDeclKind::Var), AssignOp::Assign)?;
                };
                match &a.body{
                    BlockStmtOrExpr::BlockStmt(b) => {
                        for stmt in &b.stmts{
                            builder.translate_stmt(stmt, None)?;
                        }
                    },
                    BlockStmtOrExpr::Expr(e) => {
                        let v = builder.translate_expr(&e)?;
                        builder.close();
                        builder.builder.ins().return_(&[v]);
                    }
                }

                builder.close();
                let un = builder.const_value(JValue::Undefined);
                builder.builder.ins().return_(&[un]);
                builder.builder.seal_all_blocks();

                let re = ctx.compile(self.jit_module.isa());
                let info = match re{
                    Ok(v) => v,
                    Err(e) => return Err(Error::CodegenError(Arc::new(e)))
                };

                let mem = unsafe{std::alloc::alloc(Layout::array::<u8>(info.total_size as usize).unwrap())};
                unsafe{ctx.emit_to_memory(mem)};
                builder.builder.finalize();
                ctx.clear();


                self.runtime.to_mut().new_compiled_fn(mem, info.total_size as usize);

                let vmctx = self.builder.use_var(self.vmctx);
                let addr = self.builder.ins().iconst(self.pointer_type, mem as i64);
                let is_async = self.builder.ins().bconst(types::B8, a.is_async);
                let is_generator = self.builder.ins().bconst(types::B8, a.is_generator);

                let inst = self.builder.ins().call(self.function_new, &[vmctx, addr, is_async, is_generator]);
                Ok(self.builder.inst_results(inst)[0])
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
            },

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
                
            },

            Expr::Class(c) => {
                todo!()
            },

            Expr::Cond(c) => {
                let test = self.translate_expr(&c.test)?;
                let cons = self.translate_expr(&c.cons)?;
                let alt = self.translate_expr(&c.alt)?;

                let test = self.to_bool(test);
                let v = self.builder.ins().select(test, cons, alt);
                return Ok(v)
            },

            Expr::Fn(f) => {
                todo!()
            },

            Expr::Ident(i) => {
                let id = self.runtime.to_mut().variable_names.get_or_intern(i.sym.as_ref());
                if self.has_variable(id.to_usize()){

                }

                let vmctx = self.builder.use_var(self.vmctx);
                let id = self.builder.ins().iconst(types::I64, id.to_usize() as i64);
                let inst = self.builder.ins().call(self.resolve_var, &[id]);
                Ok(self.builder.inst_results(inst)[0])
            },

            Expr::Invalid(i) => {
                Ok(self.const_value(JValue::Undefined))
            },

            Expr::Lit(l) => {
                match l{
                    Lit::BigInt(b) => {

                        Ok(self.const_value(JValue::BigInt(b.value.to_i64().unwrap())))
                    },
                    Lit::Bool(b) => {
                        Ok(self.const_value(JValue::Boolean(b.value)))
                    },
                    Lit::Null(n) => {
                        Ok(self.const_value(JValue::Null))
                    },
                    Lit::Num(n) => {
                        Ok(self.const_value(JValue::Number(n.value)))
                    },
                    Lit::Regex(r) => {
                        Ok(self.const_value(RegExp::from_str(&r.exp, &r.flags)))
                    },
                    Lit::Str(s) => {
                        Ok(self.const_value(s.value.as_ref().into()))
                    }
                    Lit::JSXText(t) => todo!()
                }
            },
            
            Expr::Member(m) => {
                let prop = self.translate_prop(&m.prop, m.computed)?;

                match &m.obj{
                    ExprOrSuper::Expr(e) => {
                        let obj = self.translate_expr(&e)?;
                        let inst = self.builder.ins().call(self.member, &[obj, prop]);
                        Ok(self.builder.inst_results(inst)[0])
                    },
                    ExprOrSuper::Super(s) => {
                        let obj = self.builder.use_var(self.this);
                        let inst = self.builder.ins().call(self.superMember, &[obj, prop]);
                        Ok(self.builder.inst_results(inst)[0])
                    }
                }
            },

            Expr::MetaProp(m) => {
                todo!()
            },

            Expr::New(n) => {
                let (argv, argc, spread) = if let Some(a) = &n.args{
                    self.translate_args(&a)?
                } else{
                    (
                        self.builder.ins().iconst(types::I64, 0),
                        self.builder.ins().iconst(types::I64, 0),
                        self.builder.ins().bconst(types::B8, false),
                    )
                };

                let callee = self.translate_expr(&n.callee)?;
                
                let vmctx = self.builder.use_var(self.vmctx);
                let inst = self.builder.ins().call(self.construct, &[
                    callee, 
                    vmctx, 
                    argv, argc, spread
                ]);
                let v = self.builder.inst_results(inst);
                let (re, ok) = (v[0], v[1]);
                
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

            Expr::Object(o) => {
                let obj = self.builder.ins().call(self.new_object, &[]);
                let obj = self.builder.inst_results(obj)[0];
                for p in &o.props{
                    match p{
                        PropOrSpread::Spread(s) => {
                            let spread = self.translate_expr(&s.expr)?;
                            self.builder.ins().call(self.set_member_spread, &[obj, spread]);
                        },
                        PropOrSpread::Prop(p) => {
                            match p.as_ref(){
                                Prop::Shorthand(i) => {
                                    let id = self.runtime.to_mut().variable_names.get_or_intern(i.sym.as_ref());
                                    if self.has_variable(id.to_usize()){

                                    }
                                    let id = self.builder.ins().iconst(types::I64, id.to_usize() as i64);
                                    let inst = self.builder.ins().call(self.resolve_var, &[id]);
                                    let v = self.builder.inst_results(inst)[0];

                                    let prop = self.const_value(i.sym.as_ref().into());

                                    self.builder.ins().call(self.set_memebr, &[prop, v]);
                                },
                                Prop::KeyValue(k) => {

                                },
                                _ => todo!()
                            }
                        }
                    }
                };
                Ok(obj)
            },

            Expr::OptChain(o) => {
                todo!()
            },

            Expr::Paren(p) => {
                self.translate_expr(&p.expr)
            },

            Expr::PrivateName(p) => {
                Ok(self.const_value(Symbol::new_private(&p.id.sym)))
            },

            Expr::Seq(s) => {
                let mut v = Value::from_u32(0);
                for i in &s.exprs{
                    v = self.translate_expr(&i)?;
                };
                Ok(v)
            },

            Expr::TaggedTpl(t) => {
                todo!()
            },
            Expr::This(t) => {
                Ok(self.builder.use_var(self.this))
            },

            Expr::Tpl(t) => {
                let mut vs = Vec::new();

                let mut i = 0;
                for e in &t.quasis{
                    vs.push(self.const_value(e.raw.value.as_ref().into()));

                    if !e.tail{
                        vs.push(self.translate_expr(&t.exprs[i])?);
                    }
                    i += 1;
                }

                let slot = self.builder.create_stack_slot(StackSlotData { 
                    kind: StackSlotKind::ExplicitSlot, 
                    size: (vs.len() *size_of::<JValue>()) as u32
                });

                let mut i = 0;
                for v in &vs{
                    self.builder.ins().stack_store(*v, slot, i*size_of::<JValue>() as i32);
                    i += 1;
                }

                let addr = self.builder.ins().stack_addr(types::I64, slot, 0);
                let len = self.builder.ins().iconst(types::I64, vs.len() as i64);
                let inst = self.builder.ins().call(self.tpl_new, &[addr, len]);
                Ok(self.builder.inst_results(inst)[0])
            },

            Expr::Unary(u) => {
                todo!()
            },

            Expr::Update(u) => {
                todo!()
            },

            Expr::Yield(y) => {
                todo!()
            }

            Expr::TsAs(a) => todo!(),
            Expr::TsConstAssertion(c) => todo!(),
            Expr::TsNonNull(n) => todo!(),
            Expr::TsTypeAssertion(t) => todo!(),

            Expr::JSXElement(e) => todo!(),
            Expr::JSXEmpty(e) => todo!(),
            Expr::JSXFragment(f) => todo!(),
            Expr::JSXMember(m) => todo!(),
            Expr::JSXNamespacedName(n) => todo!(),
        }
    }

    pub fn translate_var_decl(&mut self, decl:&VarDecl) -> Result<(), Error>{
        for dec in &decl.decls{
            let val = if let Some(e) = &dec.init{
                self.translate_expr(&e)?
            } else{
                self.const_value(JValue::Undefined)
            };

            self.translate_pat(&dec.name, val, Some(decl.kind), AssignOp::Assign)?;
        }
        Ok(())
    }

    pub fn translate_pat(&mut self, p:&Pat, val:Value, kind:Option<VarDeclKind>, op:AssignOp) -> Result<(), Error>{
        match p{
            Pat::Assign(a) => {
                let right = self.translate_expr(&a.right)?;
                let val = self.builder.ins().select(val, val, right);
                return self.translate_pat(&a.left, val, kind, op)
            },
            Pat::Array(a) => {
                
            },
            Pat::Expr(e) => todo!(),
            Pat::Ident(i) => {
                let id = self.runtime.to_mut().new_variable_name(&i.id.sym);
                let id_const = self.builder.ins().iconst(types::I64, id as i64);

                let vmctx = self.builder.use_var(self.vmctx);
                self.builder.ins().call(self.set_var, &[vmctx, id_const, val]);
            },
            Pat::Invalid(i) => todo!(),
            Pat::Object(o) => {

            }
            Pat::Rest(r) => unimplemented!()
        }
        Ok(())
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

impl<'a> Drop for BuilderContext<'a>{
    fn drop(&mut self) {
        if self.parent.is_none(){
            unsafe{
                std::ptr::drop_in_place(self.builder);
                std::ptr::drop_in_place(self.func_ctx);
            }
        }
    }
}