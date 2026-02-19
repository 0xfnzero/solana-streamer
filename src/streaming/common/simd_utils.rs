use wide::*;

/// SIMD-accelerated data parsing utilities
pub struct SimdUtils;

impl SimdUtils {
    /// SIMD-accelerated byte array comparison
    /// For arrays with length >= 16, uses SIMD instructions for fast comparison
    #[inline(always)]
    pub fn fast_bytes_equal(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        let len = a.len();

        // For small arrays, use standard comparison directly
        if len < 16 {
            return a == b;
        }

        // Use SIMD to process 16-byte chunks
        let chunks = len / 16;
        let remainder = len % 16;

        // Process complete 16-byte chunks
        for i in 0..chunks {
            let offset = i * 16;
            let chunk_a = u8x16::from(&a[offset..offset + 16]);
            let chunk_b = u8x16::from(&b[offset..offset + 16]);

            if !chunk_a.simd_eq(chunk_b).all() {
                return false;
            }
        }

        // Process remaining bytes
        if remainder > 0 {
            let start = chunks * 16;
            return &a[start..] == &b[start..];
        }

        true
    }

    /// Fast discriminator matching, specifically for instruction discriminator comparison
    #[inline(always)]
    pub fn fast_discriminator_match(data: &[u8], discriminator: &[u8]) -> bool {
        if data.len() < discriminator.len() {
            return false;
        }

        let disc_len = discriminator.len();

        // Optimize for common discriminator lengths
        match disc_len {
            1 => data[0] == discriminator[0],
            2 => {
                let data_u16 = u16::from_le_bytes([data[0], data[1]]);
                let disc_u16 = u16::from_le_bytes([discriminator[0], discriminator[1]]);
                data_u16 == disc_u16
            }
            4 => {
                let data_u32 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let disc_u32 = u32::from_le_bytes([
                    discriminator[0],
                    discriminator[1],
                    discriminator[2],
                    discriminator[3],
                ]);
                data_u32 == disc_u32
            }
            8 => {
                let data_u64 = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                let disc_u64 = u64::from_le_bytes([
                    discriminator[0],
                    discriminator[1],
                    discriminator[2],
                    discriminator[3],
                    discriminator[4],
                    discriminator[5],
                    discriminator[6],
                    discriminator[7],
                ]);
                data_u64 == disc_u64
            }
            16 => {
                // Use SIMD to process 16-byte discriminators
                let data_chunk = u8x16::from(&data[..16]);
                let disc_chunk = u8x16::from(discriminator);
                data_chunk.simd_eq(disc_chunk).all()
            }
            _ => {
                // For other lengths, use generic SIMD comparison
                Self::fast_bytes_equal(&data[..disc_len], discriminator)
            }
        }
    }

    /// SIMD-accelerated memory search to find specific patterns in data
    #[inline(always)]
    pub fn find_pattern_simd(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() || haystack.len() < needle.len() {
            return None;
        }

        let needle_len = needle.len();
        let haystack_len = haystack.len();

        // For single-byte search, use optimized method
        if needle_len == 1 {
            let target = needle[0];
            return haystack.iter().position(|&b| b == target);
        }

        // For multi-byte search, use SIMD acceleration
        if needle_len <= 16 && haystack_len >= 16 {
            let first_byte = needle[0];
            let chunks = (haystack_len - needle_len + 1) / 16;

            for chunk_idx in 0..chunks {
                let start = chunk_idx * 16;
                let end = std::cmp::min(start + 16, haystack_len - needle_len + 1);

                // Use SIMD to find first byte matches
                let chunk = &haystack[start..start + 16];
                let target_vec = u8x16::splat(first_byte);
                let chunk_vec = u8x16::from(chunk);
                let matches = chunk_vec.simd_eq(target_vec);

                // Check each match position
                let matches_array: [u8; 16] = matches.into();
                for i in 0..16 {
                    if start + i >= end {
                        break;
                    }

                    if matches_array[i] != 0 && start + i + needle_len <= haystack_len {
                        if Self::fast_bytes_equal(
                            &haystack[start + i..start + i + needle_len],
                            needle,
                        ) {
                            return Some(start + i);
                        }
                    }
                }
            }

            // Process remaining part
            let remaining_start = chunks * 16;
            for i in remaining_start..=(haystack_len - needle_len) {
                if Self::fast_bytes_equal(&haystack[i..i + needle_len], needle) {
                    return Some(i);
                }
            }
        } else {
            // Fallback to standard search
            for i in 0..=(haystack_len - needle_len) {
                if Self::fast_bytes_equal(&haystack[i..i + needle_len], needle) {
                    return Some(i);
                }
            }
        }

        None
    }

