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
        let keep_slots = self.slots.split_off(&current_slot);
        let flush_slots = std::mem::replace(&mut self.slots, keep_slots);

        let mut result = Vec::with_capacity(flush_slots.values().map(Vec::len).sum());
        for (_slot, mut events) in flush_slots {
            events.sort_unstable_by_key(|(idx, _)| *idx);
            result.extend(events.into_iter().map(|(_, event)| event));
        }

        if !result.is_empty() {
            self.last_flush_time = Some(Instant::now());
        }
        result
    }

    pub fn flush_all(&mut self) -> Vec<DexEvent> {
        let all_slots = std::mem::take(&mut self.slots);
        let mut result = Vec::with_capacity(all_slots.values().map(Vec::len).sum());

        for (_slot, mut events) in all_slots {
            events.sort_unstable_by_key(|(idx, _)| *idx);
            result.extend(events.into_iter().map(|(_, event)| event));
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
            let keep_slots = self.slots.split_off(&slot);
            let flush_slots = std::mem::replace(&mut self.slots, keep_slots);
            result.reserve(flush_slots.values().map(Vec::len).sum());
            for (old_slot, mut events) in flush_slots {
                events.sort_unstable_by_key(|(idx, _)| *idx);
                result.extend(events.into_iter().map(|(_, event)| event));
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

            let remove_empty_slot = if let Some(buffered) = self.slots.get_mut(&slot) {
                buffered.sort_unstable_by_key(|(idx, _)| *idx);
                let mut ready_count = 0;
                while ready_count < buffered.len() && buffered[ready_count].0 == watermark {
                    watermark += 1;
                    ready_count += 1;
                }
                result.reserve(ready_count);
                for (_, event) in buffered.drain(..ready_count) {
                    result.push(event);
                }
                buffered.is_empty()
            } else {
                false
            };
            if remove_empty_slot {
                self.slots.remove(&slot);
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
        let flush_slots = std::mem::take(&mut self.slots);
        let mut result = Vec::with_capacity(flush_slots.values().map(Vec::len).sum());
        for (slot, mut events) in flush_slots {
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
        let mut result = Vec::with_capacity(self.events.len());
        result.extend(self.events.drain(..).map(|(_, _, event)| event));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event_parser::protocols::BlockMetaEvent;

    fn event(id: u64) -> DexEvent {
        DexEvent::BlockMetaEvent(BlockMetaEvent::new(id, id.to_string(), 0, 0))
    }

    fn ids(events: Vec<DexEvent>) -> Vec<u64> {
        events
            .into_iter()
            .map(|event| match event {
                DexEvent::BlockMetaEvent(event) => event.slot,
                _ => unreachable!("test only creates block meta events"),
            })
            .collect()
    }

    #[test]
    fn flush_before_keeps_newer_slots_and_sorts_flushed_events() {
        let mut buffer = SlotBuffer::new();
        buffer.push(7, 2, event(72));
        buffer.push(5, 1, event(51));
        buffer.push(5, 0, event(50));

        assert_eq!(ids(buffer.flush_before(6)), vec![50, 51]);
        assert_eq!(ids(buffer.flush_all()), vec![72]);
    }

    #[test]
    fn streaming_order_drains_only_contiguous_ready_prefix() {
        let mut buffer = SlotBuffer::new();

        assert!(buffer.push_streaming(10, 3, event(103)).is_empty());
        assert!(buffer.push_streaming(10, 1, event(101)).is_empty());
        assert_eq!(ids(buffer.push_streaming(10, 0, event(100))), vec![100, 101]);
        assert_eq!(ids(buffer.push_streaming(10, 2, event(102))), vec![102, 103]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn micro_batch_flush_sorts_and_reuses_allocation() {
        let mut buffer = MicroBatchBuffer::new();
        let initial_capacity = buffer.events.capacity();

        assert!(!buffer.push(2, 1, event(21), 0, 100));
        assert!(!buffer.push(1, 0, event(10), 10, 100));

        assert_eq!(ids(buffer.flush()), vec![10, 21]);
        assert!(buffer.events.capacity() >= initial_capacity);

        assert!(!buffer.push(3, 0, event(30), 200, 100));
        assert_eq!(ids(buffer.flush()), vec![30]);
    }
}
