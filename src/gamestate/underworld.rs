use std::time::Duration;

use chrono::{DateTime, Local};
use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::{EnumIter, IntoEnumIterator};

use super::{ArrSkip, CCGet, EnumMapGet, SFError, ServerTime};
use crate::gamestate::CGet;

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The information about a characters underworld
pub struct Underworld {
    /// All the buildings, that the underworld can have. If they are not yet
    /// build, they are level 0
    pub buildings: EnumMap<UnderworldBuildingType, UnderworldBuilding>,
    /// Information about all the buildable units in the underworld
    pub units: EnumMap<UnderworldUnitType, UnderworldUnits>,
    /// All information about the production of resources in the underworld
    pub production: EnumMap<UnderWorldResourceType, UnderworldProduction>,
    /// The `last_collectable` value in `UnderWorldResource` is always out of
    /// date. Refer to the `Fortress.last_collectable_updated` for more
    /// information
    pub last_collectable_update: Option<DateTime<Local>>,

    // Both XP&silver are not really resources, so I just have this here,
    // instead of in a resouce info struct like in fortress
    /// The current souls in the underworld
    pub souls_current: u64,
    /// The maximum amount of souls, that you can store in the underworld.  If
    /// `current == limit`, you will not be able to collect resources from
    /// the building
    pub souls_limit: u64,

    /// The building, that is currently being upgraded
    pub upgrade_building: Option<UnderworldBuildingType>,
    /// The time at which the upgrade is finished
    pub upgrade_finish: Option<DateTime<Local>>,
    /// The time the building upgrade began
    pub upgrade_begin: Option<DateTime<Local>>,

    /// The combined level of all buildings in the underworld
    pub total_level: u16,
    /// The amount of players, that have been lured into the underworld today
    pub lured_today: u16,
}

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The price an upgrade, or building something in the underworld costs. These
/// are always for one upgrade/build, which is important for unit builds
pub struct UnderworldCost {
    /// The time it takes to complete one build/upgrade
    pub time: Duration,
    /// The price in silver this costs
    pub silver: u64,
    /// The price in sould this costs
    pub souls: u64,
}

impl UnderworldCost {
    pub(crate) fn parse(data: &[i64]) -> Result<UnderworldCost, SFError> {
        Ok(UnderworldCost {
            time: Duration::from_secs(data[0] as u64),
            // Guessing here
            silver: data.csiget(1, "silver cost", u64::MAX)?,
            souls: data.csiget(2, "sould cost", u64::MAX)?,
        })
    }
}

impl Underworld {
    pub(crate) fn update_building_prices(
        &mut self,
        data: &[i64],
    ) -> Result<(), SFError> {
        for (pos, typ) in UnderworldBuildingType::iter().enumerate() {
            self.buildings.get_mut(typ).upgrade_cost = UnderworldCost::parse(
                data.skip(pos * 3, "underworld building prices")?,
            )?;
        }
        Ok(())
    }

    pub(crate) fn update_underworld_unit_prices(
        &mut self,
        data: &[i64],
    ) -> Result<(), SFError> {
        for (pos, typ) in UnderworldUnitType::iter().enumerate() {
            self.units.get_mut(typ).upgrade_price.next_level =
                data.csiget(pos * 3, "uunit next lvl", 0)?;
            self.units.get_mut(typ).upgrade_price.silver =
                data.csiget(1 + pos * 3, "uunit upgrade gold", 0)?;
            self.units.get_mut(typ).upgrade_price.souls =
                data.csiget(2 + pos * 3, "uunit upgrade gold", 0)?;
        }
        Ok(())
    }

