use std::panic::panic_any;
use std::{sync::Arc, cell::RefCell};

use swc_ecma_ast::*;

use async_recursion::async_recursion;

use crate::builtins::promise::Promise;
use crate::{builtins, operator};
use crate::builtins::object::JObject;
use crate::value::JValue;
use crate::{runtime::Runtime, vm::VmContext};
use crate::error::Error;

#[derive(Clone)]
pub struct Interpreter{
    runtime:Arc<&'static Runtime>,
}

impl Interpreter{

    #[async_recursion]
    pub async fn translate_stmt(&mut self, vmctx:&mut VmContext, stmt:&'static Stmt, label:Option<String>) -> Result<(), Error>{
        match stmt{
            Stmt::Block(b) => {
                let ctx = vmctx.new_child();

                for i in &b.stmts{
                    self.translate_stmt(ctx, i, None).await?;
                }
                ctx.done();
            },
            Stmt::Break(b) => {
                if let Some(l) = &b.label{
                    return Err(Error::Break(Some(l.sym.to_string())))
                } else{
                    return Err(Error::Break(None))
                }
            },
            Stmt::Continue(c) => {
                if let Some(l) = &c.label{
                    return Err(Error::Continue(Some(l.sym.to_string())))
                } else{
                    return Err(Error::Continue(None))
                }
            },
            Stmt::Debugger(d) => {
                todo!()
            },
            Stmt::Decl(d) => {
                self.translate_decl(vmctx, d).await?;
            },
            Stmt::DoWhile(d) => {
                loop{
                    let ctx = vmctx.new_child();
                    let re = self.translate_stmt(ctx, &d.body, None).await;
                    if let Err(e) = &re{
                        match e{
                            Error::Break(b) => {
                                if b.is_some() && label.is_some(){
                                    // match the labels
                                    if b.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // label matched
                                        ctx.done();
                                        break;
                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re;
                                    }
                                } else if b.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    ctx.done();
                                    break
                                }
                            },
                            Error::Continue(c) => {

                                if c.is_some() && label.is_some(){
                                    // check if label matches
                                    if c.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // go to next loop
                                        ctx.done();
                                        continue;

                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re
                                    }
                                } else if c.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    // go to next loop
                                    ctx.done();
                                    continue;
                                }
                            }
                            _ => return re,
                        };
                    };

                    let test = self.translate_expr(ctx, &d.test).await?;
                    ctx.done();

