/// Simple object pool for reusing Vec allocations.
///
/// This module provides a basic memory pooling mechanism to reduce allocation overhead
/// for frequently allocated and deallocated vectors. The pool maintains a collection
/// of pre-allocated vectors that can be reused, reducing pressure on the system allocator.
///
/// # Performance Benefits
///
/// When vectors are frequently created and destroyed (e.g., in simulation loops),
/// reusing them from a pool can provide measurable performance benefits by:
/// - Reducing the number of heap allocations
/// - Minimizing memory fragmentation
/// - Decreasing time spent in the allocator
///
/// # Usage
///
/// ```rust,ignore
/// let mut pool = VecPool::new();
/// let mut vec = pool.acquire();
/// vec.push(42);
/// // ... use vec ...
/// pool.release(vec);
/// ```
use std::collections::VecDeque;

/// A simple pool for reusing `Vec<T>` allocations.
///
/// The pool stores cleared vectors that can be reused to avoid repeated
/// allocations. When a vector is no longer needed, it can be returned to
/// the pool for future reuse.
///
/// # Type Parameters
///
/// * `T` - The element type of the vectors in the pool
#[derive(Debug)]
pub struct VecPool<T> {
    pool: VecDeque<Vec<T>>,
    /// Maximum number of vectors to store in the pool.
    /// When this limit is reached, additional vectors returned via `release()`
    /// will be dropped instead of stored, preventing unbounded memory growth.
    max_capacity: usize,
}

impl<T> VecPool<T> {
    /// Creates a new empty vector pool with a maximum capacity.
    ///
    /// # Arguments
    ///
    /// * `max_capacity` - Maximum number of vectors to store in the pool.
    ///   When the pool reaches this size, additional vectors returned via
    ///   `release()` will be dropped instead of stored.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let pool: VecPool<i32> = VecPool::with_capacity(10);
    /// ```
    pub fn with_capacity(max_capacity: usize) -> Self {
        VecPool {
            pool: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// Creates a new empty vector pool with default capacity (16).
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Acquires a vector from the pool.
    ///
    /// If the pool is not empty, returns a reused (cleared) vector.
    /// Otherwise, allocates a new empty vector.
    ///
    /// # Returns
    ///
    /// An empty `Vec<T>` ready for use
    pub fn acquire(&mut self) -> Vec<T> {
        self.pool.pop_front().unwrap_or_default()
    }

    /// Returns a vector to the pool for future reuse.
    ///
    /// The vector is cleared before being stored in the pool.
    /// If the pool is at maximum capacity, the vector is dropped instead.
    ///
    /// # Arguments
    ///
    /// * `mut vec` - The vector to return to the pool
    pub fn release(&mut self, mut vec: Vec<T>) {
        if self.pool.len() < self.max_capacity {
            vec.clear();
            self.pool.push_back(vec);
        }
        // If pool is full, let vec drop (don't store it)
    }

    /// Returns the current number of vectors stored in the pool.
    pub fn len(&self) -> usize {
        self.pool.len()
    }

    /// Returns `true` if the pool contains no vectors.
    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

impl<T> Default for VecPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_and_release() {
        let mut pool: VecPool<i32> = VecPool::new();

        // Acquire a vector (should allocate new)
        let mut vec1 = pool.acquire();
        assert!(vec1.is_empty());
        vec1.push(42);
        vec1.push(43);

        // Release it back to the pool
        pool.release(vec1);
        assert_eq!(pool.len(), 1);

        // Acquire again (should reuse)
        let vec2 = pool.acquire();
        assert!(vec2.is_empty()); // Should be cleared
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_pool_max_capacity() {
        let mut pool: VecPool<i32> = VecPool::with_capacity(2);

        // Release 3 vectors
        pool.release(vec![1]);
        pool.release(vec![2]);
        pool.release(vec![3]); // This should be dropped (pool is full)

        // Pool should only contain 2 vectors
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_pool_empty() {
        let mut pool: VecPool<i32> = VecPool::new();
        assert!(pool.is_empty());

        pool.release(vec![1]);
        assert!(!pool.is_empty());

        pool.acquire();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_pool_acquire_when_empty() {
        let mut pool: VecPool<i32> = VecPool::new();

        // Acquiring from empty pool should work (allocates new)
        let vec = pool.acquire();
        assert!(vec.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_pool_reuse_preserves_capacity() {
        let mut pool: VecPool<i32> = VecPool::new();

        let mut vec = Vec::with_capacity(100);
        vec.push(1);
        vec.push(2);

        let capacity_before = vec.capacity();
        pool.release(vec);

        let vec2 = pool.acquire();
        // Capacity should be preserved even though vector was cleared
        assert_eq!(vec2.capacity(), capacity_before);
        assert!(vec2.is_empty());
    }
}
