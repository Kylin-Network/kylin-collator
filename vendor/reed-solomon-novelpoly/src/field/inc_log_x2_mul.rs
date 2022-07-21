
use derive_more::{Add, AddAssign, BitXor, BitXorAssign, Sub, SubAssign};


/// Additive via XOR form of f2e16
#[derive(Clone, Copy, Debug, Default, BitXor, BitXorAssign, PartialEq, Eq)] // PartialOrd,Ord
pub struct Additive {
    pub adds: [Half::Additive; 2],
}

impl Additive {
    /*
    #[inline(always)]
	pub fn to_wide(self) -> Wide {
		self.0 as Wide
	}
    #[inline(always)]
	pub fn from_wide(x: Wide) -> Additive {
		Additive(x as Half)
	}
    */

	pub const ZERO: Additive = Additive { adds: [Half::Additive::ZERO; 2] };
}

#[cfg(table_bootstrap_complete)]
impl Additive {
	/// Return multiplier prepared form
    #[inline(always)]
	pub fn to_multiplier(self) -> Multiplier {
        let muls = [ self.adds[0].to_multiplier(), self.adds[0].to_multiplier() ];
        let mul_of_2nd_b_xor_1st = (muls[1].mul(Half::OMEGA) ^ muls[0]).to_multiplier();
		DoubleMultiplier { muls, mul_of_2nd_b_xor_1st }
	}

    #[inline(always)]
	pub fn mul(self, other: DoubleMultiplier) -> Additive {
		if self == Self::ZERO {
			return Self::ZERO;
		}
        let muls = [ self.adds[0].to_multiplier(), self.adds[0].to_multiplier() ];
        Additive { adds: [
            muls[0].mul(other.muls[0]) ^ muls[1].mul(other.muls[1]),
            mul_of_2nd_b_xor_1st.mul(other.muls[1]),
        ] }
	}

	/// Multiply field elements by a single multiplier, using SIMD if available
    #[inline(always)]
	pub fn mul_assign_slice(selfy: &mut [Self], other: DoubleMultiplier) {
		// TODO: SIMD
		for s in selfy {
			*s = s.mul(other);
		}
	}
}


/// Multiplicaiton friendly LOG form of f2e16
#[derive(Clone, Copy, Debug, Add, AddAssign, Sub, SubAssign, PartialEq, Eq)] // Default, PartialOrd,Ord
pub struct DoubleMultiplier {
    pub muls: [Half::Multiplier; 2],
    pub mul_of_2nd_b_xor_1st: Multiplier,
}

// impl DoubleMultiplier { }
    