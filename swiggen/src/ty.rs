
struct Mapping {
    is_self: bool,
    
}

match arg {
    syn::FnArg::SelfRef(_) | syn::FnArg::SelfValue(_) => {
        // For self methods, we do some extra work to wrap the
        // function so that `impl Foo { fn bar(&self); }`
        // becomes `Foo_bar(wrapped_self: *const Foo)`.
        let wrapped_self = convert_self_type(&arg, self.base);
        args.push(wrapped_self.into_token_stream());

        let ws = syn::Ident::new("wrapped_self", Span::call_site());
        caller.push(ws.clone());
        caller_ref.push(quote!{@ref #ws});
    }
    syn::FnArg::Captured(ref ac) => {
        let id = match &ac.pat {
            syn::Pat::Ident(pi) => {
                &pi.ident
            },
            _ => unimplemented!(),
        };
        args.push(convert_arg_type(ac).into_token_stream());
        caller.push(id.clone());

        // this later calls the appropriate macro function as to
        // whether we need to do some pointer/box stuff
        if ac.ty.clone().into_token_stream().to_string().ends_with("str") {
            caller_ref.push(quote!{@str #id});
        } else if let syn::Type::Reference(_) = ac.ty {
            caller_ref.push(quote!{@ref #id});
        } else {
            caller_ref.push(quote!{@prim #id});
        }
    },
    _ => ()
}