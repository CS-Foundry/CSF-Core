use anyhow::Result;
use std::collections::HashSet;
use tokio::sync::Mutex;

const JAILER_UID_RANGE_START: u32 = 60000;
const JAILER_UID_RANGE_END: u32 = 60999;

pub struct JailerIdAllocator {
    allocated: Mutex<HashSet<u32>>,
}

impl Default for JailerIdAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl JailerIdAllocator {
    pub fn new() -> Self {
        Self {
            allocated: Mutex::new(HashSet::new()),
        }
    }

    pub async fn allocate(&self) -> Result<u32> {
        let mut allocated = self.allocated.lock().await;

        for candidate in JAILER_UID_RANGE_START..=JAILER_UID_RANGE_END {
            if allocated.insert(candidate) {
                return Ok(candidate);
            }
        }

        anyhow::bail!("no free jailer uid available in range")
    }

    pub async fn mark_allocated(&self, id: u32) {
        self.allocated.lock().await.insert(id);
    }

    pub async fn release(&self, id: u32) {
        self.allocated.lock().await.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allocator_reuses_released_ids() {
        let allocator = JailerIdAllocator::new();

        let first = allocator.allocate().await.unwrap();
        allocator.release(first).await;
        let second = allocator.allocate().await.unwrap();

        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn allocator_does_not_double_allocate() {
        let allocator = JailerIdAllocator::new();

        let first = allocator.allocate().await.unwrap();
        let second = allocator.allocate().await.unwrap();

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn mark_allocated_prevents_reuse() {
        let allocator = JailerIdAllocator::new();

        allocator.mark_allocated(JAILER_UID_RANGE_START).await;
        let allocated = allocator.allocate().await.unwrap();

        assert_ne!(allocated, JAILER_UID_RANGE_START);
    }
}
