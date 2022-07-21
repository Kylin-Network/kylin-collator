//! Implements the pooling instance allocator.
//!
//! The pooling instance allocator maps memory in advance
//! and allocates instances, memories, tables, and stacks from
//! a pool of available resources.
//!
//! Using the pooling instance allocator can speed up module instantiation
//! when modules can be constrained based on configurable limits.

use super::{
    initialize_instance, InstanceAllocationRequest, InstanceAllocator, InstanceHandle,
    InstantiationError,
};
use crate::{instance::Instance, Memory, Mmap, Table};
use crate::{MemoryImageSlot, ModuleRuntimeInfo, Store};
use anyhow::{anyhow, bail, Context, Result};
use libc::c_void;
use std::convert::TryFrom;
use std::mem;
use std::sync::Mutex;
use wasmtime_environ::{
    DefinedMemoryIndex, DefinedTableIndex, HostPtr, Module, PrimaryMap, Tunables, VMOffsets,
    WASM_PAGE_SIZE,
};

mod index_allocator;
use index_allocator::{PoolingAllocationState, SlotId};

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        use windows as imp;
    } else if #[cfg(all(feature = "uffd", target_os = "linux"))] {
        mod uffd;
        use uffd as imp;
        use imp::initialize_memory_pool;
    } else if #[cfg(target_os = "linux")] {
        mod linux;
        use linux as imp;
    } else {
        mod unix;
        use unix as imp;
    }
}

use imp::{commit_memory_pages, commit_table_pages, decommit_memory_pages, decommit_table_pages};

#[cfg(all(feature = "async", unix))]
use imp::{commit_stack_pages, decommit_stack_pages};

#[cfg(feature = "async")]
use super::FiberStackError;

fn round_up_to_pow2(n: usize, to: usize) -> usize {
    debug_assert!(to > 0);
    debug_assert!(to.is_power_of_two());
    (n + to - 1) & !(to - 1)
}

/// Represents the limits placed on instances by the pooling instance allocator.
#[derive(Debug, Copy, Clone)]
pub struct InstanceLimits {
    /// The maximum number of concurrent instances supported (default is 1000).
    ///
    /// This value has a direct impact on the amount of memory allocated by the pooling
    /// instance allocator.
    ///
    /// The pooling instance allocator allocates three memory pools with sizes depending on this value:
    ///
    /// * An instance pool, where each entry in the pool can store the runtime representation
    ///   of an instance, including a maximal `VMContext` structure.
    ///
    /// * A memory pool, where each entry in the pool contains the reserved address space for each
    ///   linear memory supported by an instance.
    ///
    /// * A table pool, where each entry in the pool contains the space needed for each WebAssembly table
    ///   supported by an instance (see `table_elements` to control the size of each table).
    ///
    /// Additionally, this value will also control the maximum number of execution stacks allowed for
    /// asynchronous execution (one per instance), when enabled.
    ///
    /// The memory pool will reserve a large quantity of host process address space to elide the bounds
    /// checks required for correct WebAssembly memory semantics. Even for 64-bit address spaces, the
    /// address space is limited when dealing with a large number of supported instances.
    ///
    /// For example, on Linux x86_64, the userland address space limit is 128 TiB. That might seem like a lot,
    /// but each linear memory will *reserve* 6 GiB of space by default. Multiply that by the number of linear
    /// memories each instance supports and then by the number of supported instances and it becomes apparent
    /// that address space can be exhausted depending on the number of supported instances.
    pub count: u32,

    /// The maximum size, in bytes, allocated for an instance and its
    /// `VMContext`.
    ///
    /// This amount of space is pre-allocated for `count` number of instances
    /// and is used to store the runtime `wasmtime_runtime::Instance` structure
    /// along with its adjacent `VMContext` structure. The `Instance` type has a
    /// static size but `VMContext` is dynamically sized depending on the module
    /// being instantiated. This size limit loosely correlates to the size of
    /// the wasm module, taking into account factors such as:
    ///
    /// * number of functions
    /// * number of globals
    /// * number of memories
    /// * number of tables
    /// * number of function types
    ///
    /// If the allocated size per instance is too small then instantiation of a
    /// module will fail at runtime with an error indicating how many bytes were
    /// needed. This amount of bytes are committed to memory per-instance when
    /// a pooling allocator is created.
    ///
    /// The default value for this is 1MB.
    pub size: usize,

    /// The maximum number of defined tables for a module (default is 1).
    ///
    /// This value controls the capacity of the `VMTableDefinition` table in each instance's
    /// `VMContext` structure.
    ///
    /// The allocated size of the table will be `tables * sizeof(VMTableDefinition)` for each
    /// instance regardless of how many tables are defined by an instance's module.
    pub tables: u32,

    /// The maximum table elements for any table defined in a module (default is 10000).
    ///
    /// If a table's minimum element limit is greater than this value, the module will
    /// fail to instantiate.
    ///
    /// If a table's maximum element limit is unbounded or greater than this value,
    /// the maximum will be `table_elements` for the purpose of any `table.grow` instruction.
    ///
    /// This value is used to reserve the maximum space for each supported table; table elements
    /// are pointer-sized in the Wasmtime runtime.  Therefore, the space reserved for each instance
    /// is `tables * table_elements * sizeof::<*const ()>`.
    pub table_elements: u32,

    /// The maximum number of defined linear memories for a module (default is 1).
    ///
    /// This value controls the capacity of the `VMMemoryDefinition` table in each instance's
    /// `VMContext` structure.
    ///
    /// The allocated size of the table will be `memories * sizeof(VMMemoryDefinition)` for each
    /// instance regardless of how many memories are defined by an instance's module.
    pub memories: u32,

    /// The maximum number of pages for any linear memory defined in a module (default is 160).
    ///
    /// The default of 160 means at most 10 MiB of host memory may be committed for each instance.
    ///
    /// If a memory's minimum page limit is greater than this value, the module will
    /// fail to instantiate.
    ///
    /// If a memory's maximum page limit is unbounded or greater than this value,
    /// the maximum will be `memory_pages` for the purpose of any `memory.grow` instruction.
    ///
    /// This value is used to control the maximum accessible space for each linear memory of an instance.
    ///
    /// The reservation size of each linear memory is controlled by the
    /// `static_memory_maximum_size` setting and this value cannot
    /// exceed the configured static memory maximum size.
    pub memory_pages: u64,
}

