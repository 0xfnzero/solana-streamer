use dashmap::DashMap;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

const MAX_SIGNATURES: usize = 1000;
const CLEANUP_BATCH_SIZE: usize = 100;

/// Signature-based trader addresses, completely lock-free
#[derive(Default)]
struct SignatureAddresses {
    /// Developer addresses for this signature
    dev_addresses: BTreeSet<Pubkey>,
    /// Bonk developer addresses for this signature  
    bonk_dev_addresses: BTreeSet<Pubkey>,
}

/// High-performance global state with lock-free signature-based storage
pub struct GlobalState {
    /// Signature -> trader addresses mapping (lock-free concurrent hashmap)
    signature_data: DashMap<Signature, SignatureAddresses>,
    /// Current signature count for capacity management
    signature_count: AtomicUsize,
    /// Generation counter to handle cleanup races
    generation: AtomicU64,
}

impl GlobalState {
    /// Create a new high-performance global state instance
    pub fn new() -> Self {
        Self {
            signature_data: DashMap::new(),
            signature_count: AtomicUsize::new(0),
            generation: AtomicU64::new(0),
        }
    }

    /// Lock-free capacity management - cleanup old signatures when limit exceeded
    fn maybe_cleanup(&self) {
        let current_count = self.signature_count.load(Ordering::Relaxed);
        if current_count <= MAX_SIGNATURES {
            return;
        }

        // Use CAS to ensure only one thread performs cleanup
        let gen = self.generation.load(Ordering::Relaxed);
        if self
            .generation
            .compare_exchange_weak(gen, gen + 1, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return; // Another thread is cleaning up
        }

        // Collect only the batch we need to remove (avoid allocating full list)
        let signatures_to_remove: Vec<Signature> =
            self.signature_data.iter().take(CLEANUP_BATCH_SIZE).map(|entry| *entry.key()).collect();

        // Remove old signatures atomically; only decrement count when entry was present
        for signature in signatures_to_remove {
            if self.signature_data.remove(&signature).is_some() {
                self.signature_count.fetch_sub(1, Ordering::Relaxed);
            }
        }
    }

    /// Add developer address for a specific signature (lock-free)
    pub fn add_dev_address(&self, signature: &Signature, address: Pubkey) {
        self.maybe_cleanup();

        self.signature_data
            .entry(*signature)
            .and_modify(|addresses| {
                addresses.dev_addresses.insert(address);
            })
            .or_insert_with(|| {
                self.signature_count.fetch_add(1, Ordering::Relaxed);
                let mut sig_addr = SignatureAddresses::default();
                sig_addr.dev_addresses.insert(address);
                sig_addr
            });
    }

    /// Add Bonk developer address for a specific signature (lock-free)
    pub fn add_bonk_dev_address(&self, signature: &Signature, address: Pubkey) {
        self.maybe_cleanup();

        self.signature_data
            .entry(*signature)
            .and_modify(|addresses| {
                addresses.bonk_dev_addresses.insert(address);
            })
            .or_insert_with(|| {
                self.signature_count.fetch_add(1, Ordering::Relaxed);
                let mut sig_addr = SignatureAddresses::default();
                sig_addr.bonk_dev_addresses.insert(address);
                sig_addr
            });
    }

    /// High-performance: Check if address is a developer address in specific signature (O(log m))
    pub fn is_dev_address_in_signature(&self, signature: &Signature, address: &Pubkey) -> bool {
        self.signature_data
            .get(signature)
            .map(|entry| entry.dev_addresses.contains(address))
            .unwrap_or(false)
    }

    /// High-performance: Check if address is a Bonk developer address in specific signature (O(log m))
    pub fn is_bonk_dev_address_in_signature(
        &self,
        signature: &Signature,
        address: &Pubkey,
    ) -> bool {
        self.signature_data
            .get(signature)
            .map(|entry| entry.bonk_dev_addresses.contains(address))
            .unwrap_or(false)
    }

    /// Check if address is a developer address in any signature (lock-free scan, slower)
    pub fn is_dev_address(&self, address: &Pubkey) -> bool {
        self.signature_data.iter().any(|entry| entry.dev_addresses.contains(address))
    }

