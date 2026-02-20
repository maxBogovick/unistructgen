use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, ItemFn, FnArg, Pat, Type, PathArguments, GenericArgument, ReturnType};
use unistructgen::core::{IRField, IRTypeRef, PrimitiveKind, IRStruct, IRModule, IRType, CodeGenerator};
use unistructgen::codegen::JsonSchemaRenderer;

pub fn ai_tool_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let tool_name = fn_name.to_string();
    
    // Check if function is async
    let is_async = input_fn.sig.asyncness.is_some();
    
    // Check return type
    let is_result = if let ReturnType::Type(_, ty) = &input_fn.sig.output {
        is_result_type(ty)
    } else {
        false
    };

    // 1. Extract Description
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

    // 2. Parse Arguments
    let mut ir_struct = IRStruct::new(format!("{}Args", tool_name));
    let mut args_struct_fields = Vec::new(); 
    let mut call_args = Vec::new();
    let mut context_extractions = Vec::new();

    for input in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let arg_name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
                pat_ident.ident.clone()
            } else {
                continue; 
            };
            
            let arg_name_str = arg_name.to_string();
            let ty = &*pat_type.ty;

            // Check for #[context] attribute
            // Note: attributes on function arguments are allowed in Rust but often unused.
            // We use them as markers.
            let is_context = pat_type.attrs.iter().any(|a| a.path().is_ident("context"));

            if is_context {
                // Dependency Injection logic
                // The type 'ty' is likely a reference like &DbPool or just DbPool.
                // Our Context stores Arc<dyn Any>, so we might need to handle cloning or references.
                // Context::get<T>() returns Option<&T>.
                
                // Handling Reference Types vs Owned Types:
                // If fn expects &T, we can pass reference from Context.
                // If fn expects T, we must Clone it from Context reference.
                
                // Simplify: assume user asks for T and T is Clone + Send + Sync + 'static.
                // Or user asks for Arc<T>.
                
                // For this macro implementation, let's assume we extract `T` via cloning.
                // Code: let arg_name = context.require::<T>().map_err(...).cloned()?;
                
                // But wait, if ty is &T, we can't return reference to local variable easily in async call?
                // Context is passed as reference. So we can return reference if lifetime allows.
                // But `call` is async, so lifetimes are tricky.
                // Safest bet: Clone.
                
                // We need to strip reference syntax from type for TypeId lookup if it is &T?
                // Context stores T. `get` returns &T.
                
                // Let's rely on the user asking for `Type` (owned/cloned) for now to keep macro simple.
                // Example: fn my_tool(#[context] db: DbPool) -> ... where DbPool is essentially Arc.
                
                context_extractions.push(quote! {
                    let #arg_name = {
                        let dep = context.require::<#ty>()
                            .map_err(|e| unistructgen::core::ToolError::ContextError(e))?;
                        dep.clone()
                    };
                });
                
                call_args.push(quote! { #arg_name });
                
            } else {
                // Normal JSON Argument
                let ir_type = map_syn_type_to_ir(ty);
                ir_struct.add_field(IRField::new(arg_name_str.clone(), ir_type));
                
                args_struct_fields.push(quote! { pub #arg_name: #ty });
                call_args.push(quote! { args.#arg_name });
            }
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

    let fn_call = if is_async {
        quote! { #fn_name(#(#call_args),*).await }
    } else {
        quote! { #fn_name(#(#call_args),*) }
    };

    let result_handling = if is_result {
        quote! {
            match result {
                Ok(val) => Ok(format!("{:?}", val)),
                Err(err) => Err(unistructgen::core::ToolError::ExecutionError(format!("{:?}", err))),
            }
        }
    } else {
        quote! {
            Ok(format!("{:?}", result))
        }
    };

    // Note: We need to filter out #[context] from the input function definition in the output?
    // Rust compiler might complain about #[context] if it's not a registered attribute macro or helper.
    // proc_macro_attribute usually consumes the item.
    // We output `#input_fn` which contains the attributes.
    // If we define `context` as a helper attribute, it might work?
    // Actually, since we consume the function, we should ideally strip the `#[context]` attributes from the output function definition
    // because `context` isn't a real attribute unless we define it.
    // Simpler: Just define a dummy `#[context]` macro or ignore the warning if Rust allows unknown attributes on fn args (it usually errors).
    
    // Better strategy: Strip attributes from input_fn before quoting it back.
    let mut output_fn = input_fn.clone();
    for input in &mut output_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            pat_type.attrs.retain(|a| !a.path().is_ident("context"));
        }
    }

    let output = quote! {
        #output_fn

        #[allow(non_camel_case_types)]
        pub struct #tool_struct_name;

        #[derive(serde::Deserialize)]
        struct #struct_name_ident {
            #(#args_struct_fields),*
        }

        #[unistructgen::core::async_trait]
        impl unistructgen::core::AiTool for #tool_struct_name {
            fn name(&self) -> &str {
                #tool_name
            }

            fn description(&self) -> &str {
                #description
            }

            fn parameters_schema(&self) -> serde_json::Value {
                serde_json::from_str(#schema_json_str).unwrap()
            }

            async fn call(&self, arguments_json: &str, context: &unistructgen::core::Context) -> unistructgen::core::ToolResult {
                // 1. Extract context dependencies
                #(#context_extractions)*
                
                // 2. Parse JSON arguments
                let args: #struct_name_ident = serde_json::from_str(arguments_json)
                    .map_err(unistructgen::core::ToolError::ArgumentError)?;
                
                // 3. Call function
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