impl Default for InstanceLimits {
    fn default() -> Self {
        // See doc comments for `wasmtime::InstanceLimits` for these default values
        Self {
            count: 1000,
            size: 1 << 20, // 1 MB
            tables: 1,
            table_elements: 10_000,
            memories: 1,
            memory_pages: 160,
        }
    }
}

/// The allocation strategy to use for the pooling instance allocator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolingAllocationStrategy {
    /// Allocate from the next available instance.
    NextAvailable,
    /// Allocate from a random available instance.
    Random,
    /// Try to allocate an instance slot that was previously used for
    /// the same module, potentially enabling faster instantiation by
    /// reusing e.g. memory mappings.
    ReuseAffinity,
}

impl Default for PoolingAllocationStrategy {
    fn default() -> Self {
        if cfg!(memory_init_cow) {
            Self::ReuseAffinity
        } else {
            Self::NextAvailable
        }
    }
}

/// Represents a pool of maximal `Instance` structures.
///
/// Each index in the pool provides enough space for a maximal `Instance`
/// structure depending on the limits used to create the pool.
///
/// The pool maintains a free list for fast instance allocation.
///
/// The userfault handler relies on how instances are stored in the mapping,
/// so make sure the uffd implementation is kept up-to-date.
#[derive(Debug)]
struct InstancePool {
    mapping: Mmap,
    instance_size: usize,
    max_instances: usize,
    index_allocator: Mutex<PoolingAllocationState>,
    memories: MemoryPool,
    tables: TablePool,
}

impl InstancePool {
    fn new(
        strategy: PoolingAllocationStrategy,
        instance_limits: &InstanceLimits,
        tunables: &Tunables,
    ) -> Result<Self> {
        let page_size = region::page::size();

        let instance_size = round_up_to_pow2(instance_limits.size, mem::align_of::<Instance>());

        let max_instances = instance_limits.count as usize;

        let allocation_size = round_up_to_pow2(
            instance_size
                .checked_mul(max_instances)
                .ok_or_else(|| anyhow!("total size of instance data exceeds addressable memory"))?,
            page_size,
        );

        let mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
            .context("failed to create instance pool mapping")?;

        let pool = Self {
            mapping,
            instance_size,
            max_instances,
            index_allocator: Mutex::new(PoolingAllocationState::new(strategy, max_instances)),
            memories: MemoryPool::new(instance_limits, tunables)?,
            tables: TablePool::new(instance_limits)?,
        };

        Ok(pool)
    }

    unsafe fn instance(&self, index: usize) -> &mut Instance {
        assert!(index < self.max_instances);
        &mut *(self.mapping.as_mut_ptr().add(index * self.instance_size) as *mut Instance)
    }

    unsafe fn initialize_instance(
        &self,
        instance_index: usize,
        req: InstanceAllocationRequest,
    ) -> Result<InstanceHandle, InstantiationError> {
        let module = req.runtime_info.module();

        // Before doing anything else ensure that our instance slot is actually
        // big enough to hold the `Instance` and `VMContext` for this instance.
        // If this fails then it's a configuration error at the `Engine` level
        // from when this pooling allocator was created and that needs updating
        // if this is to succeed.
        let offsets = self
            .validate_instance_size(module)
            .map_err(InstantiationError::Resource)?;

        let mut memories =
            PrimaryMap::with_capacity(module.memory_plans.len() - module.num_imported_memories);
        let mut tables =
            PrimaryMap::with_capacity(module.table_plans.len() - module.num_imported_tables);

        // If we fail to allocate the instance's resources, deallocate
        // what was successfully allocated and return before initializing the instance
        if let Err(e) = self.allocate_instance_resources(
            instance_index,
            req.runtime_info.as_ref(),
            req.store.as_raw(),
            &mut memories,
            &mut tables,
        ) {
            self.deallocate_memories(instance_index, &mut memories);
            self.deallocate_tables(instance_index, &mut tables);
            return Err(e);
        }

        let instance_ptr = self.instance(instance_index) as _;

        Instance::new_at(
            instance_ptr,
            self.instance_size,
            offsets,
            req,
            memories,
            tables,
        );

        Ok(InstanceHandle {
            instance: instance_ptr,
        })
    }

    fn allocate(
        &self,
        req: InstanceAllocationRequest,
    ) -> Result<InstanceHandle, InstantiationError> {
        let index = {
            let mut alloc = self.index_allocator.lock().unwrap();
            if alloc.is_empty() {
                return Err(InstantiationError::Limit(self.max_instances as u32));
            }
            alloc.alloc(req.runtime_info.unique_id()).index()
        };

        match unsafe { self.initialize_instance(index, req) } {
            Ok(handle) => Ok(handle),
            Err(e) => {
                // If we failed to initialize the instance, there's no need to drop
                // it as it was never "allocated", but we still need to free the
                // instance's slot.
                self.index_allocator.lock().unwrap().free(SlotId(index));
                Err(e)
            }
        }
    }

    fn deallocate(&self, handle: &InstanceHandle) {
        let addr = handle.instance as usize;
        let base = self.mapping.as_ptr() as usize;

        assert!(addr >= base && addr < base + self.mapping.len());
        assert!((addr - base) % self.instance_size == 0);

        let index = (addr - base) / self.instance_size;
        assert!(index < self.max_instances);

        let instance = unsafe { &mut *handle.instance };

        // Deallocate any resources used by the instance
        self.deallocate_memories(index, &mut instance.memories);
        self.deallocate_tables(index, &mut instance.tables);

        // We've now done all of the pooling-allocator-specific
        // teardown, so we can drop the Instance and let destructors
        // take care of any other fields (host state, globals, etc.).
        unsafe {
            std::ptr::drop_in_place(instance as *mut _);
        }
        // The instance is now uninitialized memory and cannot be
        // touched again until we write a fresh Instance in-place with
        // std::ptr::write in allocate() above.

        self.index_allocator.lock().unwrap().free(SlotId(index));
    }

