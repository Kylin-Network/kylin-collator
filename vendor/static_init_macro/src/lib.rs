// Copyright 2021 Olivier Kannengieser
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

///! Macros for static_init crate.
extern crate proc_macro;
extern crate syn;
use syn::spanned::Spanned;
use syn::*;

use core::result::Result;

extern crate quote;
use quote::{quote, quote_spanned};

use proc_macro::TokenStream;

extern crate proc_macro2;
use proc_macro2::{Span, TokenStream as TokenStream2};

macro_rules! ok_or_return {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(t) => return t.into(),
        }
    };
}

/// Attribute for functions run at program initialization (before main).
///
/// ```ignore
/// #[constructor]
/// extern "C" fn initer () {
/// // run before main start
/// }
/// ```
/// The execution order of constructors is unspecified. Nevertheless on ELF plateform (linux, any unixes but mac) and
/// windows plateform a priority can be specified using the syntax `constructor(<num>)` where
/// `<num>` is a number included in the range [0 ; 2<sup>16</sup>-1].
///
/// Constructors with a priority of 65535 are run first (in unspecified order), then constructors
/// with priority 65534 are run ...  then constructors
/// with priority number 0 
///
/// An abscence of priority is equivalent to a priority of 0.
///
/// # Safety
///
/// Any access to [macro@dynamic] statics with an equal or lower
/// initialization priority will cause undefined behavior. (NB: usual static data initialized
/// by a const expression are always in an initialized state so it is always safe to read them).
///
/// Notably, on Elf gnu variant platforms, accesses to the program argument or environment through `std::env::*` functionalities
/// with a priority 65535-100 will cause undefined behavior. On windows thoses accesses `std::env::*` will never cause
/// undefined behavior. On other plateforms (non gnu variant of unixes and mac), any access to
/// `std::env::*` in a constructor, whatever its priority, will cause undefined behavior. In this
/// last case, the information may be accessible in the /proc/self directory.
///
/// ```ignore
/// #[constructor(0)]
/// extern "C" fn first () {
/// // run before main start
/// }
///
/// #[constructor(1)]
/// extern "C" fn then () {
/// // run before main start
/// }
/// ```
///
/// NB: Whatever the priority, constructors are run after initialization of libc resources. C++ static
/// objects are initialized as constructors with no priorities. On ELF plateform, libstdc++
/// resources are initialized with priority 65535-100.
///
/// # Constructor signature
///
/// Constructor function should have type `extern "C" fn() -> ()`.
///
/// But on plateform where the program is linked
/// with the gnu variant of libc (which covers all gnu variant platforms) constructor functions
/// can take (or not) `argc: i32, argv: **const u8, env: **const u8` arguments.
/// `argc` is the size of the argv
/// sequence, `argv` and `env` both refer to null terminated contiguous sequence of pointer
/// to c-string (c-strings are null terminated sequence of u8).
/// Cf "glibc source"/csu/elf-init.c, and System V ABI.
#[proc_macro_attribute]
pub fn constructor(args: TokenStream, input: TokenStream) -> TokenStream {
    let priority = ok_or_return!(parse_priority(args));

    let section = ok_or_return!(init_section(priority));

    let func: ItemFn = parse_macro_input!(input);

    let func_ptr_name = format!("__static_init_constructor_{}", func.sig.ident);

    let func_type = get_init_func_sig(&func.sig);

    gen_ctor_dtor(func, &section, &func_ptr_name, func_type).into()
}

fn get_init_func_sig(sig: &Signature) -> TypeBareFn {
    let sp = sig.span();

    if cfg!(target_env = "gnu") && cfg!(target_family = "unix") && !sig.inputs.is_empty() {
        parse2(quote_spanned!(sp.span()=>extern "C" fn(i32,*const*const u8, *const *const u8)))
            .unwrap()
    } else {
        parse2(quote_spanned!(sp.span()=>extern "C" fn())).unwrap()
    }
}

fn const_dtor_no_support() -> TokenStream {
    quote!(compile_error!(
        "program constructors/destructors not supported on this target"
    ))
    .into()
}

