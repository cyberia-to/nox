//! Monomorphized engine boundary; NoObserver performs no capture work.
use super::super::*;
use core::convert::Infallible;

pub(in crate::sequential) enum RunError<E> {
    Execution(CompactionFailureKind),
    Capture(E),
}

pub(in crate::sequential) struct RunFailure<E> {
    pub kind: RunError<E>,
    pub stats: CompactionStats,
    pub peak_frames: u32,
}

impl<E> From<Error> for RunError<E> {
    fn from(error: Error) -> Self {
        Self::Execution(error.into())
    }
}
impl<E> From<CompactionFailureKind> for RunError<E> {
    fn from(error: CompactionFailureKind) -> Self {
        Self::Execution(error)
    }
}
impl<E> From<crate::data::reduction::compaction::Fault> for RunError<E> {
    fn from(error: crate::data::reduction::compaction::Fault) -> Self {
        Self::Execution(error.into())
    }
}

pub(in crate::sequential) trait Hook {
    type Error;
    type Before;
    fn begin<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        limits: CompactionLimits,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error>;
    fn collected<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error>;
    fn before<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        action: &Action,
        stack: &[Frame],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<Self::Before, Self::Error>;
    fn after<const N: usize>(
        &mut self,
        ar: &Reduction<N>,
        before: Self::Before,
        step: &compacting::Step,
        stack: &[Frame],
        cancelled: &mut impl FnMut() -> bool,
    ) -> Result<(), Self::Error>;
}

impl Hook for super::NoObserver {
    type Error = Infallible;
    type Before = ();
    #[inline(always)]
    fn begin<const N: usize>(
        &mut self,
        _: &Reduction<N>,
        _: &Action,
        _: CompactionLimits,
        _: &mut impl FnMut() -> bool,
    ) -> Result<(), Infallible> {
        Ok(())
    }
    #[inline(always)]
    fn collected<const N: usize>(
        &mut self,
        _: &Reduction<N>,
        _: &mut impl FnMut() -> bool,
    ) -> Result<(), Infallible> {
        Ok(())
    }
    #[inline(always)]
    fn before<const N: usize>(
        &mut self,
        _: &Reduction<N>,
        _: &Action,
        _: &[Frame],
        _: &mut impl FnMut() -> bool,
    ) -> Result<(), Infallible> {
        Ok(())
    }
    #[inline(always)]
    fn after<const N: usize>(
        &mut self,
        _: &Reduction<N>,
        _: (),
        _: &compacting::Step,
        _: &[Frame],
        _: &mut impl FnMut() -> bool,
    ) -> Result<(), Infallible> {
        Ok(())
    }
}