    fn allocate_instance_resources(
        &self,
        instance_index: usize,
        runtime_info: &dyn ModuleRuntimeInfo,
        store: Option<*mut dyn Store>,
        memories: &mut PrimaryMap<DefinedMemoryIndex, Memory>,
        tables: &mut PrimaryMap<DefinedTableIndex, Table>,
    ) -> Result<(), InstantiationError> {
        self.allocate_memories(instance_index, runtime_info, store, memories)?;
        self.allocate_tables(instance_index, runtime_info, store, tables)?;

        Ok(())
    }

    fn allocate_memories(
        &self,
        instance_index: usize,
        runtime_info: &dyn ModuleRuntimeInfo,
        store: Option<*mut dyn Store>,
        memories: &mut PrimaryMap<DefinedMemoryIndex, Memory>,
    ) -> Result<(), InstantiationError> {
        let module = runtime_info.module();

        self.validate_memory_plans(module)
            .map_err(InstantiationError::Resource)?;

        for (memory_index, plan) in module
            .memory_plans
            .iter()
            .skip(module.num_imported_memories)
        {
            let defined_index = module
                .defined_memory_index(memory_index)
                .expect("should be a defined memory since we skipped imported ones");

            let memory = unsafe {
                std::slice::from_raw_parts_mut(
                    self.memories.get_base(instance_index, defined_index),
                    self.memories.max_memory_size,
                )
            };

            if let Some(image) = runtime_info
                .memory_image(defined_index)
                .map_err(|err| InstantiationError::Resource(err.into()))?
            {
                let mut slot = self
                    .memories
                    .take_memory_image_slot(instance_index, defined_index);
                let initial_size = plan.memory.minimum * WASM_PAGE_SIZE as u64;

                // If instantiation fails, we can propagate the error
                // upward and drop the slot. This will cause the Drop
                // handler to attempt to map the range with PROT_NONE
                // memory, to reserve the space while releasing any
                // stale mappings. The next use of this slot will then
                // create a new slot that will try to map over
                // this, returning errors as well if the mapping
                // errors persist. The unmap-on-drop is best effort;
                // if it fails, then we can still soundly continue
                // using the rest of the pool and allowing the rest of
                // the process to continue, because we never perform a
                // mmap that would leave an open space for someone
                // else to come in and map something.
                slot.instantiate(initial_size as usize, Some(image))
                    .map_err(|e| InstantiationError::Resource(e.into()))?;

                memories.push(
                    Memory::new_static(plan, memory, None, Some(slot), unsafe {
                        &mut *store.unwrap()
                    })
                    .map_err(InstantiationError::Resource)?,
                );
            } else {
                memories.push(
                    Memory::new_static(plan, memory, Some(commit_memory_pages), None, unsafe {
                        &mut *store.unwrap()
                    })
                    .map_err(InstantiationError::Resource)?,
                );
            }
        }

        Ok(())
    }

    fn deallocate_memories(
        &self,
        instance_index: usize,
        memories: &mut PrimaryMap<DefinedMemoryIndex, Memory>,
    ) {
        // Decommit any linear memories that were used
        for ((def_mem_idx, memory), base) in
            memories.iter_mut().zip(self.memories.get(instance_index))
        {
            let mut memory = mem::take(memory);
            assert!(memory.is_static());

            match memory {
                Memory::Static {
                    memory_image: Some(mut image),
                    ..
                } => {
                    // If there was any error clearing the image, just
                    // drop it here, and let the drop handler for the
                    // slot unmap in a way that retains the
                    // address space reservation.
                    if image.clear_and_remain_ready().is_ok() {
                        self.memories
                            .return_memory_image_slot(instance_index, def_mem_idx, image);
                    }
                }

                _ => {
                    // Reset any faulted guard pages as the physical
                    // memory may be reused for another instance in
                    // the future.
                    #[cfg(all(feature = "uffd", target_os = "linux"))]
                    memory
                        .reset_guard_pages()
                        .expect("failed to reset guard pages");
                    // require mutable on all platforms, not just uffd
                    drop(&mut memory);

                    let size = memory.byte_size();
                    drop(memory);
                    decommit_memory_pages(base, size)
                        .expect("failed to decommit linear memory pages");
                }
            }
        }
    }

    fn allocate_tables(
        &self,
        instance_index: usize,
        runtime_info: &dyn ModuleRuntimeInfo,
        store: Option<*mut dyn Store>,
        tables: &mut PrimaryMap<DefinedTableIndex, Table>,
    ) -> Result<(), InstantiationError> {
        let module = runtime_info.module();

        self.validate_table_plans(module)
            .map_err(InstantiationError::Resource)?;

        let mut bases = self.tables.get(instance_index);
        for (_, plan) in module.table_plans.iter().skip(module.num_imported_tables) {
            let base = bases.next().unwrap() as _;

            commit_table_pages(
                base as *mut u8,
                self.tables.max_elements as usize * mem::size_of::<*mut u8>(),
            )
            .map_err(InstantiationError::Resource)?;

            tables.push(
                Table::new_static(
                    plan,
                    unsafe {
                        std::slice::from_raw_parts_mut(base, self.tables.max_elements as usize)
                    },
                    unsafe { &mut *store.unwrap() },
                )
                .map_err(InstantiationError::Resource)?,
            );
        }

        Ok(())
    }

    fn deallocate_tables(
        &self,
        instance_index: usize,
        tables: &mut PrimaryMap<DefinedTableIndex, Table>,
    ) {
        // Decommit any tables that were used
        for (table, base) in tables.values_mut().zip(self.tables.get(instance_index)) {
            let table = mem::take(table);
            assert!(table.is_static());

            let size = round_up_to_pow2(
                table.size() as usize * mem::size_of::<*mut u8>(),
                self.tables.page_size,
            );

            drop(table);
            decommit_table_pages(base, size).expect("failed to decommit table pages");
        }
    }