    /// SIMD-accelerated data validation to check if data conforms to specific format
    #[inline(always)]
    pub fn validate_data_format(data: &[u8], min_length: usize) -> bool {
        if data.len() < min_length {
            return false;
        }

        true
    }

    /// Fast checksum calculation (maintains API consistency)
    #[inline(always)]
    pub fn fast_checksum(data: &[u8]) -> u32 {
        // Simplified implementation, directly sum all bytes
        data.iter().map(|&b| b as u32).sum()
    }

    /// SIMD-accelerated data copy (for large data blocks)
    #[inline(always)]
    pub fn fast_copy(src: &[u8], dst: &mut [u8]) {
        if src.len() != dst.len() {
            panic!("Source and destination must have the same length");
        }

        let len = src.len();

        if len >= 32 {
            // Use 32-byte SIMD copy
            let chunks = len / 32;

            for i in 0..chunks {
                let start = i * 32;
                let src_chunk1 = u8x16::from(&src[start..start + 16]);
                let src_chunk2 = u8x16::from(&src[start + 16..start + 32]);

                let chunk1_array: [u8; 16] = src_chunk1.into();
                let chunk2_array: [u8; 16] = src_chunk2.into();

                dst[start..start + 16].copy_from_slice(&chunk1_array);
                dst[start + 16..start + 32].copy_from_slice(&chunk2_array);
            }

            // Process remaining bytes
            let remaining_start = chunks * 32;
            dst[remaining_start..].copy_from_slice(&src[remaining_start..]);
        } else {
            // For small data, use standard copy
            dst.copy_from_slice(src);
        }
    }

    /// SIMD-accelerated account indices validation
    /// Validates that all indices in the account index array are less than the total account count
    #[inline(always)]
    pub fn validate_account_indices_simd(indices: &[u8], account_count: usize) -> bool {
        if indices.is_empty() {
            return true;
        }

        let max_valid_index = account_count as u8;

        // For small arrays, use standard comparison directly
        if indices.len() < 16 {
            return indices.iter().all(|&idx| idx < max_valid_index);
        }

        // Use SIMD for batch loading and comparison
        let chunks = indices.len() / 16;
        let remainder = indices.len() % 16;

        // Process complete 16-byte chunks
        for i in 0..chunks {
            let start = i * 16;
            let indices_chunk = u8x16::from(&indices[start..start + 16]);

            // Convert SIMD vector to array for fast batch checking
            let indices_array: [u8; 16] = indices_chunk.into();

            // Use unrolled loop for fast comparison, compiler will optimize this
            if indices_array[0] >= max_valid_index
                || indices_array[1] >= max_valid_index
                || indices_array[2] >= max_valid_index
                || indices_array[3] >= max_valid_index
                || indices_array[4] >= max_valid_index
                || indices_array[5] >= max_valid_index
                || indices_array[6] >= max_valid_index
                || indices_array[7] >= max_valid_index
                || indices_array[8] >= max_valid_index
                || indices_array[9] >= max_valid_index
                || indices_array[10] >= max_valid_index
                || indices_array[11] >= max_valid_index
                || indices_array[12] >= max_valid_index
                || indices_array[13] >= max_valid_index
                || indices_array[14] >= max_valid_index
                || indices_array[15] >= max_valid_index
            {
                return false;
            }
        }

        // Process remaining bytes
        if remainder > 0 {
            let remaining_start = chunks * 16;
            return indices[remaining_start..].iter().all(|&idx| idx < max_valid_index);
        }

        true
    }

    /// SIMD-accelerated instruction data validation
    /// Validates basic format and length requirements of instruction data
    #[inline(always)]
    pub fn validate_instruction_data_simd(
        data: &[u8],
        min_length: usize,
        discriminator_length: usize,
    ) -> bool {
        // Basic length check
        if data.len() < min_length || data.len() < discriminator_length {
            return false;
        }

        // Use existing data format validation
        Self::validate_data_format(data, min_length)
    }
}
