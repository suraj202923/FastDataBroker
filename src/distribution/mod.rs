// Phase 5: Multi-Region Distribution Module
pub mod multi_region;
pub mod tenant_storage;

pub use multi_region::{Region, RegionConfig, MultiRegionRouter, RegionAffinity};
pub use tenant_storage::{TenantStorage, StorageEvent};
