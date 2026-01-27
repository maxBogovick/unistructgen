use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Meta, Lit, Expr};
use syn::punctuated::Punctuated;
use syn::Token;

pub fn impl_into_ir(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let name_str = name.to_string();

    let fields_code = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let recurse = fields.named.iter().map(|f| {
                        let field_name = &f.ident;
                        let field_name_str = field_name.as_ref().unwrap().to_string();
                        let field_type = &f.ty;

                        // Parse attributes
                        let mut doc = quote! { None };
                        let mut min_len = quote! { None };
                        let mut max_len = quote! { None };
                        let mut min_val = quote! { None };
                        let mut max_val = quote! { None };
                        let mut pattern = quote! { None };
                        let mut format = quote! { None };
                        let mut optional_override = false;
                        let mut default_val = quote! { None };

                        // Extract doc comments
                        let mut doc_strings = Vec::new();
                        for attr in &f.attrs {
                            if attr.path().is_ident("doc") {
                                if let Meta::NameValue(meta) = &attr.meta {
                                    if let Expr::Lit(expr_lit) = &meta.value {
                                        if let Lit::Str(lit) = &expr_lit.lit {
                                            doc_strings.push(lit.value().trim().to_string());
                                        }
                                    }
                                }
                            } else if attr.path().is_ident("field") {
                                // Parse nested meta items
                                if let Ok(nested_metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
                                    for nested in nested_metas {
                                        match nested {
                                            Meta::NameValue(nv) => {
                                                if let Expr::Lit(expr_lit) = &nv.value {
                                                    if nv.path.is_ident("min_length") {
                                                        if let Lit::Int(lit) = &expr_lit.lit {
                                                            let val: usize = lit.base10_parse().unwrap();
                                                            min_len = quote! { Some(#val) };
                                                        }
                                                    } else if nv.path.is_ident("max_length") {
                                                        if let Lit::Int(lit) = &expr_lit.lit {
                                                            let val: usize = lit.base10_parse().unwrap();
                                                            max_len = quote! { Some(#val) };
                                                        }
                                                    } else if nv.path.is_ident("min_value") {
                                                         if let Lit::Int(lit) = &expr_lit.lit {
                                                            let val: f64 = lit.base10_parse::<i64>().unwrap() as f64;
                                                            min_val = quote! { Some(#val) };
                                                        } else if let Lit::Float(lit) = &expr_lit.lit {
                                                             let val: f64 = lit.base10_parse().unwrap();
                                                             min_val = quote! { Some(#val) };
                                                        }
                                                    } else if nv.path.is_ident("max_value") {
                                                         if let Lit::Int(lit) = &expr_lit.lit {
                                                            let val: f64 = lit.base10_parse::<i64>().unwrap() as f64;
                                                            max_val = quote! { Some(#val) };
                                                        } else if let Lit::Float(lit) = &expr_lit.lit {
                                                             let val: f64 = lit.base10_parse().unwrap();
                                                             max_val = quote! { Some(#val) };
                                                        }
                                                    } else if nv.path.is_ident("pattern") {
                                                        if let Lit::Str(lit) = &expr_lit.lit {
                                                            let val = lit.value();
                                                            pattern = quote! { Some(#val.to_string()) };
                                                        }
                                                    } else if nv.path.is_ident("format") {
                                                        if let Lit::Str(lit) = &expr_lit.lit {
                                                            let val = lit.value();
                                                            format = quote! { Some(#val.to_string()) };
                                                        }
                                                    } else if nv.path.is_ident("doc") {
                                                        if let Lit::Str(lit) = &expr_lit.lit {
                                                            doc_strings.push(lit.value());
                                                        }
                                                    } else if nv.path.is_ident("default") {
                                                         if let Lit::Str(lit) = &expr_lit.lit {
                                                            let val = lit.value();
                                                            default_val = quote! { Some(#val.to_string()) };
                                                        }
                                                    }
                                                }
                                            },
                                            Meta::Path(path) => {
                                                if path.is_ident("optional") {
                                                    optional_override = true;
                                                }
                                            },
                                            _ => {} 
                                        }
                                    }
                                }
                            }
                        }

                        if !doc_strings.is_empty() {
                            let doc_joined = doc_strings.join("\n");
                            doc = quote! { Some(#doc_joined.to_string()) };
                        }

                        quote! {
                            {
                                let ty_ref = <#field_type as unistructgen_core::IntoIR>::ir_type_ref();
                                let is_optional_type = ty_ref.is_optional();
                                let final_ty = if #optional_override && !is_optional_type {
                                    ty_ref.make_optional()
                                } else {
                                    ty_ref
                                };

                                let constraints = unistructgen_core::ir::FieldConstraints {
                                    min_length: #min_len,
                                    max_length: #max_len,
                                    min_value: #min_val,
                                    max_value: #max_val,
                                    pattern: #pattern,
                                    format: #format,
                                };

                                let mut field = unistructgen_core::ir::IRField::new(
                                    #field_name_str.to_string(),
                                    final_ty,
                                );
                                field.doc = #doc;
                                field.constraints = constraints;
                                field.optional = is_optional_type || #optional_override;
                                field.default = #default_val;
                                fields.push(field);
                            }
                        }
                    });
                    quote! {
                        let mut fields = Vec::new();
                        #(#recurse)*
                    }
                }
                Fields::Unnamed(_) => {
                    // Tuple structs not supported yet in this simplified version
                    quote! { let fields = Vec::new(); }
                }
                Fields::Unit => {
                    quote! { let fields = Vec::new(); }
                }
            }
        }
        Data::Enum(_) => {
             quote! {
                 // Enum not implemented yet for IntoIR derive
                 return None;
             }
        }
        Data::Union(_) => {
            quote! {
                return None;
            }
        }
    };

    let struct_doc = {
        let mut doc_strings = Vec::new();
        for attr in &input.attrs {
            if attr.path().is_ident("doc") {
                if let Meta::NameValue(meta) = &attr.meta {
                    if let Expr::Lit(expr_lit) = &meta.value {
                        if let Lit::Str(lit) = &expr_lit.lit {
                            doc_strings.push(lit.value().trim().to_string());
                        }
                    }
                }
            }
        }
        if doc_strings.is_empty() {
            quote! { None }
        } else {
            let doc_joined = doc_strings.join("\n");
            quote! { Some(#doc_joined.to_string()) }
        }
    };

    let definition_body = match input.data {
        Data::Struct(_) => {
             quote! {
                #fields_code
                let mut s = unistructgen_core::ir::IRStruct::new(#name_str.to_string());
                s.fields = fields;
                s.doc = #struct_doc;
                Some(unistructgen_core::ir::IRType::Struct(s))
            }
        }
        _ => quote! { None }
    };

    let expanded = quote! {
        impl unistructgen_core::IntoIR for #name {
            fn ir_type_ref() -> unistructgen_core::ir::IRTypeRef {
                unistructgen_core::ir::IRTypeRef::Named(#name_str.to_string())
            }

            fn ir_definition() -> Option<unistructgen_core::ir::IRType> {
                #definition_body
            }
        }
    };

    TokenStream::from(expanded)
}