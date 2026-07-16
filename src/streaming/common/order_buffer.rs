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
            // Stable sort: events of one transaction share a tx_index and must keep parser order.
            events.sort_by_key(|(idx, _)| *idx);
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
            // Stable sort: events of one transaction share a tx_index and must keep parser order.
            events.sort_by_key(|(idx, _)| *idx);
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

    /// Push every event that belongs to a single `(slot, tx_index)` at once and return the
    /// events that are now ready to deliver in order.
    ///
    /// All events parsed out of one transaction share that transaction's `tx_index`, so they
    /// must be admitted as a group. Admitting them one-by-one would advance the per-`tx_index`
    /// watermark past `tx_index` after the first event, and every remaining event of the same
    /// transaction would then look "already delivered" and be dropped silently.
    pub fn push_streaming(
        &mut self,
        slot: u64,
        tx_index: u64,
        events: Vec<DexEvent>,
    ) -> Vec<DexEvent> {
        let mut result = Vec::new();
        if events.is_empty() {
            return result;
        }

        if slot > self.current_slot && self.current_slot > 0 {
            let keep_slots = self.slots.split_off(&slot);
            let flush_slots = std::mem::replace(&mut self.slots, keep_slots);
            result.reserve(flush_slots.values().map(Vec::len).sum());
            for (old_slot, mut buffered) in flush_slots {
                // Stable sort: events of one transaction share a tx_index and must keep parser order.
                buffered.sort_by_key(|(idx, _)| *idx);
                result.extend(buffered.into_iter().map(|(_, event)| event));
                self.streaming_watermarks.remove(&old_slot);
            }
        }

        if slot > self.current_slot {
            self.current_slot = slot;
        }

        let next_expected = *self.streaming_watermarks.get(&slot).unwrap_or(&0);

        if tx_index == next_expected {
            result.reserve(events.len());
            result.extend(events);
            let mut watermark = next_expected + 1;

            let remove_empty_slot = if let Some(buffered) = self.slots.get_mut(&slot) {
                // Stable sort: events of one transaction share a tx_index and must keep parser order.
                buffered.sort_by_key(|(idx, _)| *idx);
                // Drain whole transactions whose tx_index is contiguous with the watermark.
                // A single tx_index may hold several events, so advance the watermark per
                // distinct index rather than per event.
                let mut ready_count = 0;
                while ready_count < buffered.len() && buffered[ready_count].0 == watermark {
                    let idx = watermark;
                    while ready_count < buffered.len() && buffered[ready_count].0 == idx {
                        ready_count += 1;
                    }
                    watermark = idx + 1;
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
            let buffered = self.slots.entry(slot).or_default();
            buffered.reserve(events.len());
            for event in events {
                buffered.push((tx_index, event));
            }
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
            // Stable sort: events of one transaction share a tx_index and must keep parser order.
            events.sort_by_key(|(idx, _)| *idx);
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

        // Stable sort: events of one transaction share (slot, tx_index) and must keep parser order.
        self.events.sort_by_key(|(slot, tx_index, _)| (*slot, *tx_index));
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

        assert!(buffer.push_streaming(10, 3, vec![event(103)]).is_empty());
        assert!(buffer.push_streaming(10, 1, vec![event(101)]).is_empty());
        assert_eq!(ids(buffer.push_streaming(10, 0, vec![event(100)])), vec![100, 101]);
        assert_eq!(ids(buffer.push_streaming(10, 2, vec![event(102)])), vec![102, 103]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn streaming_order_keeps_every_event_of_a_multi_event_transaction() {
        let mut buffer = SlotBuffer::new();

        // One transaction (tx_index 0) parsed into three events must all be delivered.
        assert_eq!(
            ids(buffer.push_streaming(10, 0, vec![event(100), event(101), event(102)])),
            vec![100, 101, 102]
        );
        // A later transaction in the same slot still delivers in order.
        assert_eq!(ids(buffer.push_streaming(10, 1, vec![event(110)])), vec![110]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn streaming_order_releases_buffered_multi_event_transactions_in_order() {
        let mut buffer = SlotBuffer::new();

        // tx_index 1 arrives first (two events) and must wait for tx_index 0.
        assert!(buffer.push_streaming(10, 1, vec![event(110), event(111)]).is_empty());
        // tx_index 0 (two events) arrives and releases itself plus the buffered tx_index 1.
        assert_eq!(
            ids(buffer.push_streaming(10, 0, vec![event(100), event(101)])),
            vec![100, 101, 110, 111]
        );
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
