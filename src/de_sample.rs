use crate::*;
use serde::Deserialize;
use serde_with::*;

pub const VARS: usize = 3;

pub type DeTrace<const N: usize> = Vec<[bool; N]>;

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct DeSample<const N: usize> {
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub positive_traces: Vec<DeTrace<N>>,
    #[serde_as(as = "Vec<Vec<[_; N]>>")]
    pub negative_traces: Vec<DeTrace<N>>,
}

impl<const N: usize> DeSample<N> {
    pub fn into_sample<'a>(&'a self) -> Sample<'a, N> {
        Sample {
            positive_traces: self
                .positive_traces
                .iter()
                .map(|b| &(b[..]))
                .collect::<Vec<&[[bool; N]]>>(),
            negative_traces: self
                .negative_traces
                .iter()
                .map(|b| &(b[..]))
                .collect::<Vec<&[[bool; N]]>>(),
        }
    }
}

pub fn load_and_solve(contents: Vec<u8>) -> Option<SyntaxTree> {
    // Ugly hack to get around limitations of deserialization for types with const generics.
    (1..=5)
        .into_iter()
        .find_map(|n| {
            match n {
                1 => ron::de::from_bytes::<DeSample<1>>(&contents)
                    .map(|de_sample| par_brute_solve(&de_sample.into_sample(), true)),
                2 => ron::de::from_bytes::<DeSample<2>>(&contents)
                    .map(|de_sample| par_brute_solve(&de_sample.into_sample(), true)),
                3 => ron::de::from_bytes::<DeSample<3>>(&contents)
                    .map(|de_sample| par_brute_solve(&de_sample.into_sample(), true)),
                4 => ron::de::from_bytes::<DeSample<4>>(&contents)
                    .map(|de_sample| par_brute_solve(&de_sample.into_sample(), true)),
                5 => ron::de::from_bytes::<DeSample<5>>(&contents)
                    .map(|de_sample| par_brute_solve(&de_sample.into_sample(), true)),
                _ => panic!("out-of-bound parameter"),
            }
            .ok()
            .flatten()
        })
        .map(|arc| (*arc).clone())
}
