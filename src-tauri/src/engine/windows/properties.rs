use super::super::guard::ExplicitBool;
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::System::Variant::{VARIANT, VT_BOOL, VariantClear};

/// UIA defaults and integer zero are not evidence that a control is non-password.
pub fn explicit_bool(value: &VARIANT, not_supported: bool) -> ExplicitBool {
    if not_supported {
        return ExplicitBool::Unsupported;
    }
    // SAFETY: inspect the discriminator before accessing the corresponding member.
    unsafe {
        if value.Anonymous.Anonymous.vt != VT_BOOL {
            return ExplicitBool::Unsupported;
        }
        match value.Anonymous.Anonymous.Anonymous.boolVal {
            VARIANT_FALSE => ExplicitBool::Known(false),
            VARIANT_TRUE => ExplicitBool::Known(true),
            _ => ExplicitBool::Unsupported,
        }
    }
}

pub(super) struct OwnedVariant(pub VARIANT);

impl Drop for OwnedVariant {
    fn drop(&mut self) {
        // SAFETY: this wrapper uniquely owns the returned VARIANT, cleared once.
        unsafe {
            let _ = VariantClear(&mut self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Variant::{VARIANT, VT_BOOL, VT_I4};

    #[test]
    fn only_explicit_variant_boolean_counts_as_false() {
        let mut value = VARIANT::default();
        unsafe {
            (*value.Anonymous.Anonymous).vt = VT_BOOL;
            (*value.Anonymous.Anonymous).Anonymous.boolVal =
                windows::Win32::Foundation::VARIANT_FALSE;
        }
        assert_eq!(explicit_bool(&value, false), ExplicitBool::Known(false));
        assert_eq!(explicit_bool(&value, true), ExplicitBool::Unsupported);
        unsafe {
            (*value.Anonymous.Anonymous).vt = VT_I4;
            (*value.Anonymous.Anonymous).Anonymous.lVal = 0;
        }
        assert_eq!(explicit_bool(&value, false), ExplicitBool::Unsupported);
    }
}