                    if !test.to_bool(){
                        break
                    }
                }
            },

            Stmt::Empty(e) => {},
            Stmt::Expr(e) => {
                self.translate_expr(vmctx, &e.expr).await?;
            },

            Stmt::For(f) => {
                let ctx = vmctx.new_child();

                if let Some(i) = &f.init{
                    
                    match i{
                        VarDeclOrExpr::Expr(e) => {
                            self.translate_expr(ctx, &e).await?;
                        },
                        VarDeclOrExpr::VarDecl(d) => {
                            self.translate_var_decl(ctx, d).await?;
                        }
                    }
                }

                loop{
                    let ctx = ctx.new_child();

                    if let Some(e) = &f.test{
                        self.translate_expr(ctx, &e).await?;
                    }

                    let re = self.translate_stmt(ctx, &f.body, None).await;

                    // break or continue
                    if let Err(e) = &re{
                        match e{
                            Error::Break(b) => {
                                if b.is_some() && label.is_some(){
                                    // match the labels
                                    if b.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // label matched
                                        ctx.done();
                                        break;
                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re;
                                    }
                                } else if b.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    ctx.done();
                                    break
                                }
                            },
                            Error::Continue(c) => {

                                if c.is_some() && label.is_some(){
                                    // check if label matches
                                    if c.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // go to next loop
                                        ctx.done();
                                        continue;

                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re
                                    }
                                } else if c.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    // go to next loop
                                    ctx.done();
                                    continue;
                                }
                            }
                            _ => return re,
                        };
                    };

                    // update
                    if let Some(e) = &f.update{
                        self.translate_expr(ctx, &e).await?;
                    }
                    ctx.done();
                }

                ctx.done();
            },

            Stmt::ForIn(f) => {
                
                let right = self.translate_expr(vmctx, &f.right).await?;
                let keys = right.owned_keys();

                for key in keys{

                    let ctx = vmctx.new_child();

                    match &f.left{
                        VarDeclOrPat::Pat(p) => self.translate_pat(ctx, p, key.into()).await?,
                        VarDeclOrPat::VarDecl(d) => {
                            let decl = d.decls.first().unwrap();
                            self.translate_pat(ctx, &decl.name, key.into()).await?;
                        },
                    };

                    let re = self.translate_stmt(ctx, &f.body, None).await;

                    // break or continue
                    if let Err(e) = &re{
                        match e{
                            Error::Break(b) => {
                                if b.is_some() && label.is_some(){
                                    // match the labels
                                    if b.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // label matched
                                        ctx.done();
                                        break;
                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re;
                                    }
                                } else if b.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    ctx.done();
                                    break
                                }
                            },
                            Error::Continue(c) => {

                                if c.is_some() && label.is_some(){
                                    // check if label matches
                                    if c.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // go to next loop
                                        ctx.done();
                                        continue;

                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re
                                    }
                                } else if c.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    // go to next loop
                                    ctx.done();
                                    continue;
                                }
                            }
                            _ => return re,
                        };
                    };

                    ctx.done();
                }
                
            },

            Stmt::ForOf(f) => {
                let right = self.translate_expr(vmctx, &f.right).await?;
                let iter = right.member(*builtins::symbol::Iterator);

                let mut done = false;

                loop{

                    let next = iter.member_str("next");

                    if done{
                        break;
                    }

                    done = next.member_str("done").to_bool();
                    let value = next.member_str("value");

                    let ctx = vmctx.new_child();

                    match &f.left{
                        VarDeclOrPat::Pat(p) => self.translate_pat(ctx, p, value).await?,
                        VarDeclOrPat::VarDecl(d) => {
                            let decl = d.decls.first().unwrap();
                            self.translate_pat(ctx, &decl.name, value).await?;
                        },
                    };

                    let re = self.translate_stmt(ctx, &f.body, None).await;

                    // break or continue
                    if let Err(e) = &re{
                        match e{
                            Error::Break(b) => {
                                if b.is_some() && label.is_some(){
                                    // match the labels
                                    if b.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // label matched
                                        ctx.done();
                                        break;
                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re;
                                    }
                                } else if b.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    ctx.done();
                                    break
                                }
                            },
                            Error::Continue(c) => {

                                if c.is_some() && label.is_some(){
                                    // check if label matches
                                    if c.as_ref().unwrap().as_str() == label.as_ref().unwrap().as_str(){
                                        // go to next loop
                                        ctx.done();
                                        continue;

                                    } else{
                                        // label does not match
                                        ctx.done();
                                        return re
                                    }
                                } else if c.is_some(){
                                    ctx.done();
                                    return re
                                } else{
                                    // go to next loop
                                    ctx.done();
                                    continue;
                                }
                            }
                            _ => return re,
                        };
                    };

                    ctx.done();
                }
            },

            Stmt::If(i) => {

                if self.translate_expr(vmctx, &i.test).await?.to_bool(){
                    let ctx = vmctx.new_child();
                    self.translate_stmt(ctx, &i.cons, None).await?;
                    ctx.done();
                } else{
                    if let Some(alt) = &i.alt{
                        let ctx = vmctx.new_child();
                        self.translate_stmt(ctx, &alt, None).await?;
                        ctx.done();
                    }
                }
            },

            Stmt::Labeled(l) => {
                self.translate_stmt(vmctx, &l.body, Some(l.label.sym.to_string())).await?;
            },

            Stmt::Return(r) => {
                if let Some(arg) = &r.arg{
                    return Err(Error::Return(self.translate_expr(vmctx, &arg).await?))
                } else{
                    return Err(Error::Return(JValue::Undefined))
                }
            },

            Stmt::Switch(s) => {
                let target = self.translate_expr(vmctx, &s.discriminant).await?;

                for i in &s.cases{

                    let ctx = vmctx.new_child();

                    if let Some(test) = &i.test{
                        if self.translate_expr(ctx, &test).await?.eqeqeq(target).to_bool(){

                            for stmt in &i.cons{
                                self.translate_stmt(ctx, stmt, None).await?;
                            }
                            ctx.done();
                            break;
                        }
                    } else{
                        for stmt in &i.cons{
                            self.translate_stmt(ctx, stmt, None).await?;
                        }
                        ctx.done();
                        break;
                    }

                    ctx.done();
                }
            },

            Stmt::Throw(t) => {
                let arg = self.translate_expr(vmctx, &t.arg).await?;
                return Err(Error::Value(arg))
            },

            Stmt::Try(t) => {
                let mut ctx = vmctx.new_child();

                let mut re = Ok(());
                for stmt in &t.block.stmts{
                    re = self.translate_stmt(ctx, stmt, None).await;
                    if re.is_err(){
                        break;
                    }
                }
                ctx.done();

                if let Err(err) = re{

                    if let Error::Value(err) = err{
                        if let Some(h) = &t.handler{
                            let ctx = vmctx.new_child();
                    
                            if let Some(p) = &h.param{
                                self.translate_pat(ctx, p, err).await?;
                            }
                            
                            for stmt in &h.body.stmts{
                                self.translate_stmt(ctx, stmt, None).await?;
                            }
                            ctx.done()
                        }

                    } else{
                        // not a runtime error
                        return Err(err)
                    }
                }

                if let Some(f) = &t.finalizer{
                    let ctx = vmctx.new_child();

                    for stmt in &f.stmts{
                        self.translate_stmt(ctx, stmt, None).await?;
                    }

                    ctx.done();
                }
            },

            Stmt::While(w) => {
                loop{
                    if self.translate_expr(vmctx, &w.test).await?.to_bool(){
                        let ctx = vmctx.new_child();
                        self.translate_stmt(ctx, &w.body, None);
                        ctx.done();
                    } else{
                        break;
                    }
                }
            },

            Stmt::With(w) => {
                return Err(Error::Deprecated("with statment is deprecated and no longer supported."))
            }
        }
        Ok(())
    }


    #[async_recursion]
    pub async fn translate_expr(&mut self, vmctx:&mut VmContext, expr:&'static Expr) -> Result<JValue, Error>{
        match expr{
            Expr::Array(a) => {
                todo!()
            },
            Expr::Arrow(a) => {

                let vm = self.clone();
                
                // not a generator or async function
                if !a.is_async && !a.is_generator{

                    // create a clousure
                    Ok(JObject::fromInner(builtins::function::Function::native(move |vmctx, this, args|{

                        let mut vm = vm.clone();

                        let mut i = 0;
                        let mut err = Ok(());

                        // asign all the params
                        for p in &a.params{
                            if let Some(v) = args.get(i){
                                err = futures::executor::block_on(vm.translate_pat(vmctx, p, *v));
                            } else{
                                err = futures::executor::block_on(vm.translate_pat(vmctx, p, JValue::Undefined));
                            }
                            check_err(&err);
                            i+=1;
                        }

                        // body
                        match &a.body{
                            BlockStmtOrExpr::Expr(e) => {
                                let re = futures::executor::block_on(vm.translate_expr(vmctx, &e));
                                if let Ok(v) = re{
                                    return v
                                } else{
                                    check_err(&Err(re.err().unwrap()))
                                }
                            },
                            BlockStmtOrExpr::BlockStmt(b) => {
                                for stmt in &b.stmts{
                                    err = futures::executor::block_on(vm.translate_stmt(vmctx, stmt, None));

                                    if let Err(Error::Return(v)) = err{
                                        return v;
                                    } else{
                                        check_err(&err)
                                    }
                                }
                            }
                        }

                        JValue::Undefined
                    })).into())
                    
                } else if a.is_async && !a.is_generator{

                    // is async but not generator
                    return Ok(JObject::fromInner(builtins::function::Function::native(move |vmctx, this, args|{
                        let mut vm = vm.clone();

                        let mut i = 0;
                        let mut err = Ok(());

                        let vmctx = vmctx.new_child();

                        // asign params
                        for p in &a.params{
                            if let Some(v) = args.get(i){
                                err = futures::executor::block_on(vm.translate_pat(vmctx, p, *v));
                            } else{
                                err = futures::executor::block_on(vm.translate_pat(vmctx, p, JValue::Undefined));
                            }
                            check_err(&err);
                            i+=1;
                        }

                        // return and spawn a promise
                        return JObject::fromInner(Promise::native(async move{
                            // the async part
                            
                            // body
                            match &a.body{
                                BlockStmtOrExpr::Expr(e) => {
                                    let re = vm.translate_expr(vmctx, &e).await;
                                    if let Ok(v) = re{
                                        vmctx.done();
                                        return v
                                    } else{
                                        check_err(&Err(re.err().unwrap()))
                                    }
                                },
                                BlockStmtOrExpr::BlockStmt(b) => {
                                    for stmt in &b.stmts{
                                        err = futures::executor::block_on(vm.translate_stmt(vmctx, stmt, None));

                                        if let Err(Error::Return(v)) = err{
                                            vmctx.done();
                                            return v;
                                        } else{
                                            check_err(&err)
                                        }
                                    }
                                }
                            }
                            vmctx.done();
                            JValue::Undefined
                        })).into()
                    })).into());

                } else{
                    todo!()
                }
                
            },

            Expr::Assign(a) => {
                let right = self.translate_expr(vmctx, &a.right).await?;
                match &a.op{
                    AssignOp::Assign => {
                        Ok(right)
                    },
                    _ => todo!()
                }
            },

            Expr::Await(a) => {
                let right = self.translate_expr(vmctx, &a.arg).await?;
                if let Some(o) = right.object(){
                    if let Some(i) = &o.inner{
                        if let Some(v) = i.downcast_ref::<Promise>(){
                            return Ok(v.await)
                        }
                    }
                }
                Ok(right)
            }

            _ => todo!()
        }
    }

    pub async fn translate_decl(&mut self, vmctx:&mut VmContext, decl:&Decl) -> Result<(), Error>{
        Ok(())
    }

    pub async fn translate_var_decl(&mut self, vmctx:&mut VmContext, decl:&VarDecl) -> Result<(), Error>{
        todo!()
    }

    pub async fn translate_pat(&mut self, vmctx:&mut VmContext, p:&Pat, value:JValue) -> Result<(), Error>{
        todo!()
    }
}

pub fn check_err(err:&Result<(), Error>){
    if let Err(e) = err{
        if let Error::Value(v) = e{
            operator::throw(*v)
        } else{
            panic_any(e.clone());
        }
    };
}