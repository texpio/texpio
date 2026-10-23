//! Explicit cross-process composition evidence; never read composition text.

use crate::engine::types::RefusalReason;
use windows::{
    Win32::{
        Foundation::S_OK,
        UI::Accessibility::{IUIAutomationTextEditPattern, IUIAutomationTextRange},
    },
    core::{HRESULT, Interface},
};

pub(super) fn require_idle(pattern: &IUIAutomationTextEditPattern) -> Result<(), RefusalReason> {
    let mut raw = std::ptr::null_mut();
    // SAFETY: a worker-owned UIA pattern writes one nullable, owned COM reference.
    // The generated convenience wrapper treats S_OK + NULL as an error, although
    // GetActiveComposition documents NULL as the successful no-conversion case.
    let status = unsafe { (pattern.vtable().GetActiveComposition)(pattern.as_raw(), &mut raw) };
    let range = if raw.is_null() {
        None
    } else {
        // SAFETY: take ownership of the returned reference, releasing it on every
        // path without querying or reading any composition text.
        Some(unsafe { IUIAutomationTextRange::from_raw(raw) })
    };
    composition_result(status, range)
}

fn composition_result<T>(status: HRESULT, range: Option<T>) -> Result<(), RefusalReason> {
    if status == S_OK && range.is_none() {
        Ok(())
    } else {
        Err(RefusalReason::Composition)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::RefusalReason;
    use std::cell::Cell;
    use windows::{
        Win32::Foundation::{E_FAIL, S_FALSE, S_OK},
        core::HRESULT,
    };

    struct OwnedRange<'a>(&'a Cell<usize>);
    impl Drop for OwnedRange<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn successful_null_range_is_positive_idle_evidence() {
        assert_eq!(composition_result::<()>(S_OK, None), Ok(()));
    }

    #[test]
    fn false_success_or_failure_with_no_range_is_not_idle_evidence() {
        for status in [S_FALSE, E_FAIL, HRESULT(2)] {
            assert_eq!(
                composition_result::<()>(status, None),
                Err(RefusalReason::Composition)
            );
        }
    }

    #[test]
    fn active_composition_refuses_and_releases_owned_range() {
        let drops = Cell::new(0);
        assert_eq!(
            composition_result(S_OK, Some(OwnedRange(&drops))),
            Err(RefusalReason::Composition)
        );
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn failed_query_still_releases_any_returned_range() {
        let drops = Cell::new(0);
        assert_eq!(
            composition_result(E_FAIL, Some(OwnedRange(&drops))),
            Err(RefusalReason::Composition)
        );
        assert_eq!(drops.get(), 1);
    }
}
