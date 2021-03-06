use super::super::ir::hir::*;
use super::super::ir::ast::*;

//Hirを組み替えてカリー化を行う

impl ProgramHir {
    pub fn currying(mut self) -> ProgramHir {
        self.def_func_list =
            self.def_func_list
                .into_iter()
                .map(|(k, f)| (k, f.currying()))
                .collect();
        self
    }
}


impl DefFuncHir {
    fn currying(mut self) -> DefFuncHir {
        if self.params.len() == 0 {
            self.params.push(VariableAST { pos: self.pos, id: "_".to_string() });
        }
        let first_param = self.params[0].clone();
        self.body = self.body.currying(self.params.into_iter().skip(1), vec![first_param.clone()]);
        self.params = vec![first_param];
        self
    }
}

impl ExprAST {
    fn currying<I: Iterator<Item=VariableAST>>(self, mut iter: I, env: Vec<VariableAST>) -> ExprAST {
        match iter.next() {
            Some(v) => {
                let mut next_env = env.clone();
                next_env.push(v.clone());
                ExprAST::LambdaAST(Box::new(LambdaAST {
                    pos: self.get_pos(),
                    body: self.currying(iter, next_env),
                    env,
                    params: vec![v],
                }))
            }
            _ => match self {
                ExprAST::LambdaAST(x) => {
                    let mut x = *x;
                    x.body.currying(x.params.into_iter(), x.env)
                }
                ExprAST::TupleAST(mut x) => {
                    x.as_mut().elements = x.to_owned().elements.into_iter()
                        .map(|x| x.currying(vec![].into_iter(), vec![]))
                        .collect();
                    ExprAST::TupleAST(x)
                }
                ExprAST::FuncCallAST(x) => {
                    let mut x = *x;
                    x.func = x.func.currying(vec![].into_iter(), vec![]);
                    x.param = x.param.currying(vec![].into_iter(), vec![]);
                    ExprAST::FuncCallAST(Box::new(x))
                }
                ExprAST::OpAST(x) => {
                    let mut x = *x;
                    x.l_expr = x.l_expr.currying(vec![].into_iter(), vec![]);
                    x.r_expr = x.r_expr.currying(vec![].into_iter(), vec![]);
                    ExprAST::OpAST(Box::new(x))
                }
                ExprAST::ParenAST(x) => x.expr.currying(vec![].into_iter(), vec![]),
                ExprAST::IfAST(x) => {
                    let mut x = *x;
                    x.cond = x.cond.currying(vec![].into_iter(), vec![]);
                    x.t_expr = x.t_expr.currying(vec![].into_iter(), vec![]);
                    x.f_expr = x.f_expr.currying(vec![].into_iter(), vec![]);
                    ExprAST::IfAST(Box::new(x))
                }
                ExprAST::NamedParamsConstructorCallAST(_) => panic!("bug"),
                ExprAST::TupleStructAST(mut x) => {
                    x.tuple.elements = x.to_owned().tuple.elements.into_iter()
                        .map(|x| x.currying(vec![].into_iter(), vec![]))
                        .collect();
                    ExprAST::TupleStructAST(x)
                }
                ExprAST::IndexPropertyAST(mut x) => {
                    x.expr = x.to_owned().expr.currying(vec![].into_iter(), vec![]);
                    ExprAST::IndexPropertyAST(x)
                }
                ExprAST::NamePropertyAST(mut x) => {
                    x.expr = x.to_owned().expr.currying(vec![].into_iter(), vec![]);
                    ExprAST::NamePropertyAST(x)
                }
                ExprAST::NumAST(_) |
                ExprAST::BoolAST(_) |
                ExprAST::VariableAST(_) => self
            }
        }
    }
}
