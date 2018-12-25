use super::super::super::types::types::*;
use std::collections::HashMap;

type TypeSubstituteHashMap = HashMap<TypeId, Type>;

//TypeにTypeIdが出現するか検査
pub fn occurs_check(hash_map: &TypeSubstituteHashMap, ty: &Type, ty_id: &TypeId) -> bool {
    match ty {
        Type::TyVar(id, ref cond) => {
            (match cond.call {
                Some(ref x) =>
                    {
                        occurs_check(hash_map, &x.ret_type, &ty_id)
                            ||
                            x.param_types.iter().any(|x| occurs_check(hash_map, x, &ty_id))
                    }
                None => false
            })
                ||
                if id == ty_id { true } else {
                    if hash_map.contains_key(&id) {
                        occurs_check(hash_map, &hash_map[id], &ty_id)
                    } else {
                        false
                    }
                }
        }
        Type::Int32 | Type::Bool => false,
        Type::TupleType(x) => x.occurs_check(hash_map, ty_id),
        Type::LambdaType(x) => {
            let x = &**x;
            x.env_ty.as_ref().unwrap_or(&TupleType { element_tys: vec![] })
                .element_tys.iter().any(|e| occurs_check(hash_map, e, ty_id))
                ||
                x.func_ty.param_types.iter().any(|e| occurs_check(hash_map, e, ty_id))
                ||
                occurs_check(hash_map, &x.func_ty.ret_type, ty_id)
        }
        Type::StructType(x) => {
            match x.ty {
                StructInternalType::TupleType(ref x) => x.occurs_check(hash_map, ty_id),
                StructInternalType::RecordType(ref x) => x.element_tys.iter().any(|(_, e)| occurs_check(hash_map, e, ty_id))
            }
        }
    }
}

impl TupleType {
    fn occurs_check(&self, hash_map: &TypeSubstituteHashMap, ty_id: &TypeId) -> bool {
        self.element_tys.iter().any(|e| occurs_check(hash_map, e, ty_id))
    }
}