    fn validate_table_plans(&self, module: &Module) -> Result<()> {
        let tables = module.table_plans.len() - module.num_imported_tables;
        if tables > self.tables.max_tables {
            bail!(
                "defined tables count of {} exceeds the limit of {}",
                tables,
                self.tables.max_tables,
            );
        }

        for (i, plan) in module.table_plans.iter().skip(module.num_imported_tables) {
            if plan.table.minimum > self.tables.max_elements {
                bail!(
                    "table index {} has a minimum element size of {} which exceeds the limit of {}",
                    i.as_u32(),
                    plan.table.minimum,
                    self.tables.max_elements,
                );
            }
        }
        Ok(())
    }

    fn validate_memory_plans(&self, module: &Module) -> Result<()> {
        let memories = module.memory_plans.len() - module.num_imported_memories;
        if memories > self.memories.max_memories {
            bail!(
                "defined memories count of {} exceeds the limit of {}",
                memories,
                self.memories.max_memories,
            );
        }

        for (i, plan) in module
            .memory_plans
            .iter()
            .skip(module.num_imported_memories)
        {
            let max = self.memories.max_memory_size / (WASM_PAGE_SIZE as usize);
            if plan.memory.minimum > (max as u64) {
                bail!(
                    "memory index {} has a minimum page size of {} which exceeds the limit of {}",
                    i.as_u32(),
                    plan.memory.minimum,
                    max,
                );
            }
        }
        Ok(())
    }

    fn validate_instance_size(&self, module: &Module) -> Result<VMOffsets<HostPtr>> {
        let offsets = VMOffsets::new(HostPtr, module);
        let layout = Instance::alloc_layout(&offsets);
        if layout.size() <= self.instance_size {
            return Ok(offsets);
        }

        // If this `module` exceeds the allocation size allotted to it then an
        // error will be reported here. The error of "required N bytes but
        // cannot allocate that" is pretty opaque, however, because it's not
        // clear what the breakdown of the N bytes are and what to optimize
        // next. To help provide a better error message here some fancy-ish
        // logic is done here to report the breakdown of the byte request into
        // the largest portions and where it's coming from.
        let mut message = format!(
            "instance allocation for this module \
             requires {} bytes which exceeds the configured maximum \
             of {} bytes; breakdown of allocation requirement:\n\n",
            layout.size(),
            self.instance_size,
        );

        let mut remaining = layout.size();
        let mut push = |name: &str, bytes: usize| {
            assert!(remaining >= bytes);
            remaining -= bytes;

            // If the `name` region is more than 5% of the allocation request
            // then report it here, otherwise ignore it. We have less than 20
            // fields so we're guaranteed that something should be reported, and
            // otherwise it's not particularly interesting to learn about 5
            // different fields that are all 8 or 0 bytes. Only try to report
            // the "major" sources of bytes here.
            if bytes > layout.size() / 20 {
                message.push_str(&format!(
                    " * {:.02}% - {} bytes - {}\n",
                    ((bytes as f32) / (layout.size() as f32)) * 100.0,
                    bytes,
                    name,
                ));
            }
        };

        // The `Instance` itself requires some size allocated to it.
        push("instance state management", mem::size_of::<Instance>());

        // Afterwards the `VMContext`'s regions are why we're requesting bytes,
        // so ask it for descriptions on each region's byte size.
        for (desc, size) in offsets.region_sizes() {
            push(desc, size as usize);
        }

        // double-check we accounted for all the bytes
        assert_eq!(remaining, 0);

        bail!("{}", message)
    }
}

/// Represents a pool of WebAssembly linear memories.
///
/// A linear memory is divided into accessible pages and guard pages.
///
/// Each instance index into the pool returns an iterator over the base addresses
/// of the instance's linear memories.
///
/// The userfault handler relies on how memories are stored in the mapping,
/// so make sure the uffd implementation is kept up-to-date.
#[derive(Debug)]
struct MemoryPool {
    mapping: Mmap,
    // If using a copy-on-write allocation scheme, the slot management. We
    // dynamically transfer ownership of a slot to a Memory when in
    // use.
    image_slots: Vec<Mutex<Option<MemoryImageSlot>>>,
    // The size, in bytes, of each linear memory's reservation plus the guard
    // region allocated for it.
    memory_reservation_size: usize,
    // The maximum size, in bytes, of each linear memory. Guaranteed to be a
    // whole number of wasm pages.
    max_memory_size: usize,
    // The size, in bytes, of the offset to the first linear memory in this
    // pool. This is here to help account for the first region of guard pages,
    // if desired, before the first linear memory.
    initial_memory_offset: usize,
    max_memories: usize,
    max_instances: usize,
}

impl MemoryPool {
    fn new(instance_limits: &InstanceLimits, tunables: &Tunables) -> Result<Self> {
        // The maximum module memory page count cannot exceed 65536 pages
        if instance_limits.memory_pages > 0x10000 {
            bail!(
                "module memory page limit of {} exceeds the maximum of 65536",
                instance_limits.memory_pages
            );
        }

        // The maximum module memory page count cannot exceed the memory reservation size
        if u64::from(instance_limits.memory_pages) > tunables.static_memory_bound {
            bail!(
                "module memory page limit of {} pages exceeds maximum static memory limit of {} pages",
                instance_limits.memory_pages,
                tunables.static_memory_bound,
            );
        }

        let memory_size = if instance_limits.memory_pages > 0 {
            usize::try_from(
                u64::from(tunables.static_memory_bound) * u64::from(WASM_PAGE_SIZE)
                    + tunables.static_memory_offset_guard_size,
            )
            .map_err(|_| anyhow!("memory reservation size exceeds addressable memory"))?
        } else {
            0
        };

        assert!(
            memory_size % region::page::size() == 0,
            "memory size {} is not a multiple of system page size",
            memory_size
        );

        let max_instances = instance_limits.count as usize;
        let max_memories = instance_limits.memories as usize;
        let initial_memory_offset = if tunables.guard_before_linear_memory {
            usize::try_from(tunables.static_memory_offset_guard_size).unwrap()
        } else {
            0
        };

        // The entire allocation here is the size of each memory times the
        // max memories per instance times the number of instances allowed in
        // this pool, plus guard regions.
        //
        // Note, though, that guard regions are required to be after each linear
        // memory. If the `guard_before_linear_memory` setting is specified,
        // then due to the contiguous layout of linear memories the guard pages
        // after one memory are also guard pages preceding the next linear
        // memory. This means that we only need to handle pre-guard-page sizes
        // specially for the first linear memory, hence the
        // `initial_memory_offset` variable here. If guards aren't specified
        // before linear memories this is set to `0`, otherwise it's set to
        // the same size as guard regions for other memories.
        let allocation_size = memory_size
            .checked_mul(max_memories)
            .and_then(|c| c.checked_mul(max_instances))
            .and_then(|c| c.checked_add(initial_memory_offset))
            .ok_or_else(|| {
                anyhow!("total size of memory reservation exceeds addressable memory")
            })?;

        // Create a completely inaccessible region to start
        let mapping = Mmap::accessible_reserved(0, allocation_size)
            .context("failed to create memory pool mapping")?;

        let num_image_slots = if cfg!(memory_init_cow) {
            max_instances * max_memories
        } else {
            0
        };
        let image_slots: Vec<_> = std::iter::repeat_with(|| Mutex::new(None))
            .take(num_image_slots)
            .collect();

        let pool = Self {
            mapping,
            image_slots,
            memory_reservation_size: memory_size,
            initial_memory_offset,
            max_memories,
            max_instances,
            max_memory_size: (instance_limits.memory_pages as usize) * (WASM_PAGE_SIZE as usize),
        };

        // uffd support requires some special setup for the memory pool
        #[cfg(all(feature = "uffd", target_os = "linux"))]
        initialize_memory_pool(&pool)?;

        Ok(pool)
    }