    /// Check if address is a Bonk developer address in any signature (lock-free scan, slower)
    pub fn is_bonk_dev_address(&self, address: &Pubkey) -> bool {
        self.signature_data.iter().any(|entry| entry.bonk_dev_addresses.contains(address))
    }

    /// Get all developer addresses from all signatures (lock-free aggregation)
    pub fn get_dev_addresses(&self) -> Vec<Pubkey> {
        let mut all_addresses = BTreeSet::new();
        for entry in self.signature_data.iter() {
            for addr in &entry.dev_addresses {
                all_addresses.insert(*addr);
            }
        }
        all_addresses.into_iter().collect()
    }

    /// Get all Bonk developer addresses from all signatures (lock-free aggregation)
    pub fn get_bonk_dev_addresses(&self) -> Vec<Pubkey> {
        let mut all_addresses = BTreeSet::new();
        for entry in self.signature_data.iter() {
            for addr in &entry.bonk_dev_addresses {
                all_addresses.insert(*addr);
            }
        }
        all_addresses.into_iter().collect()
    }

    /// Get developer addresses for a specific signature
    pub fn get_dev_addresses_for_signature(&self, signature: &Signature) -> Vec<Pubkey> {
        self.signature_data
            .get(signature)
            .map(|entry| entry.dev_addresses.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get Bonk developer addresses for a specific signature
    pub fn get_bonk_dev_addresses_for_signature(&self, signature: &Signature) -> Vec<Pubkey> {
        self.signature_data
            .get(signature)
            .map(|entry| entry.bonk_dev_addresses.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Get current signature count
    pub fn get_signature_count(&self) -> usize {
        self.signature_count.load(Ordering::Relaxed)
    }

    /// Clear all data (lock-free)
    pub fn clear_all_data(&self) {
        self.signature_data.clear();
        self.signature_count.store(0, Ordering::Relaxed);
        self.generation.store(0, Ordering::Relaxed);
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Global state instance
static GLOBAL_STATE: std::sync::LazyLock<GlobalState> = std::sync::LazyLock::new(GlobalState::new);

/// Get global state instance
pub fn get_global_state() -> &'static GlobalState {
    &GLOBAL_STATE
}

/// Convenience function: Add developer address for a specific signature
pub fn add_dev_address(signature: &Signature, address: Pubkey) {
    get_global_state().add_dev_address(signature, address);
}

/// Convenience function: Check if address is a developer address
pub fn is_dev_address(address: &Pubkey) -> bool {
    get_global_state().is_dev_address(address)
}

/// Convenience function: Add Bonk developer address for a specific signature
pub fn add_bonk_dev_address(signature: &Signature, address: Pubkey) {
    get_global_state().add_bonk_dev_address(signature, address);
}

/// Convenience function: Check if address is a Bonk developer address
pub fn is_bonk_dev_address(address: &Pubkey) -> bool {
    get_global_state().is_bonk_dev_address(address)
}

/// Convenience function: Get all developer addresses
pub fn get_dev_addresses() -> Vec<Pubkey> {
    get_global_state().get_dev_addresses()
}

/// Convenience function: Get all Bonk developer addresses
pub fn get_bonk_dev_addresses() -> Vec<Pubkey> {
    get_global_state().get_bonk_dev_addresses()
}

/// Convenience function: Get developer addresses for a specific signature
pub fn get_dev_addresses_for_signature(signature: &Signature) -> Vec<Pubkey> {
    get_global_state().get_dev_addresses_for_signature(signature)
}

/// Convenience function: Get Bonk developer addresses for a specific signature
pub fn get_bonk_dev_addresses_for_signature(signature: &Signature) -> Vec<Pubkey> {
    get_global_state().get_bonk_dev_addresses_for_signature(signature)
}

/// Convenience function: Get current signature count
pub fn get_signature_count() -> usize {
    get_global_state().get_signature_count()
}

/// High-performance: Check if address is a developer address in specific signature
pub fn is_dev_address_in_signature(signature: &Signature, address: &Pubkey) -> bool {
    get_global_state().is_dev_address_in_signature(signature, address)
}

/// High-performance: Check if address is a Bonk developer address in specific signature
pub fn is_bonk_dev_address_in_signature(signature: &Signature, address: &Pubkey) -> bool {
    get_global_state().is_bonk_dev_address_in_signature(signature, address)
}