fn init_section(priority: u16) -> Result<String, TokenStream> {
    if cfg!(elf) {
        Ok(format!(".init_array.{:05}", 65535 - priority))
    } else if cfg!(mach_o) {
        if priority != 0 {
            Err(quote!(compile_error!(
                "Constructor priority other than 0 not supported on this plateform."
            ))
            .into())
        } else {
            Ok("__DATA,__mod_init_func".to_string())
        }
    } else if cfg!(coff) {
        Ok(format!(".CRT$XCTZ{:05}", 65535 - priority))
    } else {
        Err(const_dtor_no_support())
    }
}

fn fini_section(priority: u16) -> Result<String, TokenStream> {
    if cfg!(elf) {
        Ok(format!(".fini_array.{:05}", 65535 - priority))
    } else if cfg!(mach_o) {
        if priority != 0 {
            Err(quote!(compile_error!(
                "Constructor priority not supported on this plateform."
            ))
            .into())
        } else {
            Ok("__DATA,__mod_term_func".to_string())
        }
    } else if cfg!(coff) {
        Ok(format!(".CRT$XPTZ{:05}", 65535 - priority))
    } else {
        Err(const_dtor_no_support())
    }
}

/// Attribute for functions run at program termination (after main)
///
/// ```ignore
/// #[destructor]
/// extern "C" fn droper () {
/// // run after main return
/// }
/// ```
///
/// The execution order of destructors is unspecified. Nevertheless on ELF plateform (linux,any unixes but mac) and
/// windows plateform a priority can be specified using the syntax `destructor(<num>)` where
/// `<num>` is a number included in the range [0 ; 2<sup>16</sup>-1].
///
/// Destructors with priority 0 are run first (in unspecified order),
/// then destructors with priority number 1,... finaly destructors with priority 65535 are run.
///
/// An abscence of priority is equivalent to a priority of 0.
///
/// ```ignore
/// #[destructor(1)]
/// unsafe extern "C" fn first () {
/// // run after main return
/// }
///
/// #[destructor(0)]
/// unsafe extern "C" fn then () {
/// // run after main return
/// }
/// ```
///
/// # Destructor signature
///
/// Destructor function should have type `unsafe extern "C" fn() -> ()`.
#[proc_macro_attribute]
pub fn destructor(args: TokenStream, input: TokenStream) -> TokenStream {
    let priority = ok_or_return!(parse_priority(args));

    let section = ok_or_return!(fini_section(priority));

    let func: ItemFn = parse_macro_input!(input);

    let func_ptr_name = format!("__static_init_destructor_{}", func.sig.ident);

    let sp = func.sig.span();
    let func_type = parse2(quote_spanned!(sp.span()=>extern "C" fn())).unwrap();

    gen_ctor_dtor(func, &section, &func_ptr_name, func_type).into()
}

