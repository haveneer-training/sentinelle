// Module 5, optimisation O4 : un hachage FNV-1a, sans dépendance.
//
// SipHash (le hachage par défaut de `HashMap`) résiste aux collisions provoquées
// (HashDoS) : un attaquant qui contrôle les clés ne peut pas dégrader la table. FNV est
// plus rapide sur des clés courtes, mais prévisible. Pour un outil qui ingère des
// journaux écrits par l'attaquant, ce choix se discute.

use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
pub struct Fnv1a(u64);

impl Hasher for Fnv1a {
    fn write(&mut self, bytes: &[u8]) {
        if self.0 == 0 {
            self.0 = 0xcbf2_9ce4_8422_2325;
        }
        for &b in bytes {
            self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

pub type FnvBuildHasher = BuildHasherDefault<Fnv1a>;
pub type FnvHashMap<K, V> = std::collections::HashMap<K, V, FnvBuildHasher>;
