#![no_std]

// use core::intrinsics::unchecked_mul;
use allocator::{BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    // 五个字段
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = start + size;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> allocator::AllocResult {
        if start < self.start || start + size > self.end || start + size > self.p_pos {
            return Err(allocator::AllocError::NoMemory);
        }
        self.end += size;
        self.p_pos += size;

        todo!()
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<core::ptr::NonNull<u8>> {
        let align = layout.align();
        let size = layout.size();

        // Align the b_pos to the required alignment
        let aligned_b_pos = self.b_pos
            .checked_add(align - 1)
            .map(|addr| addr & !(align - 1))
            .ok_or(allocator::AllocError::NoMemory)?;

        // Check if there is enough space
        let new_b_pos = aligned_b_pos
            .checked_add(size)
            .filter(|&end| end <= self.p_pos)
            .ok_or(allocator::AllocError::NoMemory)?;

        let ptr = core::ptr::NonNull::new(aligned_b_pos as *mut u8)
            .ok_or(allocator::AllocError::NoMemory)?;


        // Update b_pos and count
        self.b_pos = new_b_pos;
        self.count = self.count.checked_add(1)
            .ok_or(allocator::AllocError::NoMemory)?;

        // Return the allocated pointer
        Ok(ptr)
    }

    fn dealloc(&mut self, pos: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        // Decrement the count
        assert!(self.count > 0);

        // If count reaches zero, reset b_pos to start
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        let align = align_pow2;
        let size = num_pages * Self::PAGE_SIZE;

        // Align the p_pos to the required alignment
        let aligned_p_pos = self.p_pos
            .checked_sub(size)
            .ok_or(allocator::AllocError::NoMemory)?
            & !(align - 1);

        // Check if there is enough space
        if aligned_p_pos < self.b_pos {
            return Err(allocator::AllocError::NoMemory);
        }

        // Update p_pos
        self.p_pos = aligned_p_pos;

        // Return the allocated page address
        Ok(aligned_p_pos)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // For pages area, it will never be freed!
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / Self::PAGE_SIZE
    }
}