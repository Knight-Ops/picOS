extern crate proc_macro;
use std::str::FromStr;

use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Colon, Comma, For, Impl, Struct};
use syn::{
    self, BareFnArg, ExprMethodCall, Field, FieldValue, FieldsNamed, FnArg, Ident, ItemFn,
    ItemImpl, ItemStruct, ParenthesizedGenericArguments, PathSegment, Type,
};

use proc_macro2::{Punct, Spacing, Span, TokenStream, TokenTree};

#[proc_macro_attribute]
pub fn task(_: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    let input_function: ItemFn = syn::parse(input).unwrap();

    // We are extracting our fields from the function input here so we can convert them to Fields of a struct
    let mut named_fields = Punctuated::new();
    input_function
        .sig
        .inputs
        .clone()
        .into_iter()
        .for_each(|x| match x {
            FnArg::Receiver(_) => panic!("No arguments should be untyped"),
            FnArg::Typed(pt) => {
                let field_name = match *pt.pat {
                    syn::Pat::Ident(pt_ident) => pt_ident.ident,
                    _ => panic!("The PatType pat field was not an Ident, not sure what to do here"),
                };

                let field = Field {
                    attrs: pt.attrs,
                    // This can be used if we need public structures, but we shouldn't
                    // vis: syn::Visibility::Restricted(syn::VisRestricted {
                    //     pub_token: syn::token::Pub::default(),
                    //     paren_token: syn::token::Paren::default(),
                    //     in_token: None,
                    //     path: Box::new(syn::Path::from(syn::PathSegment {
                    //         ident: Ident::new("crate", Span::call_site()),
                    //         arguments: syn::PathArguments::None,
                    //     })),
                    // }),
                    vis: syn::Visibility::Inherited,
                    colon_token: Some(pt.colon_token),
                    ty: *pt.ty,
                    ident: Some(field_name),
                };

                named_fields.push(field);
            }
        });

    // Place our extracted fields in a FieldsNamed we can use in a struct declaration
    let named_fields = FieldsNamed {
        brace_token: Brace::default(),
        named: named_fields,
    };

    // Build our struct using information we can get from the input function, the structure will be named based on the function name
    // with "Arguments" appended
    let matching_struct = ItemStruct {
        // We want to suppress errors from our generated names, so we need to add #[allow(non_camel_case_types)] to our struct definition
        attrs: vec![syn::Attribute {
            pound_token: syn::token::Pound::default(),
            style: syn::AttrStyle::Outer,
            bracket_token: syn::token::Bracket::default(),
            path: syn::Path {
                leading_colon: None,
                segments: Punctuated::new(),
            },
            tokens: TokenStream::from_str("allow(non_camel_case_types)").unwrap(),
        }],
        vis: input_function.vis.clone(),
        ident: Ident::new(
            &format!("_{}Arguments", input_function.sig.ident.to_string()),
            input_function.sig.ident.span(),
        ),
        generics: input_function.sig.generics.clone(),
        semi_token: None,
        fields: syn::Fields::Named(named_fields),
        struct_token: Struct::default(),
    };

    // Put our created structure into a token stream so we can emit it
    matching_struct.to_tokens(&mut output);

    // Clone the input function signature, since we need all the arguments anyways
    let mut new_fn_sig = input_function.sig.clone();
    new_fn_sig.constness = Some(syn::token::Const::default());
    new_fn_sig.ident = Ident::new("new", Span::call_site());
    new_fn_sig.output = syn::ReturnType::Type(
        syn::token::RArrow::default(),
        Box::new(syn::Type::Verbatim(TokenStream::from_str("Self").unwrap())),
    );

    // We need to retrieve the function args which are the same as the structure field names, we can use this shortcut to build
    // a function that doesn't need to know the names of the structure fields for the add_task macro
    let new_fn_args: Punctuated<FieldValue, Comma> = input_function
        .sig
        .inputs
        .clone()
        .into_iter()
        .map(|x| match x {
            FnArg::Typed(arg) => {
                let identifier = match *arg.pat {
                    syn::Pat::Ident(id) => id.ident,
                    _ => panic!("Found non-ident token in function inputs"),
                };
                FieldValue {
                    attrs: input_function.attrs.clone(),
                    member: syn::Member::Named(identifier),
                    colon_token: None,
                    // This is basically garbage since we are using shortcut definitions of the structure
                    expr: syn::Expr::Verbatim(TokenStream::from_str("").unwrap()),
                }
            }
            _ => panic!("Function should not take self for a task"),
        })
        .collect();

    // Build our new Impl for each created structure
    let new_impl = ItemImpl {
        attrs: input_function.attrs.clone(),
        defaultness: None,
        unsafety: None,
        impl_token: Impl::default(),
        generics: input_function.sig.generics.clone(),
        trait_: None,
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(PathSegment {
                ident: matching_struct.ident.clone(),
                arguments: syn::PathArguments::None,
            }),
        })),
        brace_token: Brace::default(),
        items: vec![syn::ImplItem::Method(syn::ImplItemMethod {
            attrs: input_function.attrs.clone(),
            vis: syn::Visibility::Public(syn::VisPublic {
                pub_token: syn::token::Pub::default(),
            }),
            defaultness: None,
            sig: new_fn_sig,
            block: syn::Block {
                brace_token: syn::token::Brace::default(),
                stmts: vec![syn::Stmt::Expr(syn::Expr::Struct(syn::ExprStruct {
                    attrs: input_function.attrs.clone(),
                    path: syn::Path::from(syn::PathSegment {
                        ident: Ident::new("Self", Span::call_site()),
                        arguments: syn::PathArguments::None,
                    }),
                    brace_token: syn::token::Brace::default(),
                    fields: new_fn_args,
                    dot2_token: None,
                    rest: None,
                }))],
            },
        })],
    };

    // Put our impl block in our token stream
    new_impl.to_tokens(&mut output);

    // Build our empty Impl of TaskArgument for each created structure
    let task_arg_impl = ItemImpl {
        attrs: input_function.attrs.clone(),
        defaultness: None,
        unsafety: None,
        impl_token: Impl::default(),
        generics: input_function.sig.generics.clone(),
        trait_: Some((
            None,
            syn::Path::from(PathSegment {
                ident: Ident::new("TaskArgument", Span::mixed_site()),
                arguments: syn::PathArguments::None,
            }),
            For::default(),
        )),
        self_ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path::from(PathSegment {
                ident: matching_struct.ident.clone(),
                arguments: syn::PathArguments::None,
            }),
        })),
        brace_token: Brace::default(),
        items: vec![],
    };

    // Put our impl block in our token stream
    task_arg_impl.to_tokens(&mut output);

    let mut modified_function = ItemFn {
        attrs: input_function.attrs,
        vis: input_function.vis.clone(),
        sig: input_function.sig.clone(),
        block: input_function.block,
    };

    // Create a new empty list of function args, then push our single structure to it, then add it to our function
    let mut function_arg = Punctuated::new();
    let mut path_segments = Punctuated::new();
    path_segments.push(syn::PathSegment {
        ident: Ident::new("alloc", Span::mixed_site()),
        arguments: syn::PathArguments::None,
    });
    path_segments.push(syn::PathSegment {
        ident: Ident::new("boxed", Span::mixed_site()),
        arguments: syn::PathArguments::None,
    });
    let mut box_args = Punctuated::new();
    box_args.push(syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
        qself: None,
        path: syn::Path::from(syn::PathSegment {
            ident: matching_struct.ident.clone(),
            arguments: syn::PathArguments::None,
        }),
    })));
    path_segments.push(syn::PathSegment {
        ident: Ident::new("Box", Span::mixed_site()),
        arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: syn::token::Lt::default(),
            args: box_args,
            gt_token: syn::token::Gt::default(),
        }),
    });
    function_arg.push(FnArg::Typed(syn::PatType {
        attrs: vec![],
        pat: Box::new(syn::Pat::Ident(syn::PatIdent {
            attrs: vec![],
            by_ref: None,
            mutability: None,
            ident: Ident::new("args", Span::mixed_site()),
            subpat: None,
        })),
        colon_token: Colon::default(),
        ty: Box::new(syn::Type::Path(syn::TypePath {
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: path_segments,
            },
        })),
    }));
    modified_function.sig.inputs = function_arg;

    // Finally we need to rebind the variables to a local context version that the function originally contained
    let mut rebind_statements = vec![];
    input_function.sig.inputs.into_iter().for_each(|x| match x {
        FnArg::Typed(pt) => {
            let full_pat = *pt.pat.clone();
            let pt_ident = match *pt.pat {
                syn::Pat::Ident(pt_ident) => pt_ident.ident.clone(),
                _ => panic!("FnArg typed value Pat is not an identifier which is invalid"),
            };

            rebind_statements.push(syn::Stmt::Local(syn::Local {
                attrs: vec![],
                let_token: syn::token::Let::default(),
                pat: full_pat,
                init: Some((
                    syn::token::Eq::default(),
                    Box::new(syn::Expr::Field(syn::ExprField {
                        attrs: vec![],
                        base: Box::new(syn::Expr::Path(syn::ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: syn::Path::from(PathSegment {
                                ident: Ident::new("args", Span::mixed_site()),
                                arguments: syn::PathArguments::None,
                            }),
                        })),
                        dot_token: syn::token::Dot::default(),
                        member: syn::Member::Named(pt_ident),
                    })),
                )),
                semi_token: syn::token::Semi::default(),
            }));
        }
        _ => panic!("Untyped function argument in input function is not acceptable"),
    });
    rebind_statements.append(&mut modified_function.block.stmts);
    modified_function.block.stmts = rebind_statements;

    modified_function.to_tokens(&mut output);

    // eprintln!("{}", output.to_string());

    output.into()
}

