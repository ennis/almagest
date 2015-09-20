
/// Variant keyword bitflags
// Note: some of them are mutually exclusive
bitflags! {
    flags Keywords: u64 {
        const POINT_LIGHT  			= 0b00000001,
        const DIRECTIONAL_LIGHT     = 0b00000010,
        const SPOT_LIGHT            = 0b00000100,
		const FORWARD_ADD           = 0b00001000,
		const SHADOWS_SIMPLE        = 0b00010000,
		const DEFERRED				= 0b00100000,
		const SHADOW     			= 0b01000000,
        const FORWARD_BASE          = 0b10000000
    }
}

#[derive(Copy, Clone)]
pub enum StdPass
{
    ForwardBase,
    ForwardAdd,
    Deferred,
    Shadow
}
