use super::super::my_llvm::easy::*;
use super::ast;

fn extern_test_func(module: &Module) -> Function {
    let function_type = function_type(void_type(), vec![int32_type()]);
    let function = Function::new("print", &module, function_type);
    set_linkage(function.llvm_function, LLVMLinkage::LLVMExternalLinkage);
    function
}

//コード生成する関数
impl ast::ProgramAST {
    pub fn code_gen(self, file_name: &str) {
        //llvm初期化
        init_llvm_all_target();
        let codegen = CodeGenerator::new();
        let module = Module::new("my_module");

        let print_func = extern_test_func(&module);

        self.stmt_list.into_iter().for_each(|stmt| match stmt {
            ast::StmtAST::DefFuncAST(def_func_ast) => {
                gen_def_func(def_func_ast, &print_func, &module, &codegen)
            }
            _ => (),
        });
        if let Some(err_msg) = module.verify_module() {
            panic!("llvm error:{}", err_msg);
        }
        module.dump_module();

        output_file(file_name, module, codegen);
    }
}

fn gen_def_func(
    def_func_ast: ast::DefFuncAST,
    print_func: &Function,
    module: &Module,
    codegen: &CodeGenerator,
) {
    let function_type = function_type(
        int32_type(),
        def_func_ast
            .params
            .iter()
            .map(|_| int32_type())
            .collect(),
    );
    let function = Function::new(&def_func_ast.func_name, &module, function_type);
    function.get_params(def_func_ast.params.len()).
        into_iter().
        zip(def_func_ast.params.into_iter()).
        for_each(|(val_ref, variable_ast)|
            set_value_name(val_ref, &variable_ast.id));
    let entry_block = function.append_basic_block("entry");
    codegen.position_builder_at_end(entry_block);
    let value = gen_expr(def_func_ast.body, &module, &codegen);
    codegen.build_call(print_func.llvm_function, vec![value], "");
    codegen.build_ret(const_int(int32_type(), 0, false));
}

//四則演算のコード生成
fn gen_expr(expr_ast: ast::ExprAST, module: &Module, codegen: &CodeGenerator) -> LLVMValueRef {
    match expr_ast {
        ast::ExprAST::NumAST(num_ast) => const_int(int32_type(), num_ast.num as u64, true),
        ast::ExprAST::OpAST(op_ast) => {
            let op_ast: ast::OpAST = *op_ast;
            let lhs = gen_expr(op_ast.l_expr, module, codegen);
            let rhs = gen_expr(op_ast.r_expr, module, codegen);
            match &op_ast.op as &str {
                "+" => codegen.build_add(lhs, rhs, ""),
                "-" => codegen.build_sub(lhs, rhs, ""),
                "*" => codegen.build_mul(lhs, rhs, ""),
                "/" => {
                    let lhs = codegen.build_si_to_fp(lhs, double_type(), "");
                    let rhs = codegen.build_si_to_fp(rhs, double_type(), "");
                    codegen.build_fp_to_si(codegen.build_fdiv(lhs, rhs, ""), int32_type(), "")
                }
                _ => panic!("error"),
            }
        }
        ast::ExprAST::ParenAST(paren_ast) => gen_expr(paren_ast.expr, module, codegen),
        _ => const_int(int32_type(), 0, true),
    }
}

//コードをexeで出力
fn output_file(file_name: &str, module: Module, codegen: CodeGenerator) {
    use std::env;
    use std::process::Command;
    let target_machine = TargetMachine::create(
        "generic",
        "",
        LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
        LLVMRelocMode::LLVMRelocDefault,
        LLVMCodeModel::LLVMCodeModelDefault,
    ).unwrap();
    module.set_data_layout(target_machine.create_data_layout());
    module.set_target_triple(target_machine.target_triple);
    module.write_bitcode_to_file(&(file_name.to_string() + ".bc"));
    target_machine.emit_to_file(
        &module,
        &(file_name.to_string() + ".obj"),
        LLVMCodeGenFileType::LLVMObjectFile,
    );
    module.dispose_module();
    codegen.dispose();
    target_machine.dispose();

    let current_dir = env::current_dir().unwrap().to_str().unwrap().to_string();
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(&[
                "/C",
                &(current_dir.clone() + "\\compile.bat"),
                &(current_dir + "\\" + file_name + ".obj"),
            ])
            .output()
            .expect("failed to execute process")
    } else {
        panic!("support windows only!!")
    };
}