    pub(crate) fn update(
        &mut self,
        data: &[i64],
        server_time: ServerTime,
    ) -> Result<(), SFError> {
        for (pos, typ) in UnderworldBuildingType::iter().enumerate() {
            self.buildings.get_mut(typ).level =
                data.csiget(448 + pos, "building level", 0)?;
        }

        for (i, typ) in UnderworldUnitType::iter().enumerate() {
            let start = 146 + i * 148;
            self.units.get_mut(typ).upgraded_amount =
                data.csiget(start, "uunit upgrade level", 0)?;
            self.units.get_mut(typ).count =
                data.csiget(start + 1, "uunit count", 0)?;
            self.units.get_mut(typ).atr_bonus =
                data.csiget(start + 2, "uunit atr bonus", 0)?;
            self.units.get_mut(typ).level =
                data.csiget(start + 3, "uunit level", 0)?;
        }

        let get_local = |spot, name| {
            let val = data.cget(spot, name)?;
            Ok(server_time.convert_to_local(val, name))
        };

        use UnderWorldResourceType::*;
        self.production.get_mut(Souls).last_collectable =
            data.csiget(459, "uu souls in building", 0)?;
        self.production.get_mut(Souls).limit =
            data.csiget(460, "uu sould max in building", 0)?;
        self.souls_limit = data.csiget(461, "uu souls max saved", 0)?;
        self.production.get_mut(Souls).per_hour =
            data.csiget(463, "uu souls per hour", 0)?;

        self.production.get_mut(Silver).last_collectable =
            data.csiget(464, "uu gold in building", 0)?;
        self.production.get_mut(Silver).limit =
            data.csiget(465, "uu max gold in building", 0)?;
        self.production.get_mut(Silver).per_hour =
            data.csiget(466, "uu gold ", 0)?;

        self.production.get_mut(ThirstForAdventure).last_collectable =
            data.csiget(473, "uu alu in building", 0)?;
        self.production.get_mut(ThirstForAdventure).limit =
            data.csiget(474, "uu max stored alu", 0)?;

        self.last_collectable_update = get_local(467, "uw resource time")?;
        self.upgrade_building = FromPrimitive::from_i64(data[468] - 1);
        self.upgrade_finish = get_local(469, "u expand end")?;
        self.upgrade_begin = get_local(470, "u expand begin")?;
        self.total_level = data.csiget(471, "uu max stored alu", 0)?;
        self.lured_today = data.csiget(472, "u battles today", 0)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, strum::EnumCount, Enum)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnderWorldResourceType {
    Souls = 0,
    Silver = 1,
    #[doc(alias = "ALU")]
    ThirstForAdventure = 2,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the producion of a resource in the fortress.  Note that
/// experience will not have some of these fields
pub struct UnderworldProduction {
    /// The amount the production building has already produced, that you can
    /// collect. Note that this value will be out of date by some amount of
    /// time. If you need the exact current amount collectable, look at
    /// `last_collectable_update`
    pub last_collectable: u64,
    /// The maximum amount of this resource, that this building can store. If
    /// `building_collectable == building_limit` the production stops
    pub limit: u64,
    /// The amount of this resource the coresponding production building
    /// produces per hour
    pub per_hour: u64,
}

#[derive(
    Debug, Clone, Copy, FromPrimitive, strum::EnumCount, Enum, EnumIter,
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnderworldBuildingType {
    HeartOfDarkness = 0,
    Gate = 1,
    GoldPit = 2,
    SoulExtractor = 3,
    GoblinPit = 4,
    TortureChamber = 5,
    GladiatorTrainer = 6,
    TrollBlock = 7,
    Adventuromatic = 8,
    Keeper = 9,
}

#[derive(Debug, Clone, Copy, strum::EnumCount, Enum, EnumIter)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnderworldUnitType {
    Goblin = 0,
    Troll = 1,
    Keeper = 2,
}

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnderworldBuilding {
    // 0 => not build
    pub level: u8,
    pub upgrade_cost: UnderworldCost,
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnderworldUnits {
    pub upgraded_amount: u16,
    pub count: u16,
    pub atr_bonus: u32,
    pub level: u16,
    pub upgrade_price: UnderworldUnitUpradeInfo,
}

#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnderworldUnitUpradeInfo {
    pub next_level: u16,
    pub silver: u32,
    pub souls: u32,
}
