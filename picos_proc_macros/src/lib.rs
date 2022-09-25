extern crate proc_macro;
use std::str::FromStr;

use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Colon, Comma, For, Impl, Struct};
use syn::{
    self, BareFnArg, Field, FieldsNamed, FnArg, Ident, ItemFn, ItemImpl, ItemStruct,
    ParenthesizedGenericArguments, PathSegment, Type,
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

    eprintln!("{}", output.to_string());

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
        &input_iter.next().unwrap().to_string(),
        Span::call_site(),
    ));
    input_iter.next().unwrap();

    let rest: proc_macro::TokenStream = input_iter.collect();

    let func: syn::ExprCall = syn::parse(rest).unwrap();

    eprintln!("{:?}", func);

    output.into()
}
