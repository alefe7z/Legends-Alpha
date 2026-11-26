//! Mana: the magical power that species such as the Mage (`Species::Mage`)
//! are born with.
//!
//! Every Mana kind belongs to a [`ManaCategory`] (Elemental, Physical, ...)
//! and grants its abilities through an *innate item*: an invisible item that
//! acts as the character's main-hand "weapon" whenever their main hand is
//! empty. This lets mages cast with their bare hands while every existing
//! ability, skill bar and AI system keeps working unchanged.
//!
//! Mana kinds are entirely data-driven. To add a new one:
//! 1. Create its item in `assets/common/items/mana/` (copy an existing one).
//! 2. Add its abilities to `assets/common/abilities/ability_set_manifest.ron`
//!    under `Custom("Mana: <name>")`.
//! 3. Append it to the END of `assets/common/mana/manifest.ron`.
//! 4. Add its texts (`mana-<id>` with a `.desc` attribute) to
//!    `assets/voxygen/i18n/*/mana.ftl` and map the item to that key in
//!    `assets/common/item_i18n_manifest.ron`.
//! 5. Give the item an invisible model in
//!    `assets/voxygen/voxel/biped_weapon_manifest.ron` (copy an existing
//!    `common.items.mana.*` entry).
//!
//! The index of a Mana kind in the manifest is what is stored on the
//! character, so existing entries must never be reordered or removed.

use crate::{
    assets::{AssetExt, AssetHandle, BoxedError, FileAsset, load_ron},
    comp::body::humanoid,
};
use serde::Deserialize;
use std::borrow::Cow;

/// The broad family a Mana kind belongs to.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ManaCategory {
    /// Control of the elements: fire, water, earth, air, lightning...
    Elemental,
    /// Alterations of the body itself: super strength, super speed...
    Physical,
}

impl ManaCategory {
    /// Localisation key of the category name.
    pub fn i18n_key(self) -> &'static str {
        match self {
            ManaCategory::Elemental => "mana-category-elemental",
            ManaCategory::Physical => "mana-category-physical",
        }
    }
}

/// One kind of Mana, as described in `common.mana.manifest`.
#[derive(Clone, Debug, Deserialize)]
pub struct ManaKind {
    /// Stable identifier, also used to build the localisation key `mana-<id>`
    /// (its `.desc` attribute holds the description).
    pub id: String,
    pub category: ManaCategory,
    /// Asset path of the innate item that grants this Mana's abilities.
    pub item: String,
}

impl ManaKind {
    /// Localisation key of the name; the `desc` attribute holds the
    /// description.
    pub fn i18n_key(&self) -> String { format!("mana-{}", self.id) }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManaManifest(pub Vec<ManaKind>);

impl FileAsset for ManaManifest {
    const EXTENSION: &'static str = "ron";

    fn from_bytes(bytes: Cow<[u8]>) -> Result<Self, BoxedError> { load_ron(&bytes) }
}

impl ManaManifest {
    pub const PATH: &'static str = "common.mana.manifest";

    pub fn load() -> AssetHandle<Self> { Self::load_expect(Self::PATH) }

    /// Number of Mana kinds available.
    pub fn count() -> u8 { Self::load().read().0.len().min(u8::MAX as usize) as u8 }

    pub fn get(index: u8) -> Option<ManaKind> { Self::load().read().0.get(index as usize).cloned() }
}

/// The Mana kind of a humanoid body, if its species has Mana.
pub fn mana_of(body: &humanoid::Body) -> Option<ManaKind> {
    if body.species.has_mana() {
        ManaManifest::get(body.mana)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comp::{
        Item,
        item::{ItemKind, tool::AbilityMap},
    };

    #[test]
    fn mana_manifest_is_valid() {
        let manifest = ManaManifest::load().read();
        assert!(!manifest.0.is_empty(), "Mana manifest must not be empty");
        let ability_map = AbilityMap::load().read();
        for kind in &manifest.0 {
            let item = Item::new_from_asset(&kind.item).unwrap_or_else(|e| {
                panic!("Mana '{}' references missing item {}: {e:?}", kind.id, kind.item)
            });
            assert!(
                matches!(&*item.kind(), ItemKind::Tool(_)),
                "Mana item {} must be a Tool",
                kind.item
            );
            let spec = item
                .ability_spec()
                .unwrap_or_else(|| panic!("Mana item {} must have an ability_spec", kind.item));
            assert!(
                ability_map.get_ability_set(&*spec).is_some(),
                "Mana '{}' has no abilities in the ability set manifest",
                kind.id
            );
        }
    }
}
