#[cfg(debug_mode)]
use super::StaticInfo;

pub use lazy_impl::{ConstLazy, Lazy};

#[cfg(all(support_priority,not(feature = "test_no_global_lazy_hint")))]
mod inited {

    use core::sync::atomic::{AtomicBool, Ordering};

    static LAZY_INIT_ENSURED: AtomicBool = AtomicBool::new(false);

    #[static_init_macro::constructor(__lazy_init_finished)]
    extern "C" fn mark_inited() {
        LAZY_INIT_ENSURED.store(true, Ordering::Release);
    }
    
    #[inline(always)]
    pub(crate) fn global_inited_hint() -> bool {
        LAZY_INIT_ENSURED.load(Ordering::Acquire)
    }
}

#[cfg(all(support_priority,not(feature = "test_no_global_lazy_hint")))]
use inited::global_inited_hint;

#[cfg(not(all(support_priority,not(feature = "test_no_global_lazy_hint"))))]
#[inline(always)]
fn global_inited_hint() -> bool {
    false
}

#[cfg(debug_mode)]
mod lazy_impl {
    use super::StaticInfo;
    use super::global_inited_hint;

    use core::cell::Cell;
    use core::cell::UnsafeCell;
    use core::fmt;
    use core::mem::MaybeUninit;
    use core::ops::{Deref, DerefMut};
    use core::sync::atomic::{AtomicBool, Ordering};

    use parking_lot::{
        lock_api::GetThreadId, lock_api::RawMutex as _, RawMutex, RawThreadId, ReentrantMutex,
    };

    use core::num::NonZeroUsize;

    struct DebugLazyState<F> {
        initer:   Cell<Option<NonZeroUsize>>,
        function: Cell<Option<F>>,
    }

    /// The type of *lazy statics*.
    ///
    /// Statics that are initialized on first access.
    pub struct Lazy<T, F = fn() -> T> {
        value:        UnsafeCell<MaybeUninit<T>>,
        inited:       AtomicBool,
        debug_initer: ReentrantMutex<DebugLazyState<F>>,
        info:         Option<StaticInfo>,
        dropped:      AtomicBool,
    }

    /// The type of const *lesser lazy statics*.
    ///
    /// Statics that are initialized on first access or before main is called.
    ///
    /// They are declared mut when the lazy is drop so that the compiler inform the coder that access
    /// to those statics are unsafe: during program destruction (after main exit) the state may be
    /// invalid.
    #[derive(Debug)]
    pub struct ConstLazy<T, F = fn() -> T>(Lazy<T, F>);

    impl<T, F> Lazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        ///
        /// This function is intended to be used internaly
        /// by the dynamic macro.
        pub const fn new(f: F, _info: StaticInfo) -> Self {
            Self {
                value:        UnsafeCell::new(MaybeUninit::uninit()),
                inited:       AtomicBool::new(false),
                debug_initer: ReentrantMutex::const_new(
                    RawMutex::INIT,
                    RawThreadId::INIT,
                    DebugLazyState {
                        initer:   Cell::new(None),
                        function: Cell::new(Some(f)),
                    },
                ),
                info:         Some(_info),
                dropped:      AtomicBool::new(false),
            }
        }

        /// Return a pointer to the value.
        ///
        /// The value may be in an uninitialized state.
        #[inline(always)]
        pub const fn as_mut_ptr(this: &Self) -> *mut T {
            this.value.get() as *mut T
        }

        /// Ensure the value is initialized without optimization check
        ///
        /// This is intended to be used at program start up by
        /// the dynamic macro.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            if !this.inited.load(Ordering::Acquire) {
                let l = this.debug_initer.lock();
                if let Some(initer) = l.initer.get() {
                    if initer == RawThreadId.nonzero_thread_id() {
                        if let Some(info) = &this.info {
                            core::panic!("Recurcive lazy initialization of {:#?}.", info);
                        } else {
                            core::panic!("Recurcive lazy initialization.");
                        }
                    }
                    return;
                } else {
                    l.initer.set(Some(RawThreadId.nonzero_thread_id()));
                    unsafe {
                        (*this.value.get())
                            .as_mut_ptr()
                            .write(l.function.take().unwrap()())
                    };
                    this.inited.store(true, Ordering::Release);
                }
            }
        }
        /// Ensure the value is initialized without optimization check
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn ensure_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            if !global_inited_hint() {
                Self::__do_init(this);
            }
            if this.dropped.load(Ordering::Acquire) {
                if let Some(info) = &this.info {
                    core::panic!("Access to a dropped lazy static {:#?}.", info);
                } else {
                    core::panic!("Access to a dropped lazy static.");
                }
            }
        }

        /// Drop the contained value
        ///
        /// # Safety
        ///
        /// The value should not be accessed any more.
        pub unsafe fn drop(this: &Self) {
            Self::as_mut_ptr(this).drop_in_place();
            this.dropped.store(true, Ordering::Relaxed);
        }
    }

    unsafe impl<F, T: Send + Sync> Send for Lazy<T, F> {}

    unsafe impl<F, T: Sync> Sync for Lazy<T, F> {}

    impl<T, F> Deref for Lazy<T, F>
    where
        F: FnOnce() -> T,
    {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe {
                Lazy::ensure_init(self);
                &*Lazy::as_mut_ptr(self)
            }
        }
    }
    impl<T, F> DerefMut for Lazy<T, F>
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        fn deref_mut(&mut self) -> &mut T {
            unsafe {
                Lazy::ensure_init(self);
                &mut *Lazy::as_mut_ptr(self)
            }
        }
    }
    impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Lazy")
                .field("cell", &self.value)
                .field("init", &"..")
                .finish()
        }
    }

    impl<T, F> ConstLazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        ///
        /// This function is intended to be used internaly
        /// by the dynamic macro.
        ///
        /// # Safety
        ///
        /// This variable shall not be used as a thread_local
        /// statics or within the state of a thread_local static
        pub const fn new(f: F, info: StaticInfo) -> Self {
            Self(Lazy::new(f, info))
        }

        /// Return a pointer to the value.
        ///
        /// The value may be in an uninitialized state.
        #[inline(always)]
        pub const fn as_mut_ptr(this: &Self) -> *mut T {
            Lazy::as_mut_ptr(&this.0)
        }
        /// Ensure the value is initialized without optimization check
        ///
        /// This is intended to be used at program start up by
        /// the dynamic macro.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Lazy::__do_init(&this.0)
        }
        /// Ensure the value is initialized
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn ensure_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Lazy::ensure_init(&this.0)
        }
    }

    impl<T, F> Deref for ConstLazy<T, F>
    where
        F: FnOnce() -> T,
    {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe {
                Self::ensure_init(self);
                &*Self::as_mut_ptr(self)
            }
        }
    }
}

#[cfg(not(debug_mode))]
mod lazy_impl {
    use super::global_inited_hint;

    use core::cell::Cell;
    use core::cell::UnsafeCell;
    use core::fmt;
    use core::hint::unreachable_unchecked;
    use core::mem::MaybeUninit;
    use core::ops::{Deref, DerefMut};

    use parking_lot::Once;

    /// The type of *lesser lazy statics*.
    ///
    /// Statics that are initialized on first access or
    /// before main is called.
    pub struct Lazy<T, F = fn() -> T> {
        value:    UnsafeCell<MaybeUninit<T>>,
        initer:   Once,
        init_exp: Cell<Option<F>>,
    }
    /// The type of const *lesser lazy statics*.
    ///
    /// Statics that are initialized on first access or before main is called.
    ///
    /// They are declared mut when the lazy is drop so that the compiler inform the coder that access
    /// to those statics are unsafe: during program destruction (after main exit) the state may be
    /// invalid.
    #[derive(Debug)]
    pub struct ConstLazy<T, F = fn() -> T>(Lazy<T, F>);

    impl<T, F> Lazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        pub const fn new(f: F) -> Self {
            Self {
                value:    UnsafeCell::new(MaybeUninit::uninit()),
                initer:   Once::new(),
                init_exp: Cell::new(Some(f)),
            }
        }

        /// Return a pointer to the value.
        ///
        /// The value may be in an uninitialized state.
        #[inline(always)]
        pub const fn as_mut_ptr(this: &Self) -> *mut T {
            this.value.get() as *mut T
        }

        /// Ensure the value is initialized without optimization check
        ///
        /// This is intended to be used at program start up by
        /// the dynamic macro.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            //The compiler fails to automatically choose
            //which branch is the best one...
            this.initer.call_once(|| unsafe {
                (*this.value.get()).as_mut_ptr().write(this
                    .init_exp
                    .take()
                    .unwrap_or_else(|| unreachable_unchecked())(
                ));
            });
        }
        /// Ensure the value is initialized without optimization check
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn ensure_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            if !global_inited_hint() {
                Self::__do_init(this);
            }
        }
        /// Drop the contained value
        ///
        /// # Safety
        ///
        /// The value should not be accessed any more.
        pub unsafe fn drop(this: &Self) {
            Self::as_mut_ptr(this).drop_in_place()
        }
    }

    unsafe impl<F, T: Send + Sync> Send for Lazy<T, F> {}

    unsafe impl<F, T: Sync> Sync for Lazy<T, F> {}

    impl<T, F> Deref for Lazy<T, F>
    where
        F: FnOnce() -> T,
    {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe {
                Lazy::ensure_init(self);
                &*Lazy::as_mut_ptr(self)
            }
        }
    }
    impl<T, F> DerefMut for Lazy<T, F>
    where
        F: FnOnce() -> T,
    {
        #[inline(always)]
        fn deref_mut(&mut self) -> &mut T {
            unsafe {
                Lazy::ensure_init(self);
                &mut *Lazy::as_mut_ptr(self)
            }
        }
    }
    impl<T: fmt::Debug, F> fmt::Debug for Lazy<T, F> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Lazy")
                .field("cell", &self.value)
                .field("init", &"..")
                .finish()
        }
    }

    impl<T, F> ConstLazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        ///
        /// # Safety
        ///
        /// This variable shall not be used as a thread_local
        /// statics or within the state of a thread_local static
        pub const fn new(f: F) -> Self {
            Self(Lazy::new(f))
        }

        /// Return a pointer to the value.
        ///
        /// The value may be in an uninitialized state.
        #[inline(always)]
        pub const fn as_mut_ptr(this: &Self) -> *mut T {
            Lazy::as_mut_ptr(&this.0)
        }
        /// Ensure the value is initialized without optimization check
        ///
        /// This is intended to be used at program start up by
        /// the dynamic macro.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Lazy::__do_init(&this.0)
        }
        /// Ensure the value is initialized
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn ensure_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Lazy::ensure_init(&this.0)
        }
    }

    impl<T, F> Deref for ConstLazy<T, F>
    where
        F: FnOnce() -> T,
    {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe {
                Self::ensure_init(self);
                &*Self::as_mut_ptr(self)
            }
        }
    }
}