    fn get_base(&self, instance_index: usize, memory_index: DefinedMemoryIndex) -> *mut u8 {
        assert!(instance_index < self.max_instances);
        let memory_index = memory_index.as_u32() as usize;
        assert!(memory_index < self.max_memories);
        let idx = instance_index * self.max_memories + memory_index;
        let offset = self.initial_memory_offset + idx * self.memory_reservation_size;
        unsafe { self.mapping.as_mut_ptr().offset(offset as isize) }
    }

    fn get<'a>(&'a self, instance_index: usize) -> impl Iterator<Item = *mut u8> + 'a {
        (0..self.max_memories)
            .map(move |i| self.get_base(instance_index, DefinedMemoryIndex::from_u32(i as u32)))
    }

    /// Take ownership of the given image slot. Must be returned via
    /// `return_memory_image_slot` when the instance is done using it.
    fn take_memory_image_slot(
        &self,
        instance_index: usize,
        memory_index: DefinedMemoryIndex,
    ) -> MemoryImageSlot {
        let idx = instance_index * self.max_memories + (memory_index.as_u32() as usize);
        let maybe_slot = self.image_slots[idx].lock().unwrap().take();

        maybe_slot.unwrap_or_else(|| {
            MemoryImageSlot::create(
                self.get_base(instance_index, memory_index) as *mut c_void,
                0,
                self.max_memory_size,
            )
        })
    }

    /// Return ownership of the given image slot.
    fn return_memory_image_slot(
        &self,
        instance_index: usize,
        memory_index: DefinedMemoryIndex,
        slot: MemoryImageSlot,
    ) {
        assert!(!slot.is_dirty());
        let idx = instance_index * self.max_memories + (memory_index.as_u32() as usize);
        *self.image_slots[idx].lock().unwrap() = Some(slot);
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        // Clear the `clear_no_drop` flag (i.e., ask to *not* clear on
        // drop) for all slots, and then drop them here. This is
        // valid because the one `Mmap` that covers the whole region
        // can just do its one munmap.
        for mut slot in std::mem::take(&mut self.image_slots) {
            if let Some(slot) = slot.get_mut().unwrap() {
                slot.no_clear_on_drop();
            }
        }
    }
}

/// Represents a pool of WebAssembly tables.
///
/// Each instance index into the pool returns an iterator over the base addresses
/// of the instance's tables.
#[derive(Debug)]
struct TablePool {
    mapping: Mmap,
    table_size: usize,
    max_tables: usize,
    max_instances: usize,
    page_size: usize,
    max_elements: u32,
}

impl TablePool {
    fn new(instance_limits: &InstanceLimits) -> Result<Self> {
        let page_size = region::page::size();

        let table_size = round_up_to_pow2(
            mem::size_of::<*mut u8>()
                .checked_mul(instance_limits.table_elements as usize)
                .ok_or_else(|| anyhow!("table size exceeds addressable memory"))?,
            page_size,
        );

        let max_instances = instance_limits.count as usize;
        let max_tables = instance_limits.tables as usize;

        let allocation_size = table_size
            .checked_mul(max_tables)
            .and_then(|c| c.checked_mul(max_instances))
            .ok_or_else(|| anyhow!("total size of instance tables exceeds addressable memory"))?;

        let mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
            .context("failed to create table pool mapping")?;

        Ok(Self {
            mapping,
            table_size,
            max_tables,
            max_instances,
            page_size,
            max_elements: instance_limits.table_elements,
        })
    }

    fn get(&self, instance_index: usize) -> impl Iterator<Item = *mut u8> {
        assert!(instance_index < self.max_instances);

        let base: *mut u8 = unsafe {
            self.mapping
                .as_mut_ptr()
                .add(instance_index * self.table_size * self.max_tables) as _
        };

        let size = self.table_size;
        (0..self.max_tables).map(move |i| unsafe { base.add(i * size) })
    }
}

/// Represents a pool of execution stacks (used for the async fiber implementation).
///
/// Each index into the pool represents a single execution stack. The maximum number of
/// stacks is the same as the maximum number of instances.
///
/// As stacks grow downwards, each stack starts (lowest address) with a guard page
/// that can be used to detect stack overflow.
///
/// The top of the stack (starting stack pointer) is returned when a stack is allocated
/// from the pool.
#[cfg(all(feature = "async", unix))]
#[derive(Debug)]
struct StackPool {
    mapping: Mmap,
    stack_size: usize,
    max_instances: usize,
    page_size: usize,
    index_allocator: Mutex<PoolingAllocationState>,
}

