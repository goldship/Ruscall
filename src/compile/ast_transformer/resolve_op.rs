use super::super::ir::ast::*;
use super::super::ir::hir::*;
use super::super::Error;
use std::collections::HashMap;

type InfixHash = HashMap<String, InfixAST>;
type ResolveResult<T> = Result<T, Error>;

impl ProgramHir {
    //OpASTをinfixの定義によって優先順位を置き換えたProgramASTを得る
    pub fn resolve_op(mut self) -> ResolveResult<ProgramHir> {
        let mut infix_list = self.infix_list;
        self.def_func_list =
            self.def_func_list.into_iter()
                .map(|(k, f)| Ok((k, f.resolve_op(&mut infix_list)?)))
                .collect::<ResolveResult<HashMap<String, DefFuncAST>>>()?;
        self.infix_list = infix_list;

        Ok(self)
    }
}

impl DefFuncAST {
    fn resolve_op(self, infix_hash: &InfixHash) -> ResolveResult<DefFuncAST> {
        Result::Ok(DefFuncAST {
            body: self.body.resolve_op(infix_hash)?.get_expr_ast(),
            ..self
        })
    }
}

enum Resolved {
    OtherExprAST(ExprAST),
    OpAST(OpAST, InfixAST),
}

impl Resolved {
    fn get_expr_ast(self) -> ExprAST {
        match self {
            Resolved::OpAST(x, _) => ExprAST::OpAST(Box::new(x)),
            Resolved::OtherExprAST(x) => x,
        }
    }
}

impl InfixAST {
    //自分より左にある演算子と比べて優先順位が高かったらtrue
    fn is_priority_greater(&self, child: &InfixAST) -> bool {
        if self.priority > child.priority {
            return true;
        }
        if self.priority == child.priority && self.ty == child.ty {
            return match self.ty {
                InfixType::Left => false,
                InfixType::Right => true,
            };
        }
        return false;
    }
}

impl OpAST {
    fn swap_op(mut self, infix_hash: &InfixHash) -> ResolveResult<Resolved> {
        let self_infix = match infix_hash.get(&self.op) {
            None => return Result::Err(Error::new(self.pos, "no declare op")),
            Some(x) => x.clone(),
        };

        let resolved = match self.l_expr.resolve_op(infix_hash)? {
            Resolved::OpAST(mut child_op_ast, child_infix) => {
                if self_infix.is_priority_greater(&child_infix) {
                    self.l_expr = child_op_ast.r_expr;
                    child_op_ast.r_expr = ExprAST::OpAST(Box::new(self))
                        .resolve_op(infix_hash)?
                        .get_expr_ast();
                    Resolved::OpAST(child_op_ast, child_infix)
                } else {
                    self.l_expr = ExprAST::OpAST(Box::new(child_op_ast));
                    Resolved::OpAST(self, self_infix)
                }
            }
            x => {
                self.l_expr = x.get_expr_ast();
                self.r_expr = self.r_expr.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OpAST(self, self_infix)
            }
        };
        Ok(resolved)
    }
}

impl ExprAST {
    fn resolve_op(self, infix_hash: &InfixHash) -> ResolveResult<Resolved> {
        use super::super::ir::ast::*;
        let resolved = match self {
            ExprAST::OpAST(op_ast) => op_ast.swap_op(infix_hash)?,
            ExprAST::ParenAST(paren_ast) => {
                Resolved::OtherExprAST(ExprAST::create_paren_ast(
                    paren_ast.expr.resolve_op(infix_hash)?.get_expr_ast(),
                ))
            }
            ExprAST::FuncCallAST(x) => {
                let mut x = *x;
                x.param = x.param.resolve_op(infix_hash).map(|e| e.get_expr_ast())?;
                x.func = x.func.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OtherExprAST(ExprAST::FuncCallAST(Box::new(x)))
            }
            ExprAST::LambdaAST(x) => {
                let mut x = *x;
                x.body = x.body.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OtherExprAST(ExprAST::LambdaAST(Box::new(x)))
            }
            ExprAST::TupleAST(x) => {
                let mut x = *x;
                x.elements =
                    x.elements.into_iter()
                        .map(|e| e.resolve_op(infix_hash).map(|e| e.get_expr_ast()))
                        .collect::<ResolveResult<Vec<ExprAST>>>()?;
                Resolved::OtherExprAST(ExprAST::TupleAST(Box::new(x)))
            }
            ExprAST::IfAST(x) => {
                let mut x = *x;
                x.t_expr = x.t_expr.resolve_op(infix_hash)?.get_expr_ast();
                x.f_expr = x.f_expr.resolve_op(infix_hash)?.get_expr_ast();
                x.cond = x.cond.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OtherExprAST(ExprAST::IfAST(Box::new(x)))
            }
            ExprAST::NamedParamsConstructorCallAST(x) => {
                let mut x = *x;
                x.params = x.params.into_iter()
                    .map(|(name, e)|
                        Ok((name, e.resolve_op(infix_hash)?.get_expr_ast()))
                    )
                    .collect::<ResolveResult<Vec<(String, ExprAST)>>>()?;
                Resolved::OtherExprAST(ExprAST::NamedParamsConstructorCallAST(Box::new(x)))
            }
            ExprAST::IndexPropertyAST(x) => {
                let mut x = *x;
                x.expr = x.expr.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OtherExprAST(ExprAST::IndexPropertyAST(Box::new(x)))
            }
            ExprAST::NamePropertyAST(x) => {
                let mut x = *x;
                x.expr = x.expr.resolve_op(infix_hash)?.get_expr_ast();
                Resolved::OtherExprAST(ExprAST::NamePropertyAST(Box::new(x)))
            }
            ExprAST::NumAST(_) | ExprAST::BoolAST(_) | ExprAST::VariableAST(_) => Resolved::OtherExprAST(self),
            _ => panic!("undefined")
        };
        Ok(resolved)
    }
}
