use crate::streaming::event_parser::DexEvent;
use std::collections::{BTreeMap, HashMap};
use tokio::time::Instant;

#[derive(Default)]
pub struct SlotBuffer {
    slots: BTreeMap<u64, Vec<(u64, DexEvent)>>,
    current_slot: u64,
    last_flush_time: Option<Instant>,
    streaming_watermarks: HashMap<u64, u64>,
}

impl SlotBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: BTreeMap::new(),
            current_slot: 0,
            last_flush_time: Some(Instant::now()),
            streaming_watermarks: HashMap::new(),
        }
    }

    #[inline]
    pub fn push(&mut self, slot: u64, tx_index: u64, event: DexEvent) {
        if self.slots.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        self.slots.entry(slot).or_default().push((tx_index, event));
        if slot > self.current_slot {
            self.current_slot = slot;
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn flush_before(&mut self, current_slot: u64) -> Vec<DexEvent> {
        let slots_to_flush: Vec<u64> =
            self.slots.keys().filter(|&&s| s < current_slot).copied().collect();

        let mut result = Vec::with_capacity(slots_to_flush.len() * 4);
        for slot in slots_to_flush {
            if let Some(mut events) = self.slots.remove(&slot) {
                events.sort_unstable_by_key(|(idx, _)| *idx);
                result.extend(events.into_iter().map(|(_, event)| event));
            }
        }

        if !result.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        result
    }

    pub fn flush_all(&mut self) -> Vec<DexEvent> {
        let all_slots: Vec<u64> = self.slots.keys().copied().collect();
        let mut result = Vec::with_capacity(all_slots.len() * 4);

        for slot in all_slots {
            if let Some(mut events) = self.slots.remove(&slot) {
                events.sort_unstable_by_key(|(idx, _)| *idx);
                result.extend(events.into_iter().map(|(_, event)| event));
            }
        }

        if !result.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        result
    }

    #[inline]
    pub fn should_timeout(&self, timeout_ms: u64) -> bool {
        self.last_flush_time
            .map(|t| !self.slots.is_empty() && t.elapsed().as_millis() as u64 > timeout_ms)
            .unwrap_or(false)
    }

    pub fn push_streaming(&mut self, slot: u64, tx_index: u64, event: DexEvent) -> Vec<DexEvent> {
        let mut result = Vec::new();

        if slot > self.current_slot && self.current_slot > 0 {
            let old_slots: Vec<u64> = self.slots.keys().filter(|&&s| s < slot).copied().collect();
            for old_slot in old_slots {
                if let Some(mut events) = self.slots.remove(&old_slot) {
                    events.sort_unstable_by_key(|(idx, _)| *idx);
                    result.extend(events.into_iter().map(|(_, event)| event));
                }
                self.streaming_watermarks.remove(&old_slot);
            }
        }

        if slot > self.current_slot {
            self.current_slot = slot;
        }

        let next_expected = *self.streaming_watermarks.get(&slot).unwrap_or(&0);

        if tx_index == next_expected {
            result.push(event);
            let mut watermark = next_expected + 1;

            if let Some(buffered) = self.slots.get_mut(&slot) {
                buffered.sort_unstable_by_key(|(idx, _)| *idx);
                while let Some(pos) = buffered.iter().position(|(idx, _)| *idx == watermark) {
                    result.push(buffered.remove(pos).1);
                    watermark += 1;
                }
            }
            self.streaming_watermarks.insert(slot, watermark);
        } else if tx_index > next_expected {
            if self.slots.is_empty() {
                self.last_flush_time = Some(Instant::now());
            }
            self.slots.entry(slot).or_default().push((tx_index, event));
        }

        if !result.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        result
    }

    pub fn flush_streaming_timeout(&mut self) -> Vec<DexEvent> {
        let mut result = Vec::new();
        for (slot, mut events) in std::mem::take(&mut self.slots) {
            events.sort_unstable_by_key(|(idx, _)| *idx);
            result.extend(events.into_iter().map(|(_, event)| event));
            self.streaming_watermarks.remove(&slot);
        }
        if !result.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        result
    }
}

pub struct MicroBatchBuffer {
    events: Vec<(u64, u64, DexEvent)>,
    window_start_us: i64,
}

impl MicroBatchBuffer {
    #[inline]
    pub fn new() -> Self {
        Self { events: Vec::with_capacity(64), window_start_us: 0 }
    }

    #[inline]
    pub fn push(
        &mut self,
        slot: u64,
        tx_index: u64,
        event: DexEvent,
        now_us: i64,
        window_us: u64,
    ) -> bool {
        if self.events.is_empty() {
            self.window_start_us = now_us;
        }
        self.events.push((slot, tx_index, event));
        (now_us - self.window_start_us) as u64 >= window_us
    }

    #[inline]
    pub fn flush(&mut self) -> Vec<DexEvent> {
        if self.events.is_empty() {
            return Vec::new();
        }

        self.events.sort_unstable_by_key(|(slot, tx_index, _)| (*slot, *tx_index));
        let result =
            std::mem::take(&mut self.events).into_iter().map(|(_, _, event)| event).collect();
        self.window_start_us = 0;
        result
    }

    #[inline]
    pub fn should_flush(&self, now_us: i64, window_us: u64) -> bool {
        !self.events.is_empty() && (now_us - self.window_start_us) as u64 >= window_us
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for MicroBatchBuffer {
    fn default() -> Self {
        Self::new()
    }
}