/// Statics initialized with non const functions.
///
/// Statics on which this attribute is applied will be be initialized at run time (optionaly see
/// bellow), before main start. This allow statics initialization with non const expressions.
///
/// There are two variants of dynamic statics:
///
/// # Dynamic lazy statics
///
/// These dynamics are supported on all plateforms and requires std support. Lazy dynamics are
/// enabled by the default feature "lazy".
///
/// Lazy statics are declared with the `[dynamic(lazy)]` or simply `[dynamic]` attribute. On unixes and windows plateforms these
/// statics are optimized versions of [std::lazy::SyncLazy]. After program initialization phase,
/// those statics are guaranteed to be initialized and access to them will be as fast as any access
/// to a regular static. On other plateforms, those statics are equivalent to [std::lazy::SyncLazy].
///
/// ```ignore
/// struct A(i32);
///
/// impl A {
///   //new is not const
///   fn new(v:i32) -> A {
///     A(v)
///   }
/// }
///
/// #[dynamic] //equivalently #[dynamic(lazy)]
/// static V :A = A::new(42);
/// ```
///
/// Optionnaly, if the default feature "atexit" is enabled, lazy dynamic statics declared with
/// `[dynamic(lazy,drop)]` will be dropped at program exit. Dropped lazy dynamic statics ared
/// dropped in the reverse order of their initialization. This feature is implemented thanks to
/// `libc::atexit`. See also `drop_reverse` attribute argument.
///
/// ```ignore
/// struct A(i32);
///
/// impl A {
///   //new is not const
///   fn new(v:i32) -> A {
///     A(v)
///   }
/// }
///
/// impl Drop for A {
///     fn drop(&mut A){
///         println!("{}",A.0)
///     }
/// }
///
/// // V2 is initialized before V1,
/// // so V1 will be dropped before V2
/// // The program will print:
/// // 33
/// // 42
/// #[dynamic(lazy,drop)]
/// static V1 :A = A::new(unsafe{V2.0} - 9);
///
/// #[dynamic(drop)] //implies lazy
/// static V2 :A = A::new(42);
/// ```
///
/// Even if V1 is const, access to it must be unsafe because its state may be dropped.
/// Internaly the procedural macro change V1 to a mutable statics and wrap it in a type
/// that does not implement `DerefMut`.
///
/// ## Thread locals
///
/// *lazy statics* can be declared for thread local. This feature does not require std support. 
/// They also can be dropped with if the `thread_local_drop` feature is enabled. 
/// This last feature does require std support.
/// ```ignore
/// #[thread_local]
/// #[dynamic(lazy,drop)]
/// static V1 :A = A::new(unsafe{V2.0} - 9);
///
/// #[thread_local]
/// #[dynamic(drop)] //implies lazy
/// static V2 :A = A::new(42);
/// ```
///
/// # Dynamic statics
///
/// Those statics will be initialized at program startup, without ordering, accept between those
/// that have different priorities on plateform that support priorities. Those statics are
/// supported on unixes and windows with priorities and mac without priorities.
///
/// ## Safety
///
/// Initialization expressions must be unsafe blocks. During initialization, any access to other
/// "dynamic" statics initialized with a lower priority will cause undefined behavior. Similarly,
/// during drop any access to a "dynamic" static dropped with a lower priority will cause undefined
/// behavior.
///
/// ```ignore
/// struct A(i32);
///
/// impl A {
///   //new is not const
///   fn new(v:i32) -> A {
///     A(v)
///   }
/// }
///
/// #[dynamic(0)]
/// static V :A = A::new(42);
/// ```
///
/// ## Execution Order
///
/// The execution order of "dynamic" static initializations is unspecified. Nevertheless on ELF plateform (linux,any unixes but mac) and
/// windows plateform a priority can be specified using the syntax `dynamic(<num>)` where
/// `<num>` is a number included in the range [0 ; 2<sup>16</sup>-1].
///
/// Statics with priority number 65535 are initialized first (in unspecified order), then statics
/// with priority number 65534 are initialized ...  then statics
/// with priority number 0.
///
/// ```ignore
/// struct A(i32);
///
/// impl A {
///   //new is not const
///   fn new(v:i32) -> A {
///     A(v)
///   }
/// }
///
/// //V1 must be initialized first
/// //because V2 uses the value of V1.
/// #[dynamic(10)]
/// static mut V1 :A = A::new(33);
///
/// #[dynamic(20)]
/// static V2 :A = unsafe{A::new(V1.0 + 9)};
/// ```
///
/// # Full syntax and dropped statics
///
/// Finaly the full syntax is for the attribute is:
///
/// ```text
/// "dynamic" [ "(" <dyn_opts> ")" ]
///
/// dyn_opts:
///   <dyn_opt>
///   <dyn_opt>, <dyn_opts>
///
/// dyn_opt:
///   "init" [ "=" <priority> ]
///   "drop" [ "=" <priority> ]
///   "lazy"
///   "drop_only "=" <priority>
/// ```  
///
/// The macro attribute `dynamic` is equivalent to `dynamic(lazy)`
/// and `dynamic(<num>)` to `dynamic(init=<num>)`. If a priority
/// is given it will be dropped by program destructor. The priority has the
/// same semantic as for the [macro@destructor] attribute:  statics with priority 0 are dropped first,
/// ... and finaly statics with priority 65535 are the last dropped.
///
/// The `drop_only=<priority>` is equivalent to #[dynamic(0,drop=<priority>)] except that the
/// static will be const initialized.
///
/// If no priority is given to the drop argument, the drop function will be registered using `libc::atexit`. All
/// dynamic statics registered this way will be dropped in the reverse order of their
/// initialization and before any dynamic statics marked for drop using the `drop` attribute
/// argument.
///
/// ```ignore
/// struct A(i32);
///
/// impl A {
///   //new is not const
///   fn new(v:i32) -> A {
///     A(v)
///   }
///   //new is not const
///   const fn const_new(v:i32) -> A {
///     A(v)
///   }
/// }
///
/// impl Drop for A {
///     fn drop(&mut self) {}
///     }
///
/// //const initialized droped after main exit
/// #[dynamic(init=0, drop=0)]
/// static mut V1 :A = A::new_const(33);
///
/// //initialized before V1 and droped after V1
/// #[dynamic(20,drop=10)]
/// static V2 :A = A::new(10);
///
/// // not droped, V3, V4 and V5 all have initialization priority 0
/// #[dynamic(init=0)]
/// static V3 :A = A::new(10);
///
/// // not droped
/// #[dynamic(init)]
/// static V4 :A = A::new(10);
///
/// // not droped
/// #[dynamic(0)]
/// static V5 :A = A::new(10);
///
/// // not droped
/// #[dynamic(10)]
/// static V6 :A = A::new(10);
/// ```
///
/// # Actual type of "dynamic" statics
///
/// A thread_local *lazy static* that is *not mutable* and that will be dropped is wrapped in a *mutable* thread_local static
/// of type `static_init::ThreadLocalConstLazy`. Otherwise the mutability is unchanged and the
/// static is wrapped in a `static_init::ThreadLocalLazy`.
///
/// A *lazy static* that is *not mutable* and that will be dropped is wrapped in a *mutable* static
/// of type `static_init::ConstLazy`. Otherwise the mutability is unchanged and the
/// static is wrapped in a `static_init::Lazy`.
///
/// A mutable dynamic static declared to have type `T` are wrapped in `static_init::Static<T>`.
///
/// A mutable "dynamic" static declared to have type `T` are wrapped in a mutable static of type `static_init::ConstStatic<T>` 
///
/// ```no_run
///
/// // V has type static_init::ConstStatic<i32>
/// #[dynamic]
/// static V :i32 = 0;
///
/// // W has type static_init::Static<i32>
/// #[dynamic]
/// static W :i32 = 0;
/// ```