// Gonna have to fix this by emitting a `new` function when we make our structure, that way we don't have to know the exact fields of the struct
#[proc_macro]
pub fn add_task(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    let mut input_iter = input.into_iter();

    // Grab the local variable name of the scheduler
    let scheduler_ident = Ident::new(&input_iter.next().unwrap().to_string(), Span::call_site());
    // skip the ,
    input_iter.next().unwrap();
    // Get the task name literal
    let task_name = syn::Lit::Str(syn::LitStr::new(
        &input_iter.next().unwrap().to_string().trim_matches('"'),
        Span::call_site(),
    ));
    input_iter.next().unwrap();

    // Parse the function to get the function name
    let func: syn::ExprCall = syn::parse(input_iter.collect()).unwrap();
    let argument_structure = match *func.clone().func {
        syn::Expr::Path(path) => {
            format!("_{}Arguments", path.path.segments.last().unwrap().ident)
        }
        _ => panic!("Parsing the function argument structure failed while expanding add_task"),
    };
    let argument_path = match *func.clone().func {
        syn::Expr::Path(path) => path,
        _ => panic!("Parsing the function argument structure failed while expanding add_task"),
    };

    // Start building the argument structure::new such as _IdleArguments::new
    let mut segments = Punctuated::new();
    segments.push(syn::PathSegment {
        ident: Ident::new(&argument_structure, Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    segments.push(syn::PathSegment {
        ident: Ident::new("new", Span::call_site()),
        arguments: syn::PathArguments::None,
    });

    let mut task_new_args: Punctuated<syn::Expr, Comma> = Punctuated::new();
    // Push the task name
    task_new_args.push(syn::Expr::Lit(syn::ExprLit {
        attrs: vec![],
        lit: task_name,
    }));
    // Push the function pointer
    task_new_args.push(syn::Expr::Path(syn::ExprPath {
        attrs: vec![],
        qself: None,
        path: argument_path.path,
    }));
    // Push the new function call for the arguments
    task_new_args.push(syn::Expr::Call(syn::ExprCall {
        attrs: vec![],
        func: Box::new(syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments,
            },
        })),
        paren_token: syn::token::Paren::default(),
        args: func.args,
    }));

    // Start building the Task::new segment
    let mut task_segments = Punctuated::new();
    task_segments.push(syn::PathSegment {
        ident: Ident::new("Task", Span::call_site()),
        arguments: syn::PathArguments::None,
    });
    task_segments.push(syn::PathSegment {
        ident: Ident::new("new", Span::call_site()),
        arguments: syn::PathArguments::None,
    });

    // Build the arguments to Task::new
    let mut args: Punctuated<syn::Expr, Comma> = Punctuated::new();
    args.push(syn::Expr::Call(syn::ExprCall {
        attrs: vec![],
        func: Box::new(syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: syn::Path {
                leading_colon: None,
                segments: task_segments,
            },
        })),
        paren_token: syn::token::Paren::default(),
        args: task_new_args,
    }));

    // This is the single method call we actually need to emit in the form rougly of scheduler.add_task(Task::new(TASK_NAME, TASK_FUCTION_PTR, TASK_ARG_STRUCT))
    let method_call = ExprMethodCall {
        attrs: vec![],
        receiver: Box::new(syn::Expr::Path(syn::ExprPath {
            attrs: vec![],
            qself: None,
            path: syn::Path::from(syn::PathSegment {
                ident: scheduler_ident,
                arguments: syn::PathArguments::None,
            }),
        })),
        dot_token: syn::token::Dot::default(),
        method: Ident::new("add_task", Span::call_site()),
        turbofish: None,
        paren_token: syn::token::Paren::default(),
        args,
    };

    method_call.to_tokens(&mut output);

    eprintln!("{}", output.to_string());

    output.into()
}
