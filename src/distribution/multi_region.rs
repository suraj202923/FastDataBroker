// Phase 5: Multi-Region Support for Distributed Deployment
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use anyhow::Result;

/// Geographic region definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Region {
    /// North America
    US_EAST,
    US_WEST,
    CANADA,
    /// Europe
    EU_WEST,
    EU_CENTRAL,
    /// Asia Pacific
    ASIA_SOUTHEAST,
    ASIA_NORTHEAST,
    /// Other regions
    Custom(String),
}

impl Region {
    /// Get region latency to other regions (estimated in ms)
    pub fn latency_to(&self, other: &Region) -> u32 {
        match (self, other) {
            (a, b) if a == b => 0,
            (Region::US_EAST, Region::US_WEST) => 50,
            (Region::US_WEST, Region::US_EAST) => 50,
            (Region::US_EAST, Region::CANADA) => 30,
            (Region::CANADA, Region::US_EAST) => 30,
            (Region::EU_WEST, Region::EU_CENTRAL) => 40,
            (Region::EU_CENTRAL, Region::EU_WEST) => 40,
            (Region::ASIA_SOUTHEAST, Region::ASIA_NORTHEAST) => 80,
            (Region::ASIA_NORTHEAST, Region::ASIA_SOUTHEAST) => 80,
            (Region::US_EAST, Region::EU_WEST) => 120,
            (Region::EU_WEST, Region::US_EAST) => 120,
            (Region::US_EAST, Region::ASIA_NORTHEAST) => 180,
            (Region::ASIA_NORTHEAST, Region::US_EAST) => 180,
            _ => 200, // Default high latency for unknown pairs
        }
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Region::US_EAST => write!(f, "us-east"),
            Region::US_WEST => write!(f, "us-west"),
            Region::CANADA => write!(f, "ca"),
            Region::EU_WEST => write!(f, "eu-west"),
            Region::EU_CENTRAL => write!(f, "eu-central"),
            Region::ASIA_SOUTHEAST => write!(f, "asia-se"),
            Region::ASIA_NORTHEAST => write!(f, "asia-ne"),
            Region::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Region-specific configuration
#[derive(Debug, Clone)]
pub struct RegionConfig {
    /// Region identifier
    pub region: Region,
    /// Endpoint/address for this region
    pub endpoint: String,
    /// Maximum connections
    pub max_connections: u32,
    /// Is primary region for this instance
    pub is_primary: bool,
    /// Replication factor (how many copies across regions)
    pub replication_factor: u32,
    /// Minimum replicas required for write confirmation
    pub min_acks: u32,
}

impl RegionConfig {
    /// Create new region config
    pub fn new(region: Region, endpoint: String, max_connections: u32) -> Self {
        Self {
            region,
            endpoint,
            max_connections,
            is_primary: false,
            replication_factor: 1,
            min_acks: 1,
        }
    }

    /// Set as primary region
    pub fn set_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    /// Set replication settings
    pub fn with_replication(mut self, factor: u32, min_acks: u32) -> Self {
        self.replication_factor = factor;
        self.min_acks = min_acks;
        self
    }
}

/// Region affinity for routing decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionAffinity {
    /// Route to closest region
    Closest,
    /// Route to primary region only
    Primary,
    /// Route to all regions (broadcast)
    All,
    /// Round-robin across regions
    RoundRobin,
    /// Least loaded region
    LeastLoaded,
}

/// Multi-region router
pub struct MultiRegionRouter {
    regions: Arc<RwLock<HashMap<Region, RegionConfig>>>,
    affinity: Arc<RwLock<RegionAffinity>>,
    round_robin_index: Arc<RwLock<usize>>,
}

impl MultiRegionRouter {
    /// Create new multi-region router
    pub fn new() -> Self {
        Self {
            regions: Arc::new(RwLock::new(HashMap::new())),
            affinity: Arc::new(RwLock::new(RegionAffinity::Closest)),
            round_robin_index: Arc::new(RwLock::new(0)),
        }
    }

    /// Register a region
    pub async fn register_region(&self, config: RegionConfig) -> Result<()> {
        let mut regions = self.regions.write().await;
        regions.insert(config.region.clone(), config);
        tracing::info!("Region registered: {}", regions.len());
        Ok(())
    }

    /// Remove a region
    pub async fn remove_region(&self, region: &Region) -> Result<()> {
        let mut regions = self.regions.write().await;
        regions.remove(region);
        tracing::info!("Region removed");
        Ok(())
    }

    /// Get all registered regions
    pub async fn get_regions(&self) -> Vec<Region> {
        self.regions.read().await.keys().cloned().collect()
    }

    /// Get specific region config
    pub async fn get_region_config(&self, region: &Region) -> Option<RegionConfig> {
        self.regions.read().await.get(region).cloned()
    }

    /// Set routing affinity
    pub async fn set_affinity(&self, affinity: RegionAffinity) {
        *self.affinity.write().await = affinity;
    }

