//! Internal adapters keep the public version-1 observer exhaustive.
use super::{Event, EventV2, Observer, ObserverV2};

pub(super) trait Stream: Observer {
    const VERSION: u64;
    fn reset(&mut self, next_sequence: u64, live_nodes: u32) -> Result<(), Self::Error>;
}

pub(super) struct V1<'a, O>(pub &'a mut O);
impl<O: Observer> Observer for V1<'_, O> {
    type Error = O::Error;
    fn record(&mut self, event: Event) -> Result<(), Self::Error> {
        self.0.record(event)
    }
}
impl<O: Observer> Stream for V1<'_, O> {
    const VERSION: u64 = 1;
    fn reset(&mut self, _: u64, _: u32) -> Result<(), Self::Error> {
        // Capture skips collection export entirely for this profile.
        Ok(())
    }
}

pub(super) struct V2<'a, O>(pub &'a mut O);
impl<O: ObserverV2> Observer for V2<'_, O> {
    type Error = O::Error;
    fn record(&mut self, event: Event) -> Result<(), Self::Error> {
        self.0.record(EventV2::Event(event))
    }
}
impl<O: ObserverV2> Stream for V2<'_, O> {
    const VERSION: u64 = 2;
    fn reset(&mut self, next_sequence: u64, live_nodes: u32) -> Result<(), Self::Error> {
        self.0.record(EventV2::ArenaReset {
            next_sequence,
            live_nodes,
        })
    }
}