#[cfg(all(feature = "async", unix))]
impl StackPool {
    fn new(instance_limits: &InstanceLimits, stack_size: usize) -> Result<Self> {
        let page_size = region::page::size();

        // Add a page to the stack size for the guard page when using fiber stacks
        let stack_size = if stack_size == 0 {
            0
        } else {
            round_up_to_pow2(stack_size, page_size)
                .checked_add(page_size)
                .ok_or_else(|| anyhow!("stack size exceeds addressable memory"))?
        };

        let max_instances = instance_limits.count as usize;

        let allocation_size = stack_size
            .checked_mul(max_instances)
            .ok_or_else(|| anyhow!("total size of execution stacks exceeds addressable memory"))?;

        let mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
            .context("failed to create stack pool mapping")?;

        // Set up the stack guard pages
        if allocation_size > 0 {
            unsafe {
                for i in 0..max_instances {
                    // Make the stack guard page inaccessible
                    let bottom_of_stack = mapping.as_mut_ptr().add(i * stack_size);
                    region::protect(bottom_of_stack, page_size, region::Protection::NONE)
                        .context("failed to protect stack guard page")?;
                }
            }
        }

        Ok(Self {
            mapping,
            stack_size,
            max_instances,
            page_size,
            // We always use a `NextAvailable` strategy for stack
            // allocation. We don't want or need an affinity policy
            // here: stacks do not benefit from being allocated to the
            // same compiled module with the same image (they always
            // start zeroed just the same for everyone).
            index_allocator: Mutex::new(PoolingAllocationState::new(
                PoolingAllocationStrategy::NextAvailable,
                max_instances,
            )),
        })
    }

    fn allocate(&self) -> Result<wasmtime_fiber::FiberStack, FiberStackError> {
        if self.stack_size == 0 {
            return Err(FiberStackError::NotSupported);
        }

        let index = {
            let mut alloc = self.index_allocator.lock().unwrap();
            if alloc.is_empty() {
                return Err(FiberStackError::Limit(self.max_instances as u32));
            }
            alloc.alloc(None).index()
        };

        assert!(index < self.max_instances);

        unsafe {
            // Remove the guard page from the size
            let size_without_guard = self.stack_size - self.page_size;

            let bottom_of_stack = self
                .mapping
                .as_mut_ptr()
                .add((index * self.stack_size) + self.page_size);

            commit_stack_pages(bottom_of_stack, size_without_guard)
                .map_err(FiberStackError::Resource)?;

            wasmtime_fiber::FiberStack::from_top_ptr(bottom_of_stack.add(size_without_guard))
                .map_err(|e| FiberStackError::Resource(e.into()))
        }
    }

    fn deallocate(&self, stack: &wasmtime_fiber::FiberStack) {
        let top = stack
            .top()
            .expect("fiber stack not allocated from the pool") as usize;

        let base = self.mapping.as_ptr() as usize;
        let len = self.mapping.len();
        assert!(
            top > base && top <= (base + len),
            "fiber stack top pointer not in range"
        );

        // Remove the guard page from the size
        let stack_size = self.stack_size - self.page_size;
        let bottom_of_stack = top - stack_size;
        let start_of_stack = bottom_of_stack - self.page_size;
        assert!(start_of_stack >= base && start_of_stack < (base + len));
        assert!((start_of_stack - base) % self.stack_size == 0);

        let index = (start_of_stack - base) / self.stack_size;
        assert!(index < self.max_instances);

        decommit_stack_pages(bottom_of_stack as _, stack_size).unwrap();

        self.index_allocator.lock().unwrap().free(SlotId(index));
    }
}

/// Implements the pooling instance allocator.
///
/// This allocator internally maintains pools of instances, memories, tables, and stacks.
///
/// Note: the resource pools are manually dropped so that the fault handler terminates correctly.
#[derive(Debug)]
pub struct PoolingInstanceAllocator {
    // This is manually drop so that the pools unmap their memory before the page fault handler drops.
    instances: mem::ManuallyDrop<InstancePool>,
    #[cfg(all(feature = "async", unix))]
    stacks: StackPool,
    #[cfg(all(feature = "async", windows))]
    stack_size: usize,
    #[cfg(all(feature = "uffd", target_os = "linux"))]
    _fault_handler: imp::PageFaultHandler,
}

impl PoolingInstanceAllocator {
    /// Creates a new pooling instance allocator with the given strategy and limits.
    pub fn new(
        strategy: PoolingAllocationStrategy,
        instance_limits: InstanceLimits,
        stack_size: usize,
        tunables: &Tunables,
    ) -> Result<Self> {
        if instance_limits.count == 0 {
            bail!("the instance count limit cannot be zero");
        }

        let instances = InstancePool::new(strategy, &instance_limits, tunables)?;

        #[cfg(all(feature = "uffd", target_os = "linux"))]
        let _fault_handler = imp::PageFaultHandler::new(&instances)?;

        drop(stack_size); // suppress unused warnings w/o async feature

        Ok(Self {
            instances: mem::ManuallyDrop::new(instances),
            #[cfg(all(feature = "async", unix))]
            stacks: StackPool::new(&instance_limits, stack_size)?,
            #[cfg(all(feature = "async", windows))]
            stack_size,
            #[cfg(all(feature = "uffd", target_os = "linux"))]
            _fault_handler,
        })
    }
}

impl Drop for PoolingInstanceAllocator {
    fn drop(&mut self) {
        // Manually drop the pools before the fault handler (if uffd is enabled)
        // This ensures that any fault handler thread monitoring the pool memory terminates
        unsafe {
            mem::ManuallyDrop::drop(&mut self.instances);
        }
    }
}

unsafe impl InstanceAllocator for PoolingInstanceAllocator {
    fn validate(&self, module: &Module) -> Result<()> {
        self.instances.validate_memory_plans(module)?;
        self.instances.validate_table_plans(module)?;

        // Note that this check is not 100% accurate for cross-compiled systems
        // where the pointer size may change since this check is often performed
        // at compile time instead of runtime. Given that Wasmtime is almost
        // always on a 64-bit platform though this is generally ok, and
        // otherwise this check also happens during instantiation to
        // double-check at that point.
        self.instances.validate_instance_size(module)?;

        Ok(())
    }