    /// Get target regions for a message
    pub async fn get_target_regions(&self, origin_region: Option<&Region>) 
        -> Result<Vec<Region>> {
        
        let affinity = *self.affinity.read().await;
        let regions = self.regions.read().await;

        if regions.is_empty() {
            return Err(anyhow::anyhow!("No regions registered"));
        }

        match affinity {
            RegionAffinity::Primary => {
                let primary = regions
                    .values()
                    .find(|r| r.is_primary)
                    .map(|r| r.region.clone());
                
                Ok(primary.into_iter().collect())
            }

            RegionAffinity::All => {
                Ok(regions.keys().cloned().collect())
            }

            RegionAffinity::Closest => {
                if let Some(origin) = origin_region {
                    let mut sorted: Vec<_> = regions.keys().cloned().collect();
                    sorted.sort_by_key(|r| origin.latency_to(r));
                    Ok(sorted)
                } else {
                    Ok(regions.keys().cloned().collect())
                }
            }

            RegionAffinity::RoundRobin => {
                let regions_vec: Vec<_> = regions.keys().cloned().collect();
                let mut index = self.round_robin_index.write().await;
                let target = regions_vec[*index % regions_vec.len()].clone();
                *index += 1;
                Ok(vec![target])
            }

            RegionAffinity::LeastLoaded => {
                let mut sorted: Vec<(Region, u32)> = regions
                    .iter()
                    .map(|(r, config)| (r.clone(), config.max_connections))
                    .collect();
                sorted.sort_by_key(|(_, conns)| *conns);
                Ok(sorted.into_iter().map(|(r, _)| r).collect())
            }
        }
    }

    /// Check region health
    pub async fn check_region_health(&self, region: &Region) -> Result<RegionHealth> {
        if let Some(config) = self.regions.read().await.get(region) {
            Ok(RegionHealth {
                region: region.clone(),
                endpoint: config.endpoint.clone(),
                is_healthy: true, // In real implementation, would ping endpoints
                latency_ms: 0,
                last_checked: chrono::Utc::now().timestamp(),
            })
        } else {
            Err(anyhow::anyhow!("Region not found"))
        }
    }

    /// Get replication topology for a message
    pub async fn get_replication_topology(&self, origin_region: &Region) 
        -> Result<ReplicationTopology> {
        
        let regions = self.get_target_regions(Some(origin_region)).await?;
        let config = self.get_region_config(origin_region).await
            .ok_or_else(|| anyhow::anyhow!("Origin region not found"))?;

        Ok(ReplicationTopology {
            primary: origin_region.clone(),
            replicas: regions.into_iter()
                .filter(|r| r != origin_region)
                .collect(),
            replication_factor: config.replication_factor,
            min_acks: config.min_acks,
        })
    }
}

impl Default for MultiRegionRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for MultiRegionRouter {
    fn clone(&self) -> Self {
        Self {
            regions: self.regions.clone(),
            affinity: self.affinity.clone(),
            round_robin_index: self.round_robin_index.clone(),
        }
    }
}

/// Region health status
#[derive(Debug, Clone)]
pub struct RegionHealth {
    pub region: Region,
    pub endpoint: String,
    pub is_healthy: bool,
    pub latency_ms: u32,
    pub last_checked: i64,
}

/// Replication topology for messages
#[derive(Debug, Clone)]
pub struct ReplicationTopology {
    pub primary: Region,
    pub replicas: Vec<Region>,
    pub replication_factor: u32,
    pub min_acks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_latency() {
        let us_east = Region::US_EAST;
        let us_west = Region::US_WEST;

        assert_eq!(us_east.latency_to(&us_east), 0);
        assert_eq!(us_east.latency_to(&us_west), 50);
        assert_eq!(us_west.latency_to(&us_east), 50);
    }

    #[test]
    fn test_region_display() {
        assert_eq!(Region::US_EAST.to_string(), "us-east");
        assert_eq!(Region::EU_WEST.to_string(), "eu-west");
        assert_eq!(Region::Custom("my-region".to_string()).to_string(), "my-region");
    }

    #[test]
    fn test_region_config() {
        let config = RegionConfig::new(
            Region::US_EAST,
            "localhost:6379".to_string(),
            1000,
        );
        assert_eq!(config.region, Region::US_EAST);
        assert!(!config.is_primary);
    }

    #[tokio::test]
    async fn test_multi_region_router_register() {
        let router = MultiRegionRouter::new();
        let config = RegionConfig::new(
            Region::US_EAST,
            "localhost:6379".to_string(),
            1000,
        );
        assert!(router.register_region(config).await.is_ok());

        let regions = router.get_regions().await;
        assert_eq!(regions.len(), 1);
    }

    #[tokio::test]
    async fn test_multi_region_router_affinity_primary() {
        let router = MultiRegionRouter::new();
        let mut config = RegionConfig::new(
            Region::US_EAST,
            "localhost:6379".to_string(),
            1000,
        );
        config.is_primary = true;
        router.register_region(config).await.unwrap();

        router.set_affinity(RegionAffinity::Primary).await;
        let targets = router.get_target_regions(None).await.unwrap();
        assert_eq!(targets, vec![Region::US_EAST]);
    }

    #[tokio::test]
    async fn test_multi_region_router_affinity_all() {
        let router = MultiRegionRouter::new();
        router.register_region(
            RegionConfig::new(Region::US_EAST, "ep1".to_string(), 1000)
        ).await.unwrap();
        router.register_region(
            RegionConfig::new(Region::EU_WEST, "ep2".to_string(), 1000)
        ).await.unwrap();

        router.set_affinity(RegionAffinity::All).await;
        let targets = router.get_target_regions(None).await.unwrap();
        assert_eq!(targets.len(), 2);
    }
}
