use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, Type, PathArguments, GenericArgument, ReturnType};
use unistructgen_core::{IRField, IRTypeRef, PrimitiveKind, IRStruct, IRModule, IRType, CodeGenerator};
use unistructgen_codegen::JsonSchemaRenderer;

pub fn ai_tool_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let tool_name = fn_name.to_string();
    
    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();
    
    // Check return type to see if it is a Result
    let is_result = if let ReturnType::Type(_, ty) = &input_fn.sig.output {
        is_result_type(ty)
    } else {
        false
    };

    // 1. Extract Description from Doc Comments
    let mut description = String::new();
    for attr in &input_fn.attrs {
        if attr.path().is_ident("doc") {
            if let syn::Expr::Lit(expr_lit) = &attr.meta.require_name_value().unwrap().value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                    let doc_line = lit_str.value();
                    if !description.is_empty() {
                        description.push('\n');
                    }
                    description.push_str(doc_line.trim());
                }
            }
        }
    }
    if description.is_empty() {
        description = format!("Tool for {}", tool_name);
    }

    // 2. Parse Arguments into IR for Schema Generation
    let mut ir_struct = IRStruct::new(format!("{}Args", tool_name));
    let mut args_struct_fields = Vec::new(); // For generating the internal struct definition
    let mut call_args = Vec::new(); // For calling the original function

    for input in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let arg_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.clone()
            } else {
                continue; // Skip self or complex patterns
            };
            
            let arg_name_str = arg_name.to_string();
            let ty = &*pat_type.ty;
            
            // Map syn::Type to IRTypeRef (Simplified mapping)
            let ir_type = map_syn_type_to_ir(ty);
            
            ir_struct.add_field(IRField::new(arg_name_str.clone(), ir_type));
            
            args_struct_fields.push(quote! { pub #arg_name: #ty });
            call_args.push(quote! { args.#arg_name });
        }
    }

    // 3. Generate JSON Schema
    let mut module = IRModule::new("tool_schema".to_string());
    module.add_type(IRType::Struct(ir_struct));
    
    let renderer = JsonSchemaRenderer::new().fragment();
    let schema_json_str = renderer.generate(&module).expect("Failed to generate schema");
    
    let pascal_tool_name = to_pascal_case(&tool_name);
    let tool_struct_name = format_ident!("{}Tool", pascal_tool_name);
    let struct_name_ident = format_ident!("{}Args", tool_name); 

    // Generate call logic
    let fn_call = if is_async {
        quote! { #fn_name(#(#call_args),*).await }
    } else {
        quote! { #fn_name(#(#call_args),*) }
    };

    let result_handling = if is_result {
        quote! {
            match result {
                Ok(val) => Ok(format!("{:?}", val)),
                Err(err) => Err(unistructgen_core::ToolError::ExecutionError(format!("{:?}", err))),
            }
        }
    } else {
        quote! {
            Ok(format!("{:?}", result))
        }
    };

    // Reconstruct the original function + Tool implementation
    let output = quote! {
        #input_fn

        #[allow(non_camel_case_types)]
        pub struct #tool_struct_name;

        #[derive(serde::Deserialize)]
        struct #struct_name_ident {
            #(#args_struct_fields),*
        }

        #[unistructgen_core::async_trait]
        impl unistructgen_core::AiTool for #tool_struct_name {
            fn name(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #description
            }

            fn parameters_schema(&self) -> serde_json::Value {
                serde_json::from_str(#schema_json_str).unwrap()
            }

            async fn call(&self, arguments_json: &str) -> unistructgen_core::ToolResult {
                let args: #struct_name_ident = serde_json::from_str(arguments_json)
                    .map_err(unistructgen_core::ToolError::ArgumentError)?;
                
                let result = #fn_call;
                #result_handling
            }
        }
    };

    output.into()
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn map_syn_type_to_ir(ty: &Type) -> IRTypeRef {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let ident_str = segment.ident.to_string();
            match ident_str.as_str() {
                "String" | "str" => IRTypeRef::Primitive(PrimitiveKind::String),
                "i32" | "i16" | "i8" => IRTypeRef::Primitive(PrimitiveKind::I32),
                "i64" | "isize" => IRTypeRef::Primitive(PrimitiveKind::I64),
                "f32" | "f64" => IRTypeRef::Primitive(PrimitiveKind::F64),
                "bool" => IRTypeRef::Primitive(PrimitiveKind::Bool),
                "Vec" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return IRTypeRef::Vec(Box::new(map_syn_type_to_ir(inner_ty)));
                        }
                    }
                    IRTypeRef::Vec(Box::new(IRTypeRef::Primitive(PrimitiveKind::String))) // Fallback
                }
                "Option" => {
                    if let PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                            return IRTypeRef::Option(Box::new(map_syn_type_to_ir(inner_ty)));
                        }
                    }
                    IRTypeRef::Option(Box::new(IRTypeRef::Primitive(PrimitiveKind::String))) // Fallback
                }
                _ => IRTypeRef::Primitive(PrimitiveKind::String), // Fallback for unknown types
            }
        } else {
             IRTypeRef::Primitive(PrimitiveKind::String)
        }
    } else {
        IRTypeRef::Primitive(PrimitiveKind::String)
    }
}

fn is_result_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Result";
        }
    }
    false
}