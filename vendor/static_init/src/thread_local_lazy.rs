#[cfg(debug_mode)]
use super::StaticInfo;

pub use lazy_impl::{Lazy, ConstLazy};

#[cfg(not(debug_mode))]
mod lazy_impl {
    use core::cell::Cell;
    use core::cell::UnsafeCell;
    use core::fmt;
    use core::mem::MaybeUninit;
    use core::ops::{Deref, DerefMut};

    /// The type of thread local lazy.
    pub struct Lazy<T, F = fn() -> T> {
        value:    UnsafeCell<MaybeUninit<T>>,
        init_exp: Cell<Option<F>>,
    }

    /// The type of const thread local lazy that will be dropped.
    ///
    /// This type does not implement DerefMut but is intended to be
    /// decalred mut to ensure access to is is unsafe.
    #[derive(Debug)]
    pub struct ConstLazy<T, F = fn() -> T>(Lazy<T, F>);

    impl<T, F> Lazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        pub const fn new(f: F) -> Self {
            Self {
                value:    UnsafeCell::new(MaybeUninit::uninit()),
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

        /// Ensure the value is initialized
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Self::ensure_init(this)
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
            if let Some(f) = this.init_exp.take() {
                unsafe { (*this.value.get()).as_mut_ptr().write(f()) };
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
            Lazy::ensure_init(&this.0)
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
            Self::__do_init(this);
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
                ConstLazy::ensure_init(self);
                &*ConstLazy::as_mut_ptr(self)
            }
        }
    }
}

#[cfg(debug_mode)]
mod lazy_impl {
    use super::StaticInfo;
    use core::cell::Cell;
    use core::cell::UnsafeCell;
    use core::fmt;
    use core::mem::MaybeUninit;
    use core::ops::{Deref, DerefMut};

    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    enum Status {
        NotInitialized,
        Initializing,
        Initialized,
        Droped,
    }

    /// The type of thread local lazy.
    pub struct Lazy<T, F = fn() -> T> {
        value:    UnsafeCell<MaybeUninit<T>>,
        init_exp: Cell<Option<F>>,
        status:   Cell<Status>,
        info:     Option<StaticInfo>,
    }

    /// The type of const thread local lazy that will be dropped.
    ///
    /// This type does not implement DerefMut but is intended to be
    /// decalred mut to ensure access to is is unsafe.
    #[derive(Debug)]
    pub struct ConstLazy<T, F = fn() -> T>(Lazy<T, F>);

    impl<T, F> Lazy<T, F> {
        /// Initialize a lazy with a builder as argument.
        ///
        /// This function is intended to be used internaly
        /// by the dynamic macro.
        pub const fn new(f: F, info: StaticInfo) -> Self {
            Self {
                value:    UnsafeCell::new(MaybeUninit::uninit()),
                init_exp: Cell::new(Some(f)),
                status:   Cell::new(Status::NotInitialized),
                info:     Some(info),
            }
        }

        /// Return a pointer to the value.
        ///
        /// The value may be in an uninitialized state.
        #[inline(always)]
        pub const fn as_mut_ptr(this: &Self) -> *mut T {
            this.value.get() as *mut T
        }

        /// Ensure the value is initialized
        ///
        /// Once this function is called, it is guaranteed that
        /// the value is in an initialized state.
        ///
        /// This function is always called when the lazy is dereferenced.
        #[inline(always)]
        pub fn __do_init(this: &Self)
        where
            F: FnOnce() -> T,
        {
            Self::ensure_init(this)
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
            if let Some(f) = this.init_exp.take() {
                match this.status.get() {
                    Status::NotInitialized => {
                        this.status.set(Status::Initializing);
                        unsafe { (*this.value.get()).as_mut_ptr().write(f()) };
                        this.status.set(Status::Initialized);
                    }
                    _ => panic!("Unexpected"),
                }
            }
        }
        /// Drop the contained value
        ///
        /// # Safety
        ///
        /// The value should not be accessed any more.
        pub unsafe fn drop(this: &Self) {
            check_status(this.status.get(), &this.info);
            Self::as_mut_ptr(this).drop_in_place();
            this.status.set(Status::Droped);
        }
    }
    fn check_status(st: Status, info: &Option<StaticInfo>) {
        match st {
            Status::Initializing => {
                if let Some(info) = info {
                    core::panic!("Recurcive lazy initialization of {:#?}.", info);
                } else {
                    core::panic!("Recurcive lazy initialization.");
                }
            }
            Status::Droped => {
                if let Some(info) = info {
                    core::panic!("Attempt to access {:#?} after it has been dropped.", info);
                } else {
                    core::panic!("Attempt to access a thread_local after it has been dropped.");
                }
            }
            Status::NotInitialized => {
                if let Some(info) = info {
                    core::panic!(
                        "Attempt to access {:#?} while it is not initialized. This might be a bug \
                         of static_init crate.",
                        info
                    );
                } else {
                    core::panic!(
                        "Attempt to access a thread_local while it is not initialized. This might \
                         be a bug of static_init crate."
                    );
                }
            }
            _ => (),
        }
    }

    impl<T, F> Deref for Lazy<T, F>
    where
        F: FnOnce() -> T,
    {
        type Target = T;
        #[inline(always)]
        fn deref(&self) -> &T {
            unsafe {
                Lazy::ensure_init(self);
                check_status(self.status.get(), &self.info);
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
                check_status(self.status.get(), &self.info);
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
            Lazy::ensure_init(&this.0)
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
            Self::__do_init(this);
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
                ConstLazy::ensure_init(self);
                &*ConstLazy::as_mut_ptr(self)
            }
        }
    }
}

#[cfg(feature = "thread_local_drop")]
mod lazy_drop {
    use core::cell::UnsafeCell;
    struct DestructorRegister(UnsafeCell<Option<Vec<fn()>>>);

    impl Drop for DestructorRegister {
        fn drop(&mut self) {
            if let Some(vec) = unsafe { (*self.0.get()).take() } {
                for f in vec {
                    f()
                }
            }
        }
    }

    unsafe impl Sync for DestructorRegister {}

    thread_local! {
        static DESTRUCTORS: DestructorRegister = DestructorRegister(UnsafeCell::new(None));
    }

    #[doc(hidden)]
    #[inline(always)]
    unsafe fn ensure_init() {
        DESTRUCTORS.with(|d| {
            if (*d.0.get()).is_none() {
                *d.0.get() = Some(vec![])
            }
        })
    }

    #[doc(hidden)]
    #[inline(always)]
    pub unsafe fn __push_tls_destructor(f: fn()) {
        ensure_init();
        DESTRUCTORS.with(|d| (*d.0.get()).as_mut().unwrap().push(f));
    }
}
#[cfg(feature = "thread_local_drop")]
pub use lazy_drop::__push_tls_destructor;
