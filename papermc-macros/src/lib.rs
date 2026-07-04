use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Error, FnArg, Ident, ItemFn, LitStr, Token, Type};

struct TestArgs {
    ignored: bool,
    ignore_reason: Option<LitStr>,
}

impl Parse for TestArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = TestArgs {
            ignored: false,
            ignore_reason: None,
        };
        if input.is_empty() {
            return Ok(args);
        }
        let ident: Ident = input.parse()?;
        if ident != "ignore" {
            return Err(Error::new(
                ident.span(),
                "unsupported argument; expected `ignore` or `ignore = \"reason\"`",
            ));
        }
        args.ignored = true;
        if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            args.ignore_reason = Some(input.parse()?);
        }
        if !input.is_empty() {
            return Err(input.error("unexpected trailing arguments"));
        }
        Ok(args)
    }
}

/// Marks a function as an in-server test case.
///
/// The function keeps its original definition; the attribute additionally registers it in
/// `papermc::testing::TESTS` under the name `module_path!()::<fn name>`.
///
/// Accepted signatures: first parameter `&mut Api`, then zero or more fixture parameters
/// (`&T` where `T: TestFixture`), returning `()` or `eyre::Result<()>`.
///
/// `#[papermc::test(ignore)]` or `#[papermc::test(ignore = "reason")]` registers the test as
/// ignored: reported but not run unless `/test` is invoked with `--ignored` or
/// `--include-ignored`.
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(attr as TestArgs);
    let item = syn::parse_macro_input!(item as ItemFn);
    match expand(args, item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(args: TestArgs, item: ItemFn) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(param) = item.sig.generics.params.first() {
        return Err(Error::new_spanned(
            param,
            "test functions cannot be generic",
        ));
    }
    if let Some(asyncness) = &item.sig.asyncness {
        return Err(Error::new_spanned(
            asyncness,
            "test functions cannot be async",
        ));
    }

    let mut inputs = item.sig.inputs.iter();
    let first = inputs.next().ok_or_else(|| {
        Error::new_spanned(
            &item.sig,
            "test functions take `&mut Api` as their first parameter",
        )
    })?;
    let first_is_mut_ref = matches!(
        first,
        FnArg::Typed(pat) if matches!(&*pat.ty, Type::Reference(r) if r.mutability.is_some())
    );
    if !first_is_mut_ref {
        return Err(Error::new_spanned(
            first,
            "the first test parameter must be `&mut Api`",
        ));
    }

    let mut extractions = Vec::new();
    let mut call_args = Vec::new();
    for (i, arg) in inputs.enumerate() {
        let FnArg::Typed(pat) = arg else {
            return Err(Error::new_spanned(arg, "test functions cannot take `self`"));
        };
        let Type::Reference(reference) = &*pat.ty else {
            return Err(Error::new_spanned(
                &pat.ty,
                "fixture parameters must be `&T` references",
            ));
        };
        if reference.mutability.is_some() {
            return Err(Error::new_spanned(
                &pat.ty,
                "fixture parameters must be immutable `&T` references",
            ));
        }
        let fixture_ty = &reference.elem;
        let binding = format_ident!("__papermc_fixture_{i}");
        extractions.push(quote! {
            let #binding =
                match <#fixture_ty as ::papermc::testing::TestFixture>::extract(ctx) {
                    ::core::result::Result::Ok(::papermc::testing::Fixture::Present(value)) => {
                        value
                    }
                    ::core::result::Result::Ok(::papermc::testing::Fixture::Skip(reason)) => {
                        return ::papermc::testing::TestOutcome::Skipped(reason);
                    }
                    ::core::result::Result::Err(error) => {
                        return ::papermc::testing::TestOutcome::Failed(
                            ::std::format!("fixture extraction failed: {error:?}"),
                        );
                    }
                };
        });
        call_args.push(quote! { &#binding });
    }

    let fn_ident = &item.sig.ident;
    let fn_name = fn_ident.to_string();
    let shim_ident = format_ident!("__papermc_test_shim_{fn_name}");
    let static_ident = format_ident!("__PAPERMC_TEST_{}", fn_name.to_uppercase());
    let ignored = args.ignored;
    let ignore_reason = match &args.ignore_reason {
        Some(reason) => quote! { ::core::option::Option::Some(#reason) },
        None => quote! { ::core::option::Option::None },
    };

    Ok(quote! {
        #item

        fn #shim_ident(
            ctx: &mut ::papermc::testing::TestCtx<'_, '_>,
        ) -> ::papermc::testing::TestOutcome {
            #(#extractions)*
            ::papermc::testing::IntoOutcome::into_outcome(
                #fn_ident(&mut ctx.api #(, #call_args)*),
            )
        }

        #[::papermc::__private::linkme::distributed_slice(::papermc::testing::TESTS)]
        #[linkme(crate = ::papermc::__private::linkme)]
        static #static_ident: ::papermc::testing::TestCase = ::papermc::testing::TestCase {
            name: ::core::concat!(::core::module_path!(), "::", #fn_name),
            ignored: #ignored,
            ignore_reason: #ignore_reason,
            run: #shim_ident,
        };
    })
}