#[proc_macro_attribute]
pub fn dynamic(args: TokenStream, input: TokenStream) -> TokenStream {
    let item: ItemStatic = parse_macro_input!(input);

    let options = ok_or_return!(parse_dyn_options(parse_macro_input!(args)));

    gen_dyn_init(item, options).into()
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum InitMode {
    Const,
    Lazy,
    Dynamic(u16),
}
#[derive(Clone, Copy, Eq, PartialEq)]
enum DropMode {
    None,
    AtExit,
    Dynamic(u16),
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct DynMode {
    init: InitMode,
    drop: DropMode,
}

fn parse_priority(args: TokenStream) -> std::result::Result<u16, TokenStream2> {
    if !args.is_empty() {
        if let Ok(n) = syn::parse(args.clone()).map_err(|e| e.to_compile_error()) {
            let n: Ident = n;
            if n == "__lazy_init" {
                return Ok(1);
            } else if n == "__lazy_init_finished" {
                return Ok(0);
            }
        }
        let lit: Lit = syn::parse(args).map_err(|e| e.to_compile_error())?;
        parse_priority_literal(&lit)
    } else {
        Ok(0)
    }
}

macro_rules! generate_error{
    ($span:expr => $($args:tt),*) => {
        {
        let __expand = [$(generate_error!(@expand $args)),*];
        quote_spanned!($span => ::core::compile_error!(::core::concat!(#(#__expand),*)))
        }
    };
    ($($args:tt),*) => {{
        let __expand = [$(generate_error!(@expand $args)),*];
        quote!(::core::compile_error!(::core::concat!(#(#__expand),*)))
    }
    };
    (@expand $v:literal) => {
        quote!($v)
    };
    (@expand $v:ident) => {
        {
        quote!(::core::stringify!(#$v))
        }
    };

}

fn parse_priority_literal(lit: &Lit) -> Result<u16, TokenStream2> {
    match lit {
        Lit::Int(n) => n.base10_parse::<u16>().map_err(|e| e.to_compile_error()),
        _ => Err(
            generate_error!(lit.span()=>"Expected a priority in the range [0 ; 65535], found `",lit,"`."),
        ),
    }
}

fn parse_dyn_options(args: AttributeArgs) -> std::result::Result<DynMode, TokenStream2> {
    let mut opt = DynMode {
        init: InitMode::Lazy,
        drop: DropMode::None,
    };

    let mut init_set = false;
    let mut drop_set = false;
    macro_rules! check_no_init{
        ($id: expr) => {
            if init_set {
                let __attr_arg = &$id;
                return Err(generate_error!($id.span()=>"Initialization already specified `",__attr_arg,"`"));
            } else {
                init_set = true;
            }
        }
    }
    macro_rules! check_no_drop{
        ($id: expr) => {
            if drop_set {
                let __attr_arg = &$id;
                return Err(generate_error!($id.span()=>"Drop already specified `",__attr_arg,"`"));
            } else {
                drop_set = true;
            }
        }
    }

    macro_rules! unexpected_arg{
        ($id: expr) => {{
            let __unexpected = &$id;
            Err(generate_error!($id.span()=>
                "Unexpected attribute argument `",
                __unexpected,
                "`. Expected either `init[=<u16>]`, `drop[=<u16>]`, `lazy` or `drop_only=<u16>`."
                ))
        }
        }
    }

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::Path(id)) => {
                let id = if let Some(id) = id.get_ident() {
                    id
                } else {
                    return unexpected_arg!(id);
                };
                if id == "init" {
                    check_no_init!(id);
                    opt.init = InitMode::Dynamic(0);
                } else if id == "drop" {
                    check_no_drop!(id);
                    if !cfg!(feature = "atexit") {
                        return Err(
                            generate_error!(id.span()=>"static_init crate feature `atexit` is not enabled.",id),
                        );
                    }
                    opt.drop = DropMode::AtExit;
                } else if id == "lazy" {
                    check_no_init!(id);
                    opt.init = InitMode::Lazy;
                } else {
                    return unexpected_arg!(id);
                }
            }
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let id = if let Some(id) = nv.path.get_ident() {
                    id
                } else {
                    return unexpected_arg!(nv.path);
                };
                if id == "init" {
                    check_no_init!(id);
                    let priority = parse_priority_literal(&nv.lit)?;
                    opt.init = InitMode::Dynamic(priority);
                } else if id == "drop" {
                    check_no_drop!(id);
                    let priority = parse_priority_literal(&nv.lit)?;
                    opt.drop = DropMode::Dynamic(priority);
                } else if id == "drop_only" {
                    check_no_init!(id);
                    check_no_drop!(id);
                    let priority = parse_priority_literal(&nv.lit)?;
                    opt.init = InitMode::Const;
                    opt.drop = DropMode::Dynamic(priority);
                } else {
                    return unexpected_arg!(id);
                }
            }
            NestedMeta::Lit(lit) => {
                check_no_init!(lit);
                let priority = parse_priority_literal(&lit)?;
                opt.init = InitMode::Dynamic(priority);
            }
            _ => {
                return unexpected_arg!(arg);
            }
        }
    }
    if opt.init == InitMode::Lazy && !cfg!(feature = "lazy") {
        Err(generate_error!(
            "static_init crate feature `lazy` is not enabled."
        ))
    } else if opt.init == InitMode::Lazy
        && !(opt.drop == DropMode::None || opt.drop == DropMode::AtExit)
    {
        Err(generate_error!("Drop mode not supported for lazy statics."))
    } else {
        Ok(opt)
    }
}

fn gen_ctor_dtor(
    func: ItemFn,
    section: &str,
    func_ptr_name: &str,
    typ: TypeBareFn,
) -> TokenStream2 {
    let func_ptr_name = Ident::new(func_ptr_name, Span::call_site());

    let section = LitStr::new(section, Span::call_site());

    let func_name = &func.sig.ident;

    let sp = func.sig.span();
    //if func.sig.unsafety.is_none() {
    //    quote_spanned! {sp=>compile_error!("Constructors and destructors must be unsafe functions as \
    //    they may access uninitialized memory regions")}
    //} else {
    quote_spanned! {sp=>
        #func
        #[doc(hidden)]
        #[link_section = #section]
        #[used]
        pub static #func_ptr_name: #typ = #func_name;
    }
    //}
}

fn has_thread_local(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        for seg in &attr.path.segments {
            if seg.ident == "thread_local" {
                return true;
            }
        }
    }
    false
}

fn gen_dyn_init(mut stat: ItemStatic, options: DynMode) -> TokenStream2 {
    let stat_name = &stat.ident;

    let expr = &*stat.expr;

    let stat_typ = &*stat.ty;

    let is_thread_local = has_thread_local(&stat.attrs);

    if is_thread_local && options.init != InitMode::Lazy {
        return generate_error!(
            "Only statics with `#[dynamic(lazy)]` or `#[dynamic(lazy,drop)]` can also have \
             `#[thread_local]` attribute"
        );
    }
    if is_thread_local && options.drop == DropMode::AtExit && !cfg!(feature = "thread_local_drop") {
        return generate_error!(
            "`#[thread_local] #[dynamic(lazy,drop)]` needs static_init crate `thread_local_drop` feature"
        );
    }

    let stat_ref: Expr = if options.init != InitMode::Lazy && stat.mutability.is_some() {
        parse_quote! {
            &mut #stat_name
        }
    } else {
        parse_quote! {
            &#stat_name
        }
    };

    macro_rules! into_mutable {
        () => {
            stat.mutability = Some(token::Mut {
                span: stat.ty.span(),
            })
        };
    }

    let typ: Type = if options.init != InitMode::Lazy {
        if stat.mutability.is_none() {
            into_mutable!();
            parse_quote! {
                ::static_init::ConstStatic::<#stat_typ>
            }
        } else {
            parse_quote! {
                ::static_init::Static::<#stat_typ>
            }
        }
    } else if is_thread_local {
        if stat.mutability.is_none() && options.drop == DropMode::AtExit {
            into_mutable!();
            parse_quote! {
                ::static_init::ThreadLocalConstLazy::<#stat_typ>
            }
        } else {
            parse_quote! {
                ::static_init::ThreadLocalLazy::<#stat_typ>
            }
        }
    } else {
        if stat.mutability.is_none() && options.drop == DropMode::AtExit {
            into_mutable!();
            parse_quote! {
                ::static_init::ConstLazy::<#stat_typ>
            }
        } else {
            parse_quote! {
                ::static_init::Lazy::<#stat_typ>
            }
        }
    };

    let sp = stat.expr.span();

    let initer = match options.init {
        InitMode::Dynamic(priority) if options.drop == DropMode::AtExit => {
            let attr: Attribute = parse_quote!(#[::static_init::constructor(#priority)]);
            Some(quote_spanned! {sp=>
                    extern "C" fn __static_init_dropper() {
                        unsafe{#typ::drop(#stat_ref)}
                    }
                    #attr
                    extern "C" fn __static_init_initializer() {
                        ::static_init::__set_init_prio(#priority as i32);
                        let __static_init_expr_result = #expr;
                        unsafe {#typ::set_to(#stat_ref,__static_init_expr_result);
                        ::libc::atexit(__static_init_dropper)};
                        ::static_init::__set_init_prio(i32::MIN);
                    }
            })
        }

        InitMode::Dynamic(priority) => {
            let attr: Attribute = parse_quote!(#[::static_init::constructor(#priority)]);
            Some(quote_spanned! {sp=>
                    #attr
                    extern "C" fn __static_init_initializer() {
                        ::static_init::__set_init_prio(#priority as i32);
                        let __static_init_expr_result = #expr;
                        unsafe {#typ::set_to(#stat_ref,__static_init_expr_result)};
                        ::static_init::__set_init_prio(i32::MIN);
                    }
            })
        }

        InitMode::Lazy => Some(quote_spanned! {sp=>
                #[::static_init::constructor(__lazy_init)]
                extern "C" fn __static_init_initializer() {
                    #[allow(unused_unsafe)]
                    unsafe {#typ::__do_init(#stat_ref)};
                }
        }),

        InitMode::Const => None,
    };

    let droper = if let DropMode::Dynamic(priority) = options.drop {
        let attr: Attribute = parse_quote!(#[::static_init::destructor(#priority)]);
        Some(quote_spanned! {sp=>
                #attr
                extern "C" fn __static_init_droper() {
                    unsafe {#typ::drop(#stat_ref)}
                }
        })
    } else {
        None
    };

    let statid = &stat.ident;

    let init_priority: Expr = match options.init {
        InitMode::Dynamic(n) => parse_quote!(::static_init::InitMode::Dynamic(#n)),
        InitMode::Lazy => parse_quote!(::static_init::InitMode::Lazy),
        InitMode::Const => parse_quote!(::static_init::InitMode::Const),
    };

    let drop_priority: Expr = match options.drop {
        DropMode::Dynamic(n) => parse_quote!(::static_init::DropMode::Dynamic(#n)),
        DropMode::AtExit => parse_quote!(::static_init::DropMode::AtExit),
        DropMode::None => parse_quote!(::static_init::DropMode::None),
    };

    let static_info: Option<Expr> = if cfg!(debug_mode) {
        Some(parse_quote!(
    ::static_init::StaticInfo{
        variable_name: ::core::stringify!(#statid),
        file_name: ::core::file!(),
        line: ::core::line!(),
        column: ::core::column!(),
        init_mode: #init_priority,
        drop_mode: #drop_priority
        })) 
    } else {
        None
    };

    let const_init = match options.init {
        InitMode::Dynamic(_) => {
            quote_spanned! {sp=>{
                #initer
                #droper
                #typ::uninit(#static_info)
            }
            }
        }
        InitMode::Lazy if !(options.drop == DropMode::AtExit) => {
            quote_spanned! {sp=>{
                #initer
                #typ::new(|| {#expr},#static_info)
            }
            }
        }
        InitMode::Lazy if !is_thread_local => {
            quote_spanned! {sp=>{
                extern "C" fn __static_init_dropper() {
                    unsafe{::core::ptr::drop_in_place(#typ::as_mut_ptr(#stat_ref))}
                }
                #initer
                #typ::new(
                    || {
                        let v = (|| {#expr})();
                        unsafe{::libc::atexit(__static_init_dropper)};
                        v
                        },
                    #static_info
                    )
            }}
        }
        InitMode::Lazy => {
            //thread local drop
            quote_spanned! {sp=>{
                fn __static_init_dropper() {
                    unsafe{::core::ptr::drop_in_place(#typ::as_mut_ptr(#stat_ref))}
                }
                #initer
                #typ::new(
                    || {
                        let v = (|| {#expr})();
                        unsafe{::static_init::__push_tls_destructor(__static_init_dropper)};
                        v
                        },
                    #static_info
                    )
            }
            }
        }
        InitMode::Const => {
            quote_spanned! {sp=>{
                #initer
                #droper
                #typ::from(#expr, #static_info)
            }
            }
        }
    };

    *stat.expr = match parse(const_init.into()) {
        Ok(exp) => exp,
        Err(e) => return e.to_compile_error(),
    };

    *stat.ty = typ;

    quote_spanned! {sp=>

    #stat
    }
}