    fn adjust_tunables(&self, tunables: &mut Tunables) {
        // Treat the static memory bound as the maximum for unbounded Wasm memories
        // Because we guarantee a module cannot compile unless it fits in the limits of
        // the pool allocator, this ensures all memories are treated as static (i.e. immovable).
        tunables.static_memory_bound_is_maximum = true;
    }

    unsafe fn allocate(
        &self,
        req: InstanceAllocationRequest,
    ) -> Result<InstanceHandle, InstantiationError> {
        self.instances.allocate(req)
    }

    unsafe fn initialize(
        &self,
        handle: &mut InstanceHandle,
        module: &Module,
        is_bulk_memory: bool,
    ) -> Result<(), InstantiationError> {
        let instance = handle.instance_mut();

        cfg_if::cfg_if! {
            if #[cfg(all(feature = "uffd", target_os = "linux"))] {
                match &module.memory_initialization {
                    wasmtime_environ::MemoryInitialization::Paged { .. } => {
                        if !is_bulk_memory {
                            super::check_init_bounds(instance, module)?;
                        }

                        // Initialize the tables
                        super::initialize_tables(instance, module)?;

                        // Don't initialize the memory; the fault handler will back the pages when accessed

                        Ok(())
                    },
                    _ => initialize_instance(instance, module, is_bulk_memory)
                }
            } else {
                initialize_instance(instance, module, is_bulk_memory)
            }
        }
    }

    unsafe fn deallocate(&self, handle: &InstanceHandle) {
        self.instances.deallocate(handle);
    }

    #[cfg(all(feature = "async", unix))]
    fn allocate_fiber_stack(&self) -> Result<wasmtime_fiber::FiberStack, FiberStackError> {
        self.stacks.allocate()
    }

    #[cfg(all(feature = "async", unix))]
    unsafe fn deallocate_fiber_stack(&self, stack: &wasmtime_fiber::FiberStack) {
        self.stacks.deallocate(stack);
    }

    #[cfg(all(feature = "async", windows))]
    fn allocate_fiber_stack(&self) -> Result<wasmtime_fiber::FiberStack, FiberStackError> {
        if self.stack_size == 0 {
            return Err(FiberStackError::NotSupported);
        }

        // On windows, we don't use a stack pool as we use the native fiber implementation
        wasmtime_fiber::FiberStack::new(self.stack_size)
            .map_err(|e| FiberStackError::Resource(e.into()))
    }

    #[cfg(all(feature = "async", windows))]
    unsafe fn deallocate_fiber_stack(&self, _stack: &wasmtime_fiber::FiberStack) {
        // A no-op as we don't own the fiber stack on Windows
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{CompiledModuleId, Imports, MemoryImage, StorePtr, VMSharedSignatureIndex};
    use std::sync::Arc;
    use wasmtime_environ::{DefinedFuncIndex, DefinedMemoryIndex, FunctionInfo, SignatureIndex};

    pub(crate) fn empty_runtime_info(
        module: Arc<wasmtime_environ::Module>,
    ) -> Arc<dyn ModuleRuntimeInfo> {
        struct RuntimeInfo(Arc<wasmtime_environ::Module>);

        impl ModuleRuntimeInfo for RuntimeInfo {
            fn module(&self) -> &Arc<wasmtime_environ::Module> {
                &self.0
            }
            fn image_base(&self) -> usize {
                0
            }
            fn function_info(&self, _: DefinedFuncIndex) -> &FunctionInfo {
                unimplemented!()
            }
            fn signature(&self, _: SignatureIndex) -> VMSharedSignatureIndex {
                unimplemented!()
            }
            fn memory_image(
                &self,
                _: DefinedMemoryIndex,
            ) -> anyhow::Result<Option<&Arc<MemoryImage>>> {
                Ok(None)
            }

            fn unique_id(&self) -> Option<CompiledModuleId> {
                None
            }
            fn wasm_data(&self) -> &[u8] {
                &[]
            }
            fn signature_ids(&self) -> &[VMSharedSignatureIndex] {
                &[]
            }
        }

        Arc::new(RuntimeInfo(module))
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_instance_pool() -> Result<()> {
        let instance_limits = InstanceLimits {
            count: 3,
            tables: 1,
            memories: 1,
            table_elements: 10,
            size: 1000,
            memory_pages: 1,
            ..Default::default()
        };

        let instances = InstancePool::new(
            PoolingAllocationStrategy::NextAvailable,
            &instance_limits,
            &Tunables {
                static_memory_bound: 1,
                ..Tunables::default()
            },
        )?;

        assert_eq!(instances.instance_size, 1008); // round 1000 up to alignment
        assert_eq!(instances.max_instances, 3);

        assert_eq!(
            instances.index_allocator.lock().unwrap().testing_freelist(),
            &[SlotId(0), SlotId(1), SlotId(2)]
        );

        let mut handles = Vec::new();
        let module = Arc::new(Module::default());

        for _ in (0..3).rev() {
            handles.push(
                instances
                    .allocate(InstanceAllocationRequest {
                        runtime_info: &empty_runtime_info(module.clone()),
                        imports: Imports {
                            functions: &[],
                            tables: &[],
                            memories: &[],
                            globals: &[],
                        },
                        host_state: Box::new(()),
                        store: StorePtr::empty(),
                    })
                    .expect("allocation should succeed"),
            );
        }

        assert_eq!(
            instances.index_allocator.lock().unwrap().testing_freelist(),
            &[]
        );

        match instances.allocate(InstanceAllocationRequest {
            runtime_info: &empty_runtime_info(module),
            imports: Imports {
                functions: &[],
                tables: &[],
                memories: &[],
                globals: &[],
            },
            host_state: Box::new(()),
            store: StorePtr::empty(),
        }) {
            Err(InstantiationError::Limit(3)) => {}
            _ => panic!("unexpected error"),
        };

        for handle in handles.drain(..) {
            instances.deallocate(&handle);
        }

        assert_eq!(
            instances.index_allocator.lock().unwrap().testing_freelist(),
            &[SlotId(2), SlotId(1), SlotId(0)]
        );

        Ok(())
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_memory_pool() -> Result<()> {
        let pool = MemoryPool::new(
            &InstanceLimits {
                count: 5,
                tables: 0,
                memories: 3,
                table_elements: 0,
                memory_pages: 1,
                ..Default::default()
            },
            &Tunables {
                static_memory_bound: 1,
                static_memory_offset_guard_size: 0,
                ..Tunables::default()
            },
        )?;

        assert_eq!(pool.memory_reservation_size, WASM_PAGE_SIZE as usize);
        assert_eq!(pool.max_memories, 3);
        assert_eq!(pool.max_instances, 5);
        assert_eq!(pool.max_memory_size, WASM_PAGE_SIZE as usize);

        let base = pool.mapping.as_ptr() as usize;

        for i in 0..5 {
            let mut iter = pool.get(i);

            for j in 0..3 {
                assert_eq!(
                    iter.next().unwrap() as usize - base,
                    ((i * 3) + j) * pool.memory_reservation_size
                );
            }

            assert_eq!(iter.next(), None);
        }

        Ok(())
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_table_pool() -> Result<()> {
        let pool = TablePool::new(&InstanceLimits {
            count: 7,
            table_elements: 100,
            memory_pages: 0,
            tables: 4,
            memories: 0,
            ..Default::default()
        })?;

        let host_page_size = region::page::size();

        assert_eq!(pool.table_size, host_page_size);
        assert_eq!(pool.max_tables, 4);
        assert_eq!(pool.max_instances, 7);
        assert_eq!(pool.page_size, host_page_size);
        assert_eq!(pool.max_elements, 100);

        let base = pool.mapping.as_ptr() as usize;

        for i in 0..7 {
            let mut iter = pool.get(i);

            for j in 0..4 {
                assert_eq!(
                    iter.next().unwrap() as usize - base,
                    ((i * 4) + j) * pool.table_size
                );
            }

            assert_eq!(iter.next(), None);
        }

        Ok(())
    }

    #[cfg(all(unix, target_pointer_width = "64", feature = "async"))]
    #[test]
    fn test_stack_pool() -> Result<()> {
        let pool = StackPool::new(
            &InstanceLimits {
                count: 10,
                ..Default::default()
            },
            1,
        )?;

        let native_page_size = region::page::size();
        assert_eq!(pool.stack_size, 2 * native_page_size);
        assert_eq!(pool.max_instances, 10);
        assert_eq!(pool.page_size, native_page_size);

        assert_eq!(
            pool.index_allocator.lock().unwrap().testing_freelist(),
            &[
                SlotId(0),
                SlotId(1),
                SlotId(2),
                SlotId(3),
                SlotId(4),
                SlotId(5),
                SlotId(6),
                SlotId(7),
                SlotId(8),
                SlotId(9)
            ],
        );

        let base = pool.mapping.as_ptr() as usize;

        let mut stacks = Vec::new();
        for i in (0..10).rev() {
            let stack = pool.allocate().expect("allocation should succeed");
            assert_eq!(
                ((stack.top().unwrap() as usize - base) / pool.stack_size) - 1,
                i
            );
            stacks.push(stack);
        }

        assert_eq!(pool.index_allocator.lock().unwrap().testing_freelist(), &[]);

        match pool.allocate().unwrap_err() {
            FiberStackError::Limit(10) => {}
            _ => panic!("unexpected error"),
        };

        for stack in stacks {
            pool.deallocate(&stack);
        }

        assert_eq!(
            pool.index_allocator.lock().unwrap().testing_freelist(),
            &[
                SlotId(9),
                SlotId(8),
                SlotId(7),
                SlotId(6),
                SlotId(5),
                SlotId(4),
                SlotId(3),
                SlotId(2),
                SlotId(1),
                SlotId(0)
            ],
        );

        Ok(())
    }

    #[test]
    fn test_pooling_allocator_with_zero_instance_count() {
        assert_eq!(
            PoolingInstanceAllocator::new(
                PoolingAllocationStrategy::Random,
                InstanceLimits {
                    count: 0,
                    ..Default::default()
                },
                4096,
                &Tunables::default(),
            )
            .map_err(|e| e.to_string())
            .expect_err("expected a failure constructing instance allocator"),
            "the instance count limit cannot be zero"
        );
    }

    #[test]
    fn test_pooling_allocator_with_memory_pages_exceeded() {
        assert_eq!(
            PoolingInstanceAllocator::new(
                PoolingAllocationStrategy::Random,
                InstanceLimits {
                    count: 1,
                    memory_pages: 0x10001,
                    ..Default::default()
                },
                4096,
                &Tunables {
                    static_memory_bound: 1,
                    ..Tunables::default()
                },
            )
            .map_err(|e| e.to_string())
            .expect_err("expected a failure constructing instance allocator"),
            "module memory page limit of 65537 exceeds the maximum of 65536"
        );
    }

    #[test]
    fn test_pooling_allocator_with_reservation_size_exceeded() {
        assert_eq!(
            PoolingInstanceAllocator::new(
                PoolingAllocationStrategy::Random,
                InstanceLimits {
                    count: 1,
                    memory_pages: 2,
                    ..Default::default()
                },
                4096,
                &Tunables {
                    static_memory_bound: 1,
                    static_memory_offset_guard_size: 0,
                    ..Tunables::default()
                },
            )
            .map_err(|e| e.to_string())
            .expect_err("expected a failure constructing instance allocator"),
            "module memory page limit of 2 pages exceeds maximum static memory limit of 1 pages"
        );
    }

    #[cfg(all(unix, target_pointer_width = "64", feature = "async"))]
    #[test]
    fn test_stack_zeroed() -> Result<()> {
        let allocator = PoolingInstanceAllocator::new(
            PoolingAllocationStrategy::NextAvailable,
            InstanceLimits {
                count: 1,
                table_elements: 0,
                memory_pages: 0,
                tables: 0,
                memories: 0,
                ..Default::default()
            },
            4096,
            &Tunables::default(),
        )?;

        unsafe {
            for _ in 0..10 {
                let stack = allocator.allocate_fiber_stack()?;

                // The stack pointer is at the top, so decrement it first
                let addr = stack.top().unwrap().sub(1);

                assert_eq!(*addr, 0);
                *addr = 1;

                allocator.deallocate_fiber_stack(&stack);
            }
        }

        Ok(())
    }
}